//! Unified error handling for reactive-tui
//!
//! This module provides a single, comprehensive error type that covers all
//! error conditions in the reactive-tui library. It replaces the various
//! ad-hoc error handling patterns with a consistent, well-documented system.

use thiserror::Error;

/// The main error type for reactive-tui operations
#[derive(Debug, Error)]
pub enum ReactiveError {
    /// I/O operation failed
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid parameter provided to function
    #[error("Invalid parameter: {message}")]
    InvalidParameter {
        /// Description of the invalid parameter
        message: String,
    },

    /// Invalid state for the requested operation
    #[error("Invalid state: {message}")]
    InvalidState {
        /// Description of the invalid state
        message: String,
    },

    /// Resource allocation or management error
    #[error("Resource error: {message}")]
    Resource {
        /// Description of the resource error
        message: String,
    },

    /// Terminal-related error
    #[error("Terminal error: {message}")]
    Terminal {
        /// Description of the terminal error
        message: String,
    },

    /// Configuration or parsing error
    #[error("Configuration error: {message}")]
    Config {
        /// Description of the configuration error
        message: String,
    },

    /// Component-related error
    #[error("Component error: {message}")]
    Component {
        /// Description of the component error
        message: String,
    },

    /// Layout or rendering error
    #[error("Layout error: {message}")]
    Layout {
        /// Description of the layout error
        message: String,
    },

    /// Animation system error
    #[error("Animation error: {message}")]
    Animation {
        /// Description of the animation error
        message: String,
    },

    /// Image processing or rendering error
    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    /// External tool execution error
    #[error("External tool error: {0}")]
    ExternalTool(String),

    /// Internal library error (should not happen in normal usage)
    #[error("Internal error: {message}")]
    Internal {
        /// Description of the internal error
        message: String,
    },
}

/// Convenient Result type alias
pub type Result<T> = std::result::Result<T, ReactiveError>;

impl ReactiveError {
    /// Create an invalid parameter error
    pub fn invalid_parameter(message: impl Into<String>) -> Self {
        Self::InvalidParameter {
            message: message.into(),
        }
    }

    /// Create an invalid state error
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::InvalidState {
            message: message.into(),
        }
    }

    /// Create a resource error
    pub fn resource(message: impl Into<String>) -> Self {
        Self::Resource {
            message: message.into(),
        }
    }

    /// Create a terminal error
    pub fn terminal(message: impl Into<String>) -> Self {
        Self::Terminal {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a component error
    pub fn component(message: impl Into<String>) -> Self {
        Self::Component {
            message: message.into(),
        }
    }

    /// Create a layout error
    pub fn layout(message: impl Into<String>) -> Self {
        Self::Layout {
            message: message.into(),
        }
    }

    /// Create an animation error
    pub fn animation(message: impl Into<String>) -> Self {
        Self::Animation {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create an internal error from a static string (no allocation)
    pub fn internal_static(message: &'static str) -> Self {
        Self::Internal {
            message: message.to_string(),
        }
    }
}

// Conversion to FFI error codes (only when FFI feature is enabled)
#[cfg(feature = "ffi")]
impl From<ReactiveError> for crate::ffi::ReactiveError {
    fn from(err: ReactiveError) -> Self {
        match err {
            ReactiveError::Io(io_err) => io_err.into(),
            ReactiveError::InvalidParameter { .. } => crate::ffi::ReactiveError::InvalidParameter,
            ReactiveError::InvalidState { .. } => crate::ffi::ReactiveError::InvalidState,
            ReactiveError::Resource { .. } => crate::ffi::ReactiveError::OutOfMemory,
            ReactiveError::Terminal { .. } => crate::ffi::ReactiveError::TerminalNotAvailable,
            ReactiveError::Config { .. } => crate::ffi::ReactiveError::InvalidParameter,
            ReactiveError::Component { .. } => crate::ffi::ReactiveError::NotFound,
            ReactiveError::Layout { .. } => crate::ffi::ReactiveError::InvalidParameter,
            ReactiveError::Animation { .. } => crate::ffi::ReactiveError::InvalidState,
            ReactiveError::Internal { .. } => crate::ffi::ReactiveError::Unknown,
            ReactiveError::ImageProcessing(_) => crate::ffi::ReactiveError::Unknown,
            ReactiveError::ExternalTool(_) => crate::ffi::ReactiveError::Unknown,
        }
    }
}

// Helper function to convert crossterm results
impl ReactiveError {
    /// Convert a crossterm Result to ReactiveError
    pub fn from_crossterm<T>(result: std::result::Result<T, std::io::Error>) -> Result<T> {
        result.map_err(ReactiveError::Io)
    }
}

// Helper macros for common error patterns
/// Macro for returning invalid parameter errors
#[macro_export]
macro_rules! invalid_parameter {
    ($msg:expr) => {
        return Err($crate::error::ReactiveError::invalid_parameter($msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::error::ReactiveError::invalid_parameter(format!($fmt, $($arg)*)))
    };
}

/// Macro for returning invalid state errors
#[macro_export]
macro_rules! invalid_state {
    ($msg:expr) => {
        return Err($crate::error::ReactiveError::invalid_state($msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::error::ReactiveError::invalid_state(format!($fmt, $($arg)*)))
    };
}

/// Macro for ensuring conditions are met
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !($cond) {
            return Err($err);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ReactiveError::invalid_parameter("test message");
        assert!(matches!(err, ReactiveError::InvalidParameter { .. }));
        assert_eq!(err.to_string(), "Invalid parameter: test message");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let rtui_err: ReactiveError = io_err.into();
        assert!(matches!(rtui_err, ReactiveError::Io(_)));
    }

    #[test]
    #[cfg(feature = "ffi")]
    fn test_ffi_error_conversion() {
        let rtui_err = ReactiveError::invalid_parameter("test");
        let ffi_err: crate::ffi::ReactiveError = rtui_err.into();
        assert_eq!(ffi_err, crate::ffi::ReactiveError::InvalidParameter);
    }

    #[test]
    fn test_macros() {
        fn test_function() -> Result<()> {
            invalid_parameter!("test error");
        }

        let result = test_function();
        assert!(result.is_err());
        assert!(matches!(
            result.expect_err("Result should be an error"),
            ReactiveError::InvalidParameter { .. }
        ));
    }
}
