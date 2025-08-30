use std::env;

fn main() {
    println!("Terminal Environment Debug");
    println!("=========================");
    println!();

    // Collect all relevant environment variables
    let env_vars = [
        "TERM",
        "TERM_PROGRAM",
        "TERM_PROGRAM_VERSION",
        "COLORTERM",
        "VTE_VERSION",
        "GNOME_TERMINAL_SCREEN",
        "GNOME_TERMINAL_SERVICE",
        "KONSOLE_VERSION",
        "ALACRITTY_SOCKET",
        "KITTY_WINDOW_ID",
        "WEZTERM_EXECUTABLE",
        "ITERM_SESSION_ID",
        "TMUX",
        "STY", // GNU Screen
        "SSH_CLIENT",
        "SSH_TTY",
        "DISPLAY",
        "WAYLAND_DISPLAY",
        "XDG_SESSION_TYPE",
        "DESKTOP_SESSION",
        "XDG_CURRENT_DESKTOP",
    ];

    println!("🔍 Environment Variables:");
    println!("--------------------------");
    for var in &env_vars {
        match env::var(var) {
            Ok(value) => println!("  {}: {}", var, value),
            Err(_) => println!("  {}: <not set>", var),
        }
    }

    println!();
    println!("🎯 Terminal Detection Logic:");
    println!("-----------------------------");

    let term_program = env::var("TERM_PROGRAM").unwrap_or_default().to_lowercase();
    let term = env::var("TERM").unwrap_or_default().to_lowercase();
    let colorterm = env::var("COLORTERM").unwrap_or_default().to_lowercase();
    let vte_version = env::var("VTE_VERSION").ok();

    // Detection logic
    let is_gnome_terminal = term_program.contains("gnome-terminal")
        || vte_version.is_some()
        || env::var("GNOME_TERMINAL_SCREEN").is_ok()
        || env::var("GNOME_TERMINAL_SERVICE").is_ok();

    let is_konsole = env::var("KONSOLE_VERSION").is_ok();
    let is_alacritty = env::var("ALACRITTY_SOCKET").is_ok() || term_program.contains("alacritty");
    let is_kitty = env::var("KITTY_WINDOW_ID").is_ok() || term_program.contains("kitty");
    let is_wezterm = env::var("WEZTERM_EXECUTABLE").is_ok() || term_program.contains("wezterm");
    let is_iterm = env::var("ITERM_SESSION_ID").is_ok() || term_program.contains("iterm");
    let is_tmux = env::var("TMUX").is_ok();
    let is_screen = env::var("STY").is_ok();

    let has_truecolor = colorterm.contains("truecolor") || colorterm.contains("24bit");
    let is_xterm_compatible =
        term.contains("xterm") || term.contains("screen") || term.contains("tmux");

    println!(
        "  Gnome Terminal: {}",
        if is_gnome_terminal {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!(
        "  KDE Konsole: {}",
        if is_konsole { "✅ Yes" } else { "❌ No" }
    );
    println!(
        "  Alacritty: {}",
        if is_alacritty { "✅ Yes" } else { "❌ No" }
    );
    println!("  Kitty: {}", if is_kitty { "✅ Yes" } else { "❌ No" });
    println!("  WezTerm: {}", if is_wezterm { "✅ Yes" } else { "❌ No" });
    println!("  iTerm2: {}", if is_iterm { "✅ Yes" } else { "❌ No" });
    println!("  Tmux: {}", if is_tmux { "✅ Yes" } else { "❌ No" });
    println!(
        "  GNU Screen: {}",
        if is_screen { "✅ Yes" } else { "❌ No" }
    );
    println!(
        "  Has True Color: {}",
        if has_truecolor { "✅ Yes" } else { "❌ No" }
    );
    println!(
        "  xterm Compatible: {}",
        if is_xterm_compatible {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );

    println!();
    println!("🎨 Expected Capabilities:");
    println!("-------------------------");

    let is_modern_terminal = is_kitty
        || is_wezterm
        || is_iterm
        || is_alacritty
        || is_gnome_terminal
        || is_konsole
        || has_truecolor;
    let is_high_performance = is_kitty || is_wezterm || is_alacritty;

    println!(
        "  Modern Terminal: {}",
        if is_modern_terminal {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!(
        "  High Performance: {}",
        if is_high_performance {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );

    // Color depth
    let color_depth = if has_truecolor || is_gnome_terminal || is_modern_terminal {
        "True Color (24-bit)"
    } else if term.contains("256") || is_xterm_compatible {
        "256 Colors"
    } else if term.contains("color") {
        "16 Colors"
    } else {
        "Monochrome"
    };
    println!("  Color Depth: {}", color_depth);

    // Mouse support
    println!("  Mouse Support:");
    println!("    Basic: ✅ Yes (universal)");
    println!(
        "    Drag: {}",
        if is_modern_terminal {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!(
        "    Motion: {}",
        if is_modern_terminal {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!(
        "    Pixels: {}",
        if is_high_performance {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!(
        "    SGR Mode: {}",
        if is_modern_terminal || is_gnome_terminal {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );

    // Graphics support
    println!("  Graphics Support:");
    println!(
        "    Sixel: {}",
        if is_wezterm || is_alacritty || is_xterm_compatible || is_gnome_terminal {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!(
        "    Kitty Graphics: {}",
        if is_kitty { "✅ Yes" } else { "❌ No" }
    );
    println!(
        "    iTerm2 Images: {}",
        if is_iterm { "✅ Yes" } else { "❌ No" }
    );

    // Protocol support
    println!("  Protocol Support:");
    println!(
        "    Synchronized Output: {}",
        if is_modern_terminal || is_gnome_terminal {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!(
        "    Hyperlinks (OSC 8): {}",
        if is_modern_terminal || is_gnome_terminal {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!(
        "    Focus Tracking: {}",
        if is_modern_terminal {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!(
        "    Bracketed Paste: {}",
        if !term.contains("dumb") {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );

    println!();
    println!("💡 Tips:");
    println!("--------");
    println!("  • Run this in different terminals to see how detection varies");
    println!("  • Gnome Terminal should show VTE_VERSION and modern capabilities");
    println!("  • Kitty/WezTerm should show high-performance features");
    println!("  • SSH sessions will show SSH_CLIENT/SSH_TTY variables");

    if is_gnome_terminal {
        println!("  🎉 Gnome Terminal detected! This should have excellent capabilities.");
    } else if is_modern_terminal {
        println!("  🚀 Modern terminal detected! Great capabilities expected.");
    } else {
        println!("  ⚠️  Basic terminal detected. Limited capabilities expected.");
    }
}
