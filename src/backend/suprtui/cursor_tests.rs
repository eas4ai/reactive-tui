use super::*;
use crate::{
    builder::ElementBuilder,
    component::{element::TextCursor, ElementType, LayoutType},
};
use ::suprtui::render::CursorStyle;
use std::sync::Mutex;

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<(Vec<u8>, bool)>>);
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        if std::mem::take(&mut self.0.lock().unwrap().1) {
            Err(io::Error::other("cursor flush failed"))
        } else {
            Ok(())
        }
    }
}
impl Capture {
    fn take(&self) -> String {
        String::from_utf8(std::mem::take(&mut self.0.lock().unwrap().0)).unwrap()
    }
}
fn caret(class: &str) -> Element {
    let mut element = Element::text("界R").with_class(class);
    element.metadata.text_cursor = Some(TextCursor {
        column: 1,
        style: CursorStyle::Line,
    });
    element
}
fn frame(children: Vec<Element>) -> Element {
    ElementBuilder::new(ElementType::Layout(LayoutType::Absolute))
        .class("w-full h-full")
        .children(children)
        .build()
}
fn present(backend: &mut SuprTuiBackend, root: &Element) {
    backend.render_frame(root).unwrap();
    backend.present().unwrap();
    backend.sync().unwrap();
}
#[test]
fn native_cursor_follows_painted_text_clipping_and_coverage() {
    let capture = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(8, 4, capture.clone()).unwrap();
    let cursor = caret("absolute left-2 top-1 w-3 h-1 whitespace-pre");
    present(&mut backend, &frame(vec![cursor.clone()]));
    let output = capture.take();
    assert!(output.contains("\x1b[6 q"));
    assert!(output.contains("\x1b[2;4H\x1b[?25h"));
    for cover in [
        Element::text("X").with_class("absolute left-3 top-1 w-1 h-1"),
        ElementBuilder::new(ElementType::Layout(LayoutType::Absolute))
            .class("absolute left-3 top-1 w-1 h-1 bg-red-500")
            .build(),
    ] {
        present(&mut backend, &frame(vec![cursor.clone(), cover]));
        let output = capture.take();
        assert!(output.contains("\x1b[?25l"));
        assert!(
            !output.contains("\x1b[?25h"),
            "covered cursor was shown: {output:?}"
        );
    }
    let clipped = ElementBuilder::new(ElementType::Layout(LayoutType::Absolute))
        .class("absolute left-0 top-0 w-1 h-1 overflow-hidden")
        .child(caret("absolute left-0 top-0 w-3 h-1 whitespace-pre"))
        .build();
    present(&mut backend, &frame(vec![clipped]));
    assert!(!capture.take().contains("\x1b[?25h"));
    present(&mut backend, &frame(vec![cursor]));
    assert!(capture.take().contains("\x1b[?25h"));
    present(&mut backend, &frame(Vec::new()));
    assert!(!capture.take().contains("\x1b[?25h"));
}
#[test]
fn native_cursor_retries_failed_output_and_does_not_leak_into_cell_frames() {
    let capture = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(8, 4, capture.clone()).unwrap();
    let root = frame(vec![caret("absolute left-0 top-0 w-3 h-1 whitespace-pre")]);
    backend.render_frame(&root).unwrap();
    capture.0.lock().unwrap().1 = true;
    // The write fails after present returned; the sync, like the next
    // present, reports it.
    backend.present().unwrap();
    assert!(backend.sync().is_err());
    capture.0.lock().unwrap().1 = false;
    capture.take();
    present(&mut backend, &root);
    let output = capture.take();
    assert!(output.contains("\x1b[6 q") && output.contains("\x1b[?25h"));
    backend
        .render_cells(Arc::new(
            CellFrame::new(
                8,
                4,
                vec![
                    crate::backend::FrameCell {
                        text: " ".into(),
                        width: 1,
                        foreground: [255; 3],
                        background: [0; 3],
                        attributes: 0,
                        decoration: Default::default(),
                    };
                    32
                ],
                Some((0, 0)),
            )
            .unwrap(),
        ))
        .unwrap();
    backend.present().unwrap();
    backend.sync().unwrap();
    let output = capture.take();
    assert!(!output.contains("\x1b[6 q"));
    assert!(
        output.contains("\x1b[0 q"),
        "default shape missing: {output:?}"
    );
}
#[test]
fn native_cursor_style_and_color_reset_when_host_session_is_restored() {
    let capture = Capture::default();
    let writer = Rc::new(RefCell::new(capture.clone()));
    let mut session = TerminalOutput::new(writer, true);
    session.enter().unwrap();
    capture.take();
    session.restore().unwrap();
    let output = capture.take();
    assert!(output.contains("\x1b[0 q\x1b]112\x07\x1b[?25h"));
    assert!(output.ends_with("\x1b[?1049l"));
}
