use super::{Image, ImageDisplayMode, ImageFormat, ImageSource};
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

/// Cells the worker drew for an image: a tool's captured output, the
/// renderer's blitters, or ASCII art, which is the picture in its own mode
/// and the text a pixel protocol falls back to where the host cannot show
/// it. Each costs work in proportion to the image's area, so none of them
/// is drawn on the App's thread (BAR-005).
#[derive(Clone)]
pub(super) enum Cells {
    Captured(Arc<vt100::Screen>),
    Blitted(Arc<crate::layout::paint_tree::cells::CellGrid>),
    Text(Arc<str>),
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
    /// Whether the worker has not answered the latest request yet: the
    /// image is still being prepared (BAR-003).
    pending: bool,
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
        self.pending = false;
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
        self.pending = self.worker.is_some();
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
    /// The image's state as a screen reader announces it: what failed, its
    /// size once decoded, or that it is still loading.
    fn describe(&self) -> String {
        if let Some(error) = &self.error {
            format!("Image error: {error}")
        } else if let Some(pixels) = &self.pixels {
            format!("{} by {} pixels", pixels.width(), pixels.height())
        } else if self.pending {
            "Loading image".into()
        } else {
            "Image has no source".into()
        }
    }
    fn receive(&mut self) {
        if let Some(worker) = &self.worker {
            worker.observe();
            if let Some(response) = worker.take() {
                if self.request == Some(response.id) {
                    // A request sent before the image knew its size decodes
                    // the pixels but draws no cells, so the area stays empty
                    // until the request its first layout sends is answered
                    // (BAR-003).
                    self.pending = self.layout.is_none()
                        && response.cells.is_none()
                        && response.result.is_ok();
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
            pending: false,
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
            } else if view.pixels.is_some() {
                // The worker is drawing the picture at this size; until it
                // answers the frame shows nothing, never the cells drawn for
                // another size (BAR-003).
                String::new()
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
                Some(Cells::Captured(screen)) => {
                    cells::element(screen, &crate::theme::Theme::active())
                }
                Some(Cells::Blitted(grid)) => cells::grid_element(grid.clone()),
                Some(Cells::Text(text)) => {
                    Element::text(text.to_string()).with_class("whitespace-pre")
                }
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
        accessible.set_description(view.describe());
        if view.pending {
            accessible.set_busy();
        }
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
            view.pending = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props() -> LiveProps {
        LiveProps {
            image: Arc::new(Image {
                display_mode: ImageDisplayMode::AsciiArt,
                ..Image::default()
            }),
            format: None,
        }
    }

    /// A live image in a given state, with no worker, so rendering reads that
    /// state instead of a worker's answer.
    fn in_state(pending: bool, pixels: Option<(u32, u32)>, error: Option<&str>) -> LiveImage {
        LiveImage {
            previous: props(),
            image_id: 1,
            view: Mutex::new(View {
                worker: None,
                request: pending.then_some(1),
                pending,
                pixels: pixels.map(|(w, h)| Arc::new(image::RgbaImage::new(w, h))),
                error: error.map(str::to_owned),
                layout: None,
                ascii: None,
                cells: None,
            }),
        }
    }

    /// Whether the image's rendered root is marked busy.
    fn busy(image: &LiveImage, props: &LiveProps) -> bool {
        image
            .render(props, &())
            .metadata
            .accessibility
            .as_ref()
            .is_some_and(|node| node.is_busy())
    }

    /// Calls `receive` until `done` holds for the view, failing after a hang
    /// guard rather than a timing bound.
    fn receive_until(image: &mut LiveImage, done: impl Fn(&View) -> bool) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let view = image.view.get_mut().unwrap();
        view.receive();
        while !done(view) {
            assert!(
                std::time::Instant::now() < deadline,
                "the image worker did not answer"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
            view.receive();
        }
    }

    /// BAR-003: until the worker has drawn the picture at the image's size,
    /// the area is empty and the node is busy. The first request goes out
    /// before the image knows its size, so its answer decodes the pixels
    /// but draws no cells; the image stays busy until its first layout's
    /// request is answered, and only then is a frame that is not busy one
    /// that shows the picture.
    #[test]
    fn bar_003_an_image_decoded_before_its_first_layout_stays_busy_until_drawn() {
        let mut props = LiveProps {
            image: Arc::new(Image {
                source: ImageSource::RawBytes {
                    data: [[220u8, 40, 40], [40, 40, 220]].repeat(8).concat(),
                    width: 4,
                    height: 4,
                    format: ImageFormat::RGB888,
                },
                display_mode: ImageDisplayMode::AsciiArt,
                ..Image::default()
            }),
            format: None,
        };
        let mut image = LiveImage::new(props.clone());
        receive_until(&mut image, |view| view.pixels.is_some());
        assert!(
            image.view.get_mut().unwrap().cells.is_none(),
            "no cells are drawn before the image knows its size"
        );
        assert!(
            busy(&image, &props),
            "an image with nothing drawn yet must be busy"
        );
        let bounds = crate::event::hit::Bounds::new(0.0, 0.0, 8.0, 4.0);
        image.layout(LayoutInfo::from_bounds(bounds), &mut props, &mut ());
        receive_until(&mut image, |view| view.cells.is_some());
        assert!(
            !busy(&image, &props),
            "the image drew its picture, so it is no longer busy"
        );
    }

    /// BAR-003: the image's screen-reader node describes its state, marks
    /// it busy while the worker prepares it, and its drawn content is inert,
    /// so it has no pointer action that would need a keyboard equivalent.
    #[test]
    fn bar_003_the_image_describes_its_state_to_the_screen_reader() {
        for (image, description, busy) in [
            (in_state(true, None, None), "Loading image", true),
            (
                in_state(false, Some((96, 32)), None),
                "96 by 32 pixels",
                false,
            ),
            (
                in_state(false, None, Some("not a PNG")),
                "Image error: not a PNG",
                false,
            ),
            (in_state(false, None, None), "Image has no source", false),
        ] {
            let root = image.render(&props(), &());
            let node = root
                .metadata
                .accessibility
                .as_ref()
                .expect("an accessibility node");
            assert_eq!(node.inner.role(), crate::accessibility::Role::Image);
            // The label is the image's fallback text.
            assert_eq!(
                node.inner.label(),
                props().image.fallback_text.as_deref().or(Some("Image"))
            );
            assert_eq!(node.inner.description(), Some(description));
            assert_eq!(node.is_busy(), busy, "{description}");
            assert!(
                root.children.iter().all(|child| child.metadata.inert),
                "the drawn content must be inert"
            );
        }
    }
}
