//! Matrix Rain Effect
//! 
//! A terminal implementation of the iconic "digital rain" effect from The Matrix,
//! demonstrating the animation system with falling characters and fade effects.
//!
//! Press Q or ESC to quit.

use reactive_tui::prelude::*;
use reactive_tui::backend::CrosstermBackend;
use reactive_tui::builder::{div, to_element};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Matrix rain column data
#[derive(Debug, Clone)]
struct MatrixColumn {
    characters: Vec<char>,
    position: f32,
    speed: f32,
    brightness: f32,
    trail_length: usize,
}

/// The main Matrix Rain component
struct MatrixRain {
    columns: Arc<Mutex<Vec<MatrixColumn>>>,
    last_update: Arc<Mutex<Instant>>,
    width: usize,
    height: usize,
}

impl MatrixRain {
    fn new() -> Self {
        // Get terminal dimensions or use defaults
        let (width, height) = crossterm::terminal::size()
            .map(|(w, h)| (w as usize, h as usize))
            .unwrap_or((80, 24));
        
        let mut columns = Vec::new();
        
        // Initialize columns
        for i in 0..width {
            columns.push(Self::create_column(i, height));
        }
        
        Self {
            columns: Arc::new(Mutex::new(columns)),
            last_update: Arc::new(Mutex::new(Instant::now())),
            width,
            height,
        }
    }
    
    fn create_column(index: usize, _height: usize) -> MatrixColumn {
        // Use index and current time for pseudo-randomness
        let seed = (Instant::now().elapsed().as_nanos() as usize) + index * 1337;
        
        // Matrix-like characters
        let chars = vec![
            'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ',
            'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ',
            'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ',
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J'
        ];
        
        let trail_length = 5 + (seed % 15);
        let mut characters = Vec::new();
        
        for i in 0..trail_length {
            let char_idx = (seed + i * 7) % chars.len();
            characters.push(chars[char_idx]);
        }
        
        // Start above the screen
        let start_pos = -((seed % 20) as f32) - 5.0;
        
        MatrixColumn {
            characters,
            position: start_pos,
            speed: 8.0 + ((seed % 12) as f32),
            brightness: 0.6 + ((seed % 40) as f32 / 100.0),
            trail_length,
        }
    }
    
    fn update_animation(&self) {
        let now = Instant::now();
        let delta_time = {
            let mut last = self.last_update.lock().unwrap();
            let delta = now.duration_since(*last);
            *last = now;
            delta.as_secs_f32()
        };
        
        let mut columns = self.columns.lock().unwrap();
        
        for (i, column) in columns.iter_mut().enumerate() {
            // Update position
            column.position += column.speed * delta_time;
            
            // Reset when column goes off screen
            if column.position > (self.height + column.trail_length) as f32 {
                *column = Self::create_column(i, self.height);
            }
            
            // Randomly change a character occasionally
            let seed = (now.elapsed().as_nanos() + i as u128) as usize;
            if seed % 100 < 3 && !column.characters.is_empty() {
                let char_idx = seed % column.characters.len();
                let new_chars = ['0', '1', '2', 'ｱ', 'ｲ', 'ｳ'];
                column.characters[char_idx] = new_chars[seed % new_chars.len()];
            }
        }
    }
}

impl reactive_tui::app::RootComponent for MatrixRain {
    fn render(&self) -> Element {
        // Update animation state
        self.update_animation();
        
        let columns = self.columns.lock().unwrap();
        let mut elements = Vec::new();
        
        // Create the falling characters
        for (x, column) in columns.iter().enumerate() {
            let y_pos = column.position.floor() as i32;
            
            for (i, ch) in column.characters.iter().enumerate() {
                let char_y = y_pos + i as i32;
                
                // Skip if outside visible bounds
                if char_y < 0 || char_y >= self.height as i32 {
                    continue;
                }
                
                // Calculate fade based on position in trail
                let fade = 1.0 - (i as f32 / column.trail_length as f32);
                let brightness = fade * column.brightness;
                
                // Color based on position and brightness
                let color_class = if i == 0 {
                    // Leading character is bright white
                    "text-white font-bold"
                } else if brightness > 0.8 {
                    "text-green-300"
                } else if brightness > 0.6 {
                    "text-green-400"
                } else if brightness > 0.4 {
                    "text-green-500"
                } else if brightness > 0.2 {
                    "text-green-600"
                } else {
                    "text-green-700"
                };
                
                // Build character element
                // Using nested divs to position each character
                elements.push(
                    div()
                        .class("absolute")
                        .class(&format!("left-{} top-{}", x, char_y))
                        .class(color_class)
                        .child(to_element(ch.to_string()))
                        .build()
                );
            }
        }
        
        // Add some static "glitch" characters randomly
        let glitch_seed = Instant::now().elapsed().as_nanos() as usize;
        if glitch_seed % 30 == 0 {
            let glitch_x = glitch_seed % self.width;
            let glitch_y = (glitch_seed / 7) % self.height;
            elements.push(
                div()
                    .class("absolute")
                    .class(&format!("left-{} top-{}", glitch_x, glitch_y))
                    .class("text-white font-bold")
                    .child(to_element("◊"))
                    .build()
            );
        }
        
        // Main container with black background
        div()
            .class("relative w-screen h-screen bg-black overflow-hidden")
            .children(elements)
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("🌧️ Starting Matrix Rain...");
    println!("Press Q or ESC to quit");
    
    // Create backend
    let backend = CrosstermBackend::new()?;
    
    // Create the Matrix Rain component
    let matrix = MatrixRain::new();
    
    // Build and run the app
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(matrix)
        .build()?;
    
    app.run()
}