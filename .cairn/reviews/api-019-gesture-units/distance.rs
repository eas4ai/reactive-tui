use reactive_tui::{hooks::mouse::DragState, event::types::Position};
fn main() {
    let cell = DragState { drag_start: Some(Position::cell(0, 0)), current_position: Some(Position::cell(3, 4)), ..Default::default() };
    let pixel = DragState { drag_start: Some(Position::pixel(0, 0)), current_position: Some(Position::pixel(3, 4)), ..Default::default() };
    println!("CELL_DISTANCE={} PIXEL_DISTANCE={}", cell.distance(), pixel.distance());
    assert_eq!(cell.distance(), 5.0);
    assert_eq!(pixel.distance(), 5.0, "pixel drag distance was discarded");
}
