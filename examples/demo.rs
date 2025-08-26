use reactive_tui::core::renderer::Renderer;
use reactive_tui::core::surface::Rgba;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};

fn main() -> std::io::Result<()> {
    let debug = std::env::args().any(|a| a == "--debug");
    let mut r = Renderer::new(80, 24)?;
    r.begin_frame()?;
    r.clear(Rgba{r:0.0,g:0.0,b:0.0,a:1.0});

    // Demo tree: grid with padding and gap, nested flex, margins and text attrs
    use std::borrow::Cow;
    let root = NodeSpec { class: Cow::from("grid grid-cols-2 gap-2 p-2"), text: None, children: vec![
        NodeSpec { class: Cow::from("bg-blue-500 text-white p-1 underline"), text: Some(Cow::from("Hello")), children: vec![] },
        NodeSpec { class: Cow::from("flex flex-col gap-y-1"), text: None, children: vec![
            NodeSpec { class: Cow::from("flex flex-row gap-x-3"), text: None, children: vec![
                NodeSpec { class: Cow::from("text-red-500 font-bold"), text: Some(Cow::from("R")), children: vec![] },
                NodeSpec { class: Cow::from("text-green-500 italic"), text: Some(Cow::from("G")), children: vec![] },
                NodeSpec { class: Cow::from("text-blue-500 line-through"), text: Some(Cow::from("B")), children: vec![] },
            ]},
            NodeSpec { class: Cow::from("grid grid-cols-3 gap-1 my-1"), text: None, children: vec![
                NodeSpec { class: Cow::from("bg-gray-600 text-white p-1"), text: Some(Cow::from("1")), children: vec![] },
                NodeSpec { class: Cow::from("bg-red-400 text-white p-1"), text: Some(Cow::from("2")), children: vec![] },
                NodeSpec { class: Cow::from("bg-green-400 text-white p-1"), text: Some(Cow::from("3")), children: vec![] },
            ]},
        ]},
    ]};

    let surf = r.surface_mut();
    let opts = PaintOptions { debug_overlay: debug };
    layout_and_paint_with(&root, surf, 80, &opts);

    r.end_frame()?;
    // wait a bit so user can see
    std::thread::sleep(std::time::Duration::from_millis(800));
    r.shutdown()?;
    Ok(())
}

