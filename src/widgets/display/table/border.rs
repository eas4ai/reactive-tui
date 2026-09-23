use super::*;
use crate::{
    accessibility::{Node, Role},
    builder::ElementBuilder,
    component::ElementType,
    layout::style::StyleBuilder,
};

pub(in crate::widgets::display) fn enabled(border: &Border) -> bool {
    border.enabled && border.style != super::super::BorderStyle::None
}

pub(in crate::widgets) fn elements(border: &Border, width: usize, height: usize) -> Vec<Element> {
    use super::super::BorderStyle;
    if !enabled(border) || width < 2 || height < 2 {
        return Vec::new();
    }
    let (width, height) = (width - 2, height - 2);
    let (tl, tr, bl, br, h, v) = match border.style {
        BorderStyle::Double => ('╔', '╗', '╚', '╝', '═', '║'),
        BorderStyle::Rounded => ('╭', '╮', '╰', '╯', '─', '│'),
        BorderStyle::Thick => ('┏', '┓', '┗', '┛', '━', '┃'),
        _ => ('┌', '┐', '└', '┘', '─', '│'),
    };
    let line = h.to_string().repeat(width);
    let class = border
        .color
        .as_ref()
        .map_or_else(String::new, |color| format!("text-{color}"));
    let text = |text: String, x: f32, y: f32, width: usize, height: usize| {
        let mut node = Node::new(Role::Label);
        node.set_hidden();
        ElementBuilder::new(ElementType::Text(text))
            .styles(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(x)
                    .inset_top(y)
                    .width_px(width as f32)
                    .height_px(height as f32),
            )
            .class(&format!("whitespace-pre {class}"))
            .build()
            .with_accessibility(node)
    };
    let side = std::iter::repeat_n(v.to_string(), height)
        .collect::<Vec<_>>()
        .join("\n");
    vec![
        text(format!("{tl}{line}{tr}"), 0.0, 0.0, width + 2, 1),
        text(
            format!("{bl}{line}{br}"),
            0.0,
            (height + 1) as f32,
            width + 2,
            1,
        ),
        text(side.clone(), 0.0, 1.0, 1, height),
        text(side, (width + 1) as f32, 1.0, 1, height),
    ]
}
