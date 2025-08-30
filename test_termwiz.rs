use termwiz::caps::Capabilities;

fn main() {
    match Capabilities::new_from_env() {
        Ok(caps) => {
            println!("Available methods:");
            println!("Colors: {}", caps.colors());
            println!("Sixel: {}", caps.sixel());
            println!("Mouse: {}", caps.mouse_reporting());
            println!("Hyperlinks: {}", caps.hyperlinks());
            println!("Bracketed paste: {}", caps.bracketed_paste());
            println!("Alternate screen: {}", caps.alternate_screen());
            
            // Check what other methods are available
            println!("Debug: {:?}", caps);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
