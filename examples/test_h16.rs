use reactive_tui::layout::css::parsers::parse_spacing_terminal;
use reactive_tui::layout::css::sizing::apply_height;
use reactive_tui::layout::style::StyleBuilder;

fn main() {
    // Test if h-16 parsing works
    let result = parse_spacing_terminal("h-16", "h-");
    eprintln!("parse_spacing_terminal('h-16', 'h-') = {:?}", result);
    
    // Test if apply_height works
    let sb = StyleBuilder::new();
    let result = apply_height("h-16", sb);
    eprintln!("apply_height('h-16') = {}", result.is_some());
    
    if let Some(sb) = result {
        let style = sb.build();
        eprintln!("Style height = {:?}", style.size.height);
    }
}