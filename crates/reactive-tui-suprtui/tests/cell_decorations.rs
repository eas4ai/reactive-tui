use std::{cell::RefCell, rc::Rc};
use suprtui::{
    ansi::{CellDecoration, UnderlineStyle, rgb_color},
    buffer::make_cell,
    render::{MemoryBackend, RenderStatus, Renderer},
    uni::pool::GraphemePool,
};

#[test]
fn decoration_only_changes_render_and_reset_without_style_leaks() {
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut renderer = Renderer::new(2, 1, pool, MemoryBackend::new()).unwrap();
    let fg = rgb_color(255, 255, 255, 255);
    let bg = rgb_color(0, 0, 0, 255);
    let plain = make_cell('x' as u32, fg, bg, 0);
    renderer.next_buffer().set(0, 0, plain);
    assert_eq!(renderer.render(false), RenderStatus::Rendered);
    for (underline, code) in [
        (UnderlineStyle::Single, "4:1"),
        (UnderlineStyle::Double, "4:2"),
        (UnderlineStyle::Curly, "4:3"),
        (UnderlineStyle::Dotted, "4:4"),
        (UnderlineStyle::Dashed, "4:5"),
    ] {
        let decoration = CellDecoration {
            underline,
            underline_color: Some([10, 20, 30]),
            overline: true,
        };
        renderer
            .next_buffer()
            .set(0, 0, plain.with_decoration(decoration));
        assert_eq!(renderer.render(false), RenderStatus::Rendered);
        let bytes = renderer.backend().frames().last().unwrap();
        let output = String::from_utf8_lossy(bytes);
        assert!(output.contains(&format!("\x1b[{code}m")), "{output:?}");
        assert!(output.contains("\x1b[58:2::10:20:30m"));
        assert!(output.contains("\x1b[53m"));
        renderer
            .next_buffer()
            .set(0, 0, plain.with_decoration(decoration));
        assert_eq!(renderer.render(false), RenderStatus::Skipped);
    }
    renderer.next_buffer().set(0, 0, plain);
    assert_eq!(renderer.render(false), RenderStatus::Rendered);
    let output = String::from_utf8_lossy(renderer.backend().frames().last().unwrap());
    assert!(output.contains("\x1b[0m"));
    assert!(!output.contains("\x1b[53m"));
    assert!(!output.contains("\x1b[58:"));
}

#[test]
fn wide_cells_and_composites_keep_decorations_and_clear_them_on_erase() {
    use suprtui::buffer::{InitOptions, OptimizedBuffer};
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut source = OptimizedBuffer::new(4, 1, InitOptions::new(Rc::clone(&pool))).unwrap();
    let mut dest = OptimizedBuffer::new(4, 1, InitOptions::new(pool)).unwrap();
    let fg = rgb_color(255, 255, 255, 255);
    let bg = rgb_color(0, 0, 0, 255);
    let decoration = CellDecoration {
        overline: true,
        ..Default::default()
    };
    source
        .draw_grapheme("界".as_bytes(), 2, 0, 0, fg, bg, 0)
        .unwrap();
    source.set(0, 0, source.get(0, 0).unwrap().with_decoration(decoration));
    assert_eq!(source.get(1, 0).unwrap().decoration, decoration);
    dest.draw_frame_buffer(0, 0, &source, None, None, None, None);
    assert_eq!(dest.get(0, 0).unwrap().decoration, decoration);
    assert_eq!(dest.get(1, 0).unwrap().decoration, decoration);
    dest.set(1, 0, make_cell(' ' as u32, fg, bg, 0));
    assert_eq!(
        dest.get(0, 0).unwrap().decoration,
        CellDecoration::default()
    );
    dest.clear(bg, None);
    assert_eq!(
        dest.get(1, 0).unwrap().decoration,
        CellDecoration::default()
    );
}
