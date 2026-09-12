Example from README.md:43

```rust
use reactive_tui::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::builder()
        .title("My App")
        .size(80, 24)
        .component(|| {
            Element::new("div")
                .class("flex items-center justify-center bg-blue-500 text-white p-4")
                .text("Hello, Reactive-TUI!")
        })
        .build()?;
    
    app.run()
}
```

Example from README.md:65

```rust
use reactive_tui::widgets::Image;
use reactive_tui::core::terminal::Terminal;

let image = Image::from_file("photo.jpg")
    .with_max_size(80, 24)
    .with_preserve_aspect(true);

let capabilities = Terminal::detect_image_capabilities();
let output = image.render(&capabilities)?;
println!("{}", output);
```
