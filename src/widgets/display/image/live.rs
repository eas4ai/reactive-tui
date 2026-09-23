use super::{Image, ImageDisplayMode, ImageFormat, ImageProcessor, ImageSource};
use crate::{
    builder::ElementBuilder,
    component::{Component, Element, ElementType, LayoutInfo, LayoutType, LifecycleEvent, Props},
    layout::style::StyleBuilder,
};
use std::sync::{Arc, Mutex};
mod animation;
mod blocks;
mod cells;
mod worker;

/// Cells the worker drew for an image: a tool's captured output, or the
/// renderer's blitters.
#[derive(Clone)]
pub(super) enum Cells {
    Captured(Arc<vt100::Screen>),
    Blitted(Arc<crate::layout::paint_tree::cells::CellGrid>),
}

#[derive(Clone, PartialEq)]
pub(super) struct LiveProps {
    pub image: Arc<Image>,
    pub format: Option<ImageFormat>,
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub(super) struct LiveImage {
    previous: LiveProps,
    image_id: u32,
    view: Mutex<View>,
}
struct View {
    worker: Option<worker::Worker>,
    request: Option<u64>,
    pixels: Option<Arc<image::RgbaImage>>,
    error: Option<String>,
    layout: Option<LayoutInfo>,
    ascii: Option<String>,
    cells: Option<Cells>,
}
impl View {
    fn request(&mut self, props: &LiveProps) {
        self.submit(props, false);
    }
    fn submit(&mut self, props: &LiveProps, reuse_pixels: bool) {
        let pixels = reuse_pixels.then(|| self.pixels.clone()).flatten();
        self.request = None;
        self.pixels = pixels.clone();
        self.error = None;
        self.ascii = None;
        self.cells = None;
        if matches!(&props.image.source, ImageSource::FilePath(path) if path.as_os_str().is_empty())
        {
            self.worker = None;
            return;
        }
        if self.worker.is_none() {
            match worker::Worker::new() {
                Ok(worker) => self.worker = Some(worker),
                Err(error) => {
                    self.error = Some(format!("Cannot start image loader: {error}"));
                    return;
                }
            }
        }
        self.request = self.worker.as_ref().map(|worker| {
            worker.submit_at(
                props.image.clone(),
                props.format,
                self.layout.map(|layout| {
                    let (w, h) = layout.content_size();
                    (w.max(0.0) as u32, h.max(0.0) as u32)
                }),
                pixels,
            )
        });
    }
    fn receive(&mut self) {
        if let Some(worker) = &self.worker {
            worker.observe();
            if let Some(response) = worker.take() {
                if self.request == Some(response.id) {
                    self.ascii = None;
                    self.cells = response.cells;
                    match response.result {
                        Ok(pixels) => {
                            self.pixels = Some(pixels);
                            self.error = None;
                        }
                        Err(error) => {
                            self.pixels = None;
                            self.error = Some(error);
                        }
                    }
                }
            }
        }
    }
}
impl Component for LiveImage {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        let mut view = View {
            worker: None,
            request: None,
            pixels: None,
            error: None,
            layout: None,
            ascii: None,
            cells: None,
        };
        view.request(&props);
        Self {
            previous: props,
            image_id: super::ProtocolRenderer::new().generate_image_id(),
            view: Mutex::new(view),
        }
    }
    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        let view = self.view.get_mut().unwrap();
        if self.previous.image.source != props.image.source || self.previous.format != props.format
        {
            view.request(props);
        } else if self.previous != *props {
            view.submit(props, true);
        }
        if self.previous != *props {
            view.ascii = None;
        }
        self.previous = props.clone();
        true
    }
    fn layout(&mut self, layout: LayoutInfo, props: &mut Self::Props, _: &mut ()) -> bool {
        let view = self.view.get_mut().unwrap();
        let changed = view
            .layout
            .is_none_or(|old| old.content_size() != layout.content_size());
        view.layout = Some(layout);
        if changed {
            view.ascii = None;
            view.receive();
            view.submit(props, true);
        }
        changed
    }
    fn render(&self, props: &Self::Props, _: &()) -> Element {
        let mut view = self.view.lock().unwrap();
        view.receive();
        let (width, height) = view.layout.map_or((0, 0), |layout| {
            let (width, height) = layout.content_size();
            (width.max(0.0) as u32, height.max(0.0) as u32)
        });
        if view.ascii.is_none() && view.cells.is_none() {
            let text = if props.image.display_mode == ImageDisplayMode::Fallback {
                props
                    .image
                    .fallback_text
                    .clone()
                    .unwrap_or_else(|| "Image".into())
            } else if let Some(error) = &view.error {
                format!("Image error: {error}")
            } else if let Some(pixels) = &view.pixels {
                let mut config = Image {
                    source: ImageSource::FilePath(Default::default()),
                    display_mode: ImageDisplayMode::AsciiArt,
                    preserve_aspect: props.image.preserve_aspect,
                    background_color: props.image.background_color,
                    quality: props.image.quality,
                    ..Image::default()
                };
                config.size_constraints = Some((width, height));
                ImageProcessor::new()
                    .ascii_from_pixels(pixels, &config)
                    .unwrap_or_else(|error| format!("Image error: {error}"))
            } else if view.request.is_some() {
                "Loading image…".into()
            } else {
                "Image has no source".into()
            };
            view.ascii = Some(text);
        }
        let insets = view.layout.map_or([0.0; 4], |layout| layout.insets);
        let mut child = ElementBuilder::new(ElementType::Layout(LayoutType::Absolute))
            .styles(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(insets[0])
                    .inset_top(insets[1])
                    .width_px(width as f32)
                    .height_px(height as f32)
                    .overflow_hidden(),
            )
            .child(match &view.cells {
                Some(Cells::Captured(screen)) => cells::element(screen),
                Some(Cells::Blitted(grid)) => cells::grid_element(grid.clone()),
                None => Element::text(view.ascii.as_deref().unwrap_or_default())
                    .with_class("whitespace-pre"),
            })
            .build();
        child.metadata.image_fallback = Some(self.image_id);
        child.metadata.inert = true;
        let mut root = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .width_percent(100.0)
                    .height_percent(100.0)
                    .min_width_px(0.0)
                    .min_height_px(0.0)
                    .overflow_hidden(),
            )
            .child(child)
            .build();
        let mut accessible = crate::accessibility::Node::new(crate::accessibility::Role::Image);
        accessible.set_label(props.image.fallback_text.as_deref().unwrap_or("Image"));
        root.metadata.accessibility = Some(accessible);
        if let Some(pixels) = &view.pixels {
            root.metadata.image = Some(Arc::new(super::paint::ImagePaint::new(
                self.image_id,
                Arc::clone(pixels),
                &props.image,
            )));
        }
        root
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut ()) {
        if event == LifecycleEvent::Unmount {
            let view = self.view.get_mut().unwrap();
            view.worker = None;
            view.pixels = None;
            view.cells = None;
            view.ascii = None;
            view.request = None;
        }
    }
}
