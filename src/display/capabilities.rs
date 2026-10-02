use serde::{Deserialize, Serialize};

/// Display capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayCapabilities {
    /// Detected maximum viable FPS for this terminal/system
    pub max_fps: u32,
    /// Recommended FPS for optimal performance/quality balance
    pub recommended_fps: u32,
    /// Terminal type and capabilities
    pub terminal_info: TerminalInfo,
    /// System performance characteristics
    pub performance_profile: PerformanceProfile,
    /// VSync-like capabilities (terminal responsiveness)
    pub sync_capabilities: SyncCapabilities,
}

/// Information about the terminal and its capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInfo {
    /// Terminal program name (if detectable)
    pub program: Option<String>,
    /// Terminal supports high refresh rates
    pub supports_high_refresh: bool,
    /// Terminal has hardware acceleration
    pub has_gpu_acceleration: bool,
    /// Connection type (local, SSH, etc.)
    pub connection_type: ConnectionType,
    /// Color depth capabilities
    pub color_depth: ColorDepth,
}

/// Type of terminal connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Direct local terminal connection
    Local,
    /// SSH remote connection
    SSH,
    /// Running inside tmux session
    Tmux,
    /// Running inside GNU Screen session
    Screen,
    /// Web-based terminal (browser)
    Web,
    /// Unknown or undetected connection type
    Unknown,
}

/// Terminal color support capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorDepth {
    /// No color support (black and white only)
    Monochrome,
    /// 16 color support (basic ANSI colors)
    Color16,
    /// 256 color support (extended palette)
    Color256,
    /// 24-bit true color support (RGB)
    TrueColor,
}

/// System performance characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    /// Average render time in microseconds
    pub avg_render_time_us: f32,
    /// System can handle high FPS without dropping frames
    pub high_fps_capable: bool,
    /// CPU usage characteristics
    pub cpu_efficiency: f32,
    /// Memory usage efficiency
    pub memory_efficiency: f32,
}

/// Terminal synchronization and responsiveness capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCapabilities {
    /// Terminal can handle rapid updates smoothly
    pub smooth_updates: bool,
    /// Effective maximum update rate (measured)
    pub max_update_rate: u32,
    /// Input lag characteristics
    pub input_latency_ms: f32,
}

impl TerminalInfo {
    /// Detect terminal program and capabilities
    pub fn detect() -> Self {
        let program = std::env::var("TERM_PROGRAM")
            .ok()
            .or_else(|| std::env::var("TERMINAL_EMULATOR").ok());

        let connection_type = Self::detect_connection_type();
        let supports_high_refresh =
            Self::terminal_supports_high_refresh(&program, &connection_type);
        let has_gpu_acceleration = Self::detect_gpu_acceleration(&program);
        let color_depth = Self::detect_color_depth();

        TerminalInfo {
            program,
            supports_high_refresh,
            has_gpu_acceleration,
            connection_type,
            color_depth,
        }
    }

    fn detect_connection_type() -> ConnectionType {
        if std::env::var("SSH_CLIENT").is_ok() || std::env::var("SSH_TTY").is_ok() {
            ConnectionType::SSH
        } else if std::env::var("TMUX").is_ok() {
            ConnectionType::Tmux
        } else if std::env::var("STY").is_ok() {
            ConnectionType::Screen
        } else if std::env::var("TERM").is_ok_and(|t| t.contains("web")) {
            ConnectionType::Web
        } else {
            ConnectionType::Local
        }
    }

    fn terminal_supports_high_refresh(
        program: &Option<String>,
        connection: &ConnectionType,
    ) -> bool {
        match connection {
            ConnectionType::SSH => return false, // Network latency limits high refresh
            ConnectionType::Web => return false, // Browser limitations
            _ => {}
        }

        if let Some(prog) = program {
            match prog.as_str() {
                "iTerm.app" => true,        // Excellent performance
                "WezTerm" => true,          // GPU accelerated
                "Alacritty" => true,        // High performance
                "kitty" => true,            // GPU accelerated
                "Windows Terminal" => true, // Modern Windows terminal
                "Hyper" => false,           // Electron-based, slower
                "Terminal.app" => false,    // Basic macOS terminal
                _ => true,                  // Assume modern terminals can handle it
            }
        } else {
            true // Unknown terminal, optimistically assume capable
        }
    }

    fn detect_gpu_acceleration(program: &Option<String>) -> bool {
        if let Some(prog) = program {
            matches!(prog.as_str(), "WezTerm" | "Alacritty" | "kitty")
        } else {
            false
        }
    }

    fn detect_color_depth() -> ColorDepth {
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            if colorterm.contains("truecolor") || colorterm.contains("24bit") {
                return ColorDepth::TrueColor;
            }
        }

        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256") {
                return ColorDepth::Color256;
            } else if term.contains("color") {
                return ColorDepth::Color16;
            }
        }

        ColorDepth::Color16 // Conservative default
    }
}

impl DisplayCapabilities {
    /// Calculate maximum viable FPS based on all factors
    pub fn calculate_max_fps(&self) -> u32 {
        let mut max_fps = 240u32; // Start optimistically

        // Reduce based on connection type
        match self.terminal_info.connection_type {
            ConnectionType::SSH => max_fps = max_fps.min(60),
            ConnectionType::Web => max_fps = max_fps.min(60),
            ConnectionType::Tmux | ConnectionType::Screen => max_fps = max_fps.min(90),
            _ => {}
        }

        // Reduce based on terminal capabilities
        if !self.terminal_info.supports_high_refresh {
            max_fps = max_fps.min(60);
        }

        // Reduce based on performance
        if !self.performance_profile.high_fps_capable {
            max_fps = max_fps.min(60);
        } else if self.performance_profile.avg_render_time_us > 1000.0 {
            max_fps = max_fps.min(90);
        }

        // Limit by terminal sync capabilities
        max_fps = max_fps.min(self.sync_capabilities.max_update_rate * 2); // 2x headroom

        max_fps.max(30) // Never go below 30fps
    }

    /// Calculate recommended FPS from maximum
    pub fn calculate_recommended_fps(&self) -> u32 {
        let max_fps = self.calculate_max_fps();

        match max_fps {
            240.. => {
                if self.terminal_info.has_gpu_acceleration {
                    144
                } else {
                    120
                }
            }
            120..=239 => 90,
            60..=119 => 60,
            _ => 30,
        }
    }
}
