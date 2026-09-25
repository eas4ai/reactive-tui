//! Granular mouse tracking system with multiple support levels
//!
//! This module provides a comprehensive mouse tracking system that supports
//! different levels of mouse interaction, from basic clicks to pixel-perfect
//! motion tracking, with automatic capability detection and fallback.

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use std::io::{self, Write};

/// Mouse capabilities that can be detected
#[derive(Debug, Clone, Default)]
pub struct MouseCapabilities {
    /// Basic mouse support (clicks)
    pub basic: bool,
    /// Drag tracking support
    pub drag: bool,
    /// Motion tracking support
    pub motion: bool,
    /// Pixel-level mouse coordinates
    pub pixels: bool,
    /// SGR mouse mode support
    pub sgr_mode: bool,
    /// Mouse wheel support
    pub wheel: bool,
}

/// Mouse tracking levels from basic to advanced
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MouseLevel {
    /// No mouse tracking
    None = 0,
    /// Basic click events only (mode 9)
    Basic = 1,
    /// Click and drag events (mode 1000)
    Drag = 2,
    /// All motion events including hover (mode 1003)
    Motion = 3,
    /// Pixel-level coordinates (mode 1016)
    Pixels = 4,
}

/// Mouse coordinate system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateSystem {
    /// Character-based coordinates (default)
    Character,
    /// Pixel-based coordinates (if supported)
    Pixel,
}

/// Mouse reporting format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReportingFormat {
    /// Standard X10 format (limited to 223x223)
    X10,
    /// UTF-8 format (supports larger coordinates)
    Utf8,
    /// SGR format (most modern, supports all features)
    Sgr,
    /// URXVT format (alternative extended format)
    Urxvt,
}

/// Comprehensive mouse tracker with automatic capability detection
pub struct MouseTracker {
    /// Current tracking level
    current_level: MouseLevel,
    /// Current coordinate system
    coordinate_system: CoordinateSystem,
    /// Current reporting format
    reporting_format: ReportingFormat,
    /// Terminal capabilities
    capabilities: Option<MouseCapabilities>,
    /// Whether mouse capture is currently active
    capture_active: bool,
    /// Statistics for performance monitoring
    stats: MouseStats,
}

/// Mouse tracking statistics
#[derive(Debug, Clone, Default)]
pub struct MouseStats {
    /// Total events processed
    pub events_processed: u64,
    /// Events by type
    /// Number of click events processed
    pub click_events: u64,
    /// Number of mouse move events processed
    pub move_events: u64,
    /// Number of drag events processed
    pub drag_events: u64,
    /// Number of wheel scroll events processed
    pub wheel_events: u64,
    /// Performance metrics
    pub avg_processing_time_us: f32,
    /// Error counts
    /// Number of event parsing errors
    pub parse_errors: u64,
    /// Number of capability detection errors
    pub capability_errors: u64,
}

/// Mouse tracking configuration
#[derive(Debug, Clone)]
pub struct MouseConfig {
    /// Desired tracking level
    pub level: MouseLevel,
    /// Preferred coordinate system
    pub coordinate_system: CoordinateSystem,
    /// Preferred reporting format
    pub reporting_format: ReportingFormat,
    /// Enable focus tracking
    pub focus_tracking: bool,
    /// Enable bracketed paste mode
    pub bracketed_paste: bool,
    /// Fallback behavior when capabilities are limited
    pub fallback_strategy: FallbackStrategy,
}

/// Strategy for handling limited terminal capabilities
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FallbackStrategy {
    /// Use the highest available level
    BestAvailable,
    /// Fail if exact level is not supported
    Strict,
    /// Disable mouse tracking if exact level is not supported
    Disable,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            level: MouseLevel::Basic,
            coordinate_system: CoordinateSystem::Character,
            reporting_format: ReportingFormat::Sgr,
            focus_tracking: false,
            bracketed_paste: true,
            fallback_strategy: FallbackStrategy::BestAvailable,
        }
    }
}

impl Default for MouseTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseTracker {
    /// Create a new mouse tracker
    pub fn new() -> Self {
        Self {
            current_level: MouseLevel::None,
            coordinate_system: CoordinateSystem::Character,
            reporting_format: ReportingFormat::Sgr,
            capabilities: None,
            capture_active: false,
            stats: MouseStats::default(),
        }
    }

    /// Create a mouse tracker with terminal capabilities
    pub fn with_capabilities(capabilities: MouseCapabilities) -> Self {
        Self {
            current_level: MouseLevel::None,
            coordinate_system: CoordinateSystem::Character,
            reporting_format: ReportingFormat::Sgr,
            capabilities: Some(capabilities),
            capture_active: false,
            stats: MouseStats::default(),
        }
    }

    /// Initialize mouse tracking with configuration
    pub fn initialize(&mut self, config: MouseConfig) -> io::Result<MouseLevel> {
        // Determine the best available level based on capabilities
        let target_level = self.determine_target_level(&config)?;

        // Set up the tracking level
        self.set_level(target_level)?;

        // Configure additional features
        if config.focus_tracking {
            self.enable_focus_tracking()?;
        }

        if config.bracketed_paste {
            self.enable_bracketed_paste()?;
        }

        Ok(target_level)
    }

    /// Set mouse tracking level
    pub fn set_level(&mut self, level: MouseLevel) -> io::Result<()> {
        if self.current_level == level {
            return Ok(());
        }

        // Disable current level first
        self.disable_current_level()?;

        // Enable new level
        match level {
            MouseLevel::None => {
                // Already disabled
            }
            MouseLevel::Basic => {
                self.enable_basic_tracking()?;
            }
            MouseLevel::Drag => {
                self.enable_drag_tracking()?;
            }
            MouseLevel::Motion => {
                self.enable_motion_tracking()?;
            }
            MouseLevel::Pixels => {
                self.enable_pixel_tracking()?;
            }
        }

        self.current_level = level;
        Ok(())
    }

    /// Get current tracking level
    pub fn current_level(&self) -> MouseLevel {
        self.current_level
    }

