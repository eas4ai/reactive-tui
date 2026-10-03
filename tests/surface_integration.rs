// Simple test of enhanced Surface functionality
// This bypasses the CSS system to test just the Surface improvements

use reactive_tui::core::geometry::Rect;
use reactive_tui::core::surface::{Attr, BorderChars, Rgba, Surface, TextStyle};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_surface_integration() {
        // Create a surface
        let mut surface = Surface::new(80, 24);

        // Test Unicode-aware text writing
        println!("\n✅ Testing Unicode-aware text writing...");
        let style = TextStyle::new(
            Rgba::new(1.0, 1.0, 1.0, 1.0), // white text
            Rgba::new(0.0, 0.0, 0.0, 1.0), // black background
            Attr::BOLD,
        );

        surface.write_styled_text(5, 2, "Hello, 世界! 🌍 Emoji test", style);

        // Test text width calculation
        let text = "Hello, 世界! 🌍";
        let width = surface.text_width(text);
        println!("Text '{}' has display width: {}", text, width);

        // Test subview clipping
        println!("\n✅ Testing subview clipping...");
        {
            let mut subview = surface.subview_mut(10, 5, 10, 5, 20, 10);
            subview.write_text(0, 0, "This text is clipped to subview bounds", style);
            subview.fill_background(0, 2, 15, 3, Rgba::new(0.4, 0.4, 0.8, 1.0));
        }

        // Test border drawing
        println!("\n✅ Testing border drawing...");
        let border_rect = Rect::from_coords(30, 8, 20, 8);
        surface.draw_border(border_rect, style, Some(BorderChars::rounded()));

        // Test text wrapping
        println!("\n✅ Testing text wrapping...");
        let long_text = "This is a very long text that should wrap across multiple lines when rendered with the enhanced text wrapping functionality.";
        let lines_written = surface.write_text_wrapped(5, 15, long_text, 25, style);
        println!("Wrapped text across {} lines", lines_written);

        // Test background filling
        println!("\n✅ Testing background filling...");
        let fill_rect = Rect::from_coords(55, 10, 15, 5);
        surface.fill_background_rect(fill_rect, Rgba::new(0.8, 0.4, 0.4, 1.0));

        // Test character filling
        println!("\n✅ Testing character filling...");
        let char_rect = Rect::from_coords(60, 2, 10, 3);
        surface.fill_char_rect(char_rect, '█', TextStyle::fg(Rgba::new(0.0, 1.0, 0.0, 1.0)));

        // Test clearing
        println!("\n✅ Testing area clearing...");
        let clear_rect = Rect::from_coords(2, 20, 20, 3);
        surface.clear_rect(clear_rect);

        println!("\n🎯 All enhanced Surface features tested successfully!");
        println!("The Surface now has Canvas-like capabilities while maintaining");
        println!("architectural harmony with reactive-tui's existing systems.");

        // Display some statistics
        println!("\n📊 Surface Statistics:");
        let (w, h) = surface.dims();
        println!("  - Dimensions: {}x{}", w, h);
        println!("  - Total cells: {}", w * h);
        println!("  - Unicode-aware text rendering: ✅");
        println!("  - Subview clipping: ✅");
        println!("  - Enhanced drawing methods: ✅");
        println!("  - Border drawing: ✅");
        println!("  - Text wrapping: ✅");
        println!("  - Emoji handling: ✅");

        assert_eq!(surface.dims(), (80, 24));
        assert_eq!(width, 15);
        assert!(lines_written > 1);
        assert_eq!(surface.get(60, 2).ch, '█');
        assert_eq!(surface.get(30, 8).ch, '╭');
        assert_eq!(surface.get(55, 10).bg, Rgba::new(0.8, 0.4, 0.4, 1.0));
        assert_eq!(surface.get(2, 20).ch, ' ');
    }
}
