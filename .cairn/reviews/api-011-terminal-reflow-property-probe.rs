use reactive_tui::terminal::{AnsiParser, VirtualScreen};
fn feed(screen: &mut VirtualScreen, text: &str) {
    for event in AnsiParser::new().parse_bytes(text.as_bytes()) { screen.process_event(event); }
}
fn logical(screen: &VirtualScreen) -> String {
    let (width,height)=screen.size();
    let mut result=String::new();
    for y in 0..height {
        let cells=(0..width).map(|x|screen.cell_at(x,y).unwrap()).collect::<Vec<_>>();
        let wrap=cells.iter().position(|cell|cell.wrapped);
        let end=wrap.map_or_else(||cells.iter().rposition(|cell|cell.character!=" ").map_or(0,|i|i+1),|i|i+1);
        for cell in &cells[..end] {result.push_str(&cell.character);}
        if wrap.is_none() { result.push('\n'); }
    }
    result.trim_end_matches('\n').to_string()
}
#[test]
fn reflow_round_trips_mixed_graphemes_and_continued_input() {
    let glyphs=["a","界","e\u{301}","👩‍💻","Z","한"];
    let mut random=12345u64;
    for case in 0..100 {
        let mut expected=String::new();
        for index in 0..40 {
            random=random.wrapping_mul(6364136223846793005).wrapping_add(1);
            expected.push_str(glyphs[((random>>32)as usize)%glyphs.len()]);
            if index%7==6 {expected.push('\n');}
        }
        let mut screen=VirtualScreen::new(2+(case%12),64,10);
        feed(&mut screen,&expected.replace('\n',"\r\n"));
        assert_eq!(logical(&screen),expected,"initial case {case}");
        for width in [31,2,7,3,19,4,8,5,27,2] {
            screen.resize(width,64).unwrap();
            assert_eq!(logical(&screen),expected,"case {case}, width {width}");
            feed(&mut screen,"!"); expected.push('!');
            assert_eq!(logical(&screen),expected,"continued case {case}, width {width}");
            assert_eq!(screen.scrollback_len(),0);
        }
    }
}