    /// Get tracking statistics
    pub fn stats(&self) -> &MouseStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = MouseStats::default();
    }

    /// Check if a specific level is supported
    pub fn supports_level(&self, level: MouseLevel) -> bool {
        if let Some(caps) = &self.capabilities {
            match level {
                MouseLevel::None => true,
                MouseLevel::Basic => caps.basic,
                MouseLevel::Drag => caps.drag,
                MouseLevel::Motion => caps.motion,
                MouseLevel::Pixels => caps.pixels,
            }
        } else {
            // Conservative assumption without capabilities
            matches!(level, MouseLevel::None | MouseLevel::Basic)
        }
    }

    /// Get the highest supported level
    pub fn max_supported_level(&self) -> MouseLevel {
        if let Some(caps) = &self.capabilities {
            if caps.pixels {
                MouseLevel::Pixels
            } else if caps.motion {
                MouseLevel::Motion
            } else if caps.drag {
                MouseLevel::Drag
            } else if caps.basic {
                MouseLevel::Basic
            } else {
                MouseLevel::None
            }
        } else {
            MouseLevel::Basic // Conservative default
        }
    }

    /// Shutdown mouse tracking and cleanup
    pub fn shutdown(&mut self) -> io::Result<()> {
        self.disable_current_level()?;
        self.disable_focus_tracking()?;
        self.disable_bracketed_paste()?;

        if self.capture_active {
            execute!(io::stdout(), DisableMouseCapture)?;
            self.capture_active = false;
        }

        Ok(())
    }

    fn determine_target_level(&self, config: &MouseConfig) -> io::Result<MouseLevel> {
        let requested = config.level;
        let max_supported = self.max_supported_level();

        match config.fallback_strategy {
            FallbackStrategy::Strict => {
                if self.supports_level(requested) {
                    Ok(requested)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        format!("Mouse level {:?} not supported", requested),
                    ))
                }
            }
            FallbackStrategy::BestAvailable => Ok(requested.min(max_supported)),
            FallbackStrategy::Disable => {
                if self.supports_level(requested) {
                    Ok(requested)
                } else {
                    Ok(MouseLevel::None)
                }
            }
        }
    }

    fn disable_current_level(&mut self) -> io::Result<()> {
        match self.current_level {
            MouseLevel::None => Ok(()),
            MouseLevel::Basic => self.send_escape_sequence("\x1b[?9l"),
            MouseLevel::Drag => self.send_escape_sequence("\x1b[?1000l"),
            MouseLevel::Motion => self.send_escape_sequence("\x1b[?1003l"),
            MouseLevel::Pixels => {
                // Disable both pixel mode and the base tracking
                self.send_escape_sequence("\x1b[?1016l")?;
                self.send_escape_sequence("\x1b[?1003l")
            }
        }
    }

    fn enable_basic_tracking(&mut self) -> io::Result<()> {
        execute!(io::stdout(), EnableMouseCapture)?;
        self.capture_active = true;
        self.send_escape_sequence("\x1b[?9h")?;
        self.setup_reporting_format()
    }

    fn enable_drag_tracking(&mut self) -> io::Result<()> {
        execute!(io::stdout(), EnableMouseCapture)?;
        self.capture_active = true;
        self.send_escape_sequence("\x1b[?1000h")?;
        self.setup_reporting_format()
    }

    fn enable_motion_tracking(&mut self) -> io::Result<()> {
        execute!(io::stdout(), EnableMouseCapture)?;
        self.capture_active = true;
        self.send_escape_sequence("\x1b[?1003h")?;
        self.setup_reporting_format()
    }

    fn enable_pixel_tracking(&mut self) -> io::Result<()> {
        execute!(io::stdout(), EnableMouseCapture)?;
        self.capture_active = true;
        // Enable motion tracking first, then pixel coordinates
        self.send_escape_sequence("\x1b[?1003h")?;
        self.send_escape_sequence("\x1b[?1016h")?;
        self.coordinate_system = CoordinateSystem::Pixel;
        self.setup_reporting_format()
    }

    fn setup_reporting_format(&mut self) -> io::Result<()> {
        match self.reporting_format {
            ReportingFormat::Sgr => self.send_escape_sequence("\x1b[?1006h"),
            ReportingFormat::Utf8 => self.send_escape_sequence("\x1b[?1005h"),
            ReportingFormat::Urxvt => self.send_escape_sequence("\x1b[?1015h"),
            ReportingFormat::X10 => {
                // X10 is the default, no setup needed
                Ok(())
            }
        }
    }

    fn enable_focus_tracking(&mut self) -> io::Result<()> {
        self.send_escape_sequence("\x1b[?1004h")
    }

    fn disable_focus_tracking(&mut self) -> io::Result<()> {
        self.send_escape_sequence("\x1b[?1004l")
    }

    fn enable_bracketed_paste(&mut self) -> io::Result<()> {
        self.send_escape_sequence("\x1b[?2004h")
    }

    fn disable_bracketed_paste(&mut self) -> io::Result<()> {
        self.send_escape_sequence("\x1b[?2004l")
    }

    fn send_escape_sequence(&self, sequence: &str) -> io::Result<()> {
        let mut stdout = io::stdout();
        stdout.write_all(sequence.as_bytes())?;
        stdout.flush()
    }
}

impl Drop for MouseTracker {
    fn drop(&mut self) {
        // Attempt graceful shutdown with error logging
        if let Err(e) = self.shutdown() {
            // Log error but don't panic during drop
            log::warn!("MouseTracker shutdown failed: {}", e);

            // Attempt emergency cleanup of critical terminal state
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                use crossterm::{event::DisableMouseCapture, execute};
                let _ = execute!(std::io::stdout(), DisableMouseCapture);
            }));
        }
    }
}
