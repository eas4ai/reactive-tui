//! Unified error handling for reactive-tui
//!
//! This module provides a single, comprehensive error type that covers all
//! error conditions in the reactive-tui library. It replaces the various
//! ad-hoc error handling patterns with a consistent, well-documented system.

use thiserror::Error;

/// The main error type for reactive-tui operations
#[derive(Debug, Error)]
pub enum RTuiError {
    /// I/O operation failed
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid parameter provided to function
    #[error("Invalid parameter: {message}")]
    InvalidParameter { message: String },

    /// Invalid state for the requested operation
    #[error("Invalid state: {message}")]
    InvalidState { message: String },

    /// Resource allocation or management error
    #[error("Resource error: {message}")]
    Resource { message: String },

    /// Terminal-related error
    #[error("Terminal error: {message}")]
    Terminal { message: String },

    /// Configuration or parsing error
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Component-related error
    #[error("Component error: {message}")]
    Component { message: String },

    /// Layout or rendering error
    #[error("Layout error: {message}")]
    Layout { message: String },

    /// Animation system error
    #[error("Animation error: {message}")]
    Animation { message: String },

    /// Image processing or rendering error
    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    /// External tool execution error
    #[error("External tool error: {0}")]
    ExternalTool(String),

    /// Internal library error (should not happen in normal usage)
    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Convenient Result type alias
pub type Result<T> = std::result::Result<T, RTuiError>;

impl RTuiError {
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
impl From<RTuiError> for crate::ffi::RTuiError {
    fn from(err: RTuiError) -> Self {
        match err {
            RTuiError::Io(io_err) => io_err.into(),
            RTuiError::InvalidParameter { .. } => crate::ffi::RTuiError::InvalidParameter,
            RTuiError::InvalidState { .. } => crate::ffi::RTuiError::InvalidState,
            RTuiError::Resource { .. } => crate::ffi::RTuiError::OutOfMemory,
            RTuiError::Terminal { .. } => crate::ffi::RTuiError::TerminalNotAvailable,
            RTuiError::Config { .. } => crate::ffi::RTuiError::InvalidParameter,
            RTuiError::Component { .. } => crate::ffi::RTuiError::NotFound,
            RTuiError::Layout { .. } => crate::ffi::RTuiError::InvalidParameter,
            RTuiError::Animation { .. } => crate::ffi::RTuiError::InvalidState,
            RTuiError::Internal { .. } => crate::ffi::RTuiError::Unknown,
        }
    }
}

// Helper function to convert crossterm results
impl RTuiError {
    /// Convert a crossterm Result to RTuiError
    pub fn from_crossterm<T>(result: std::result::Result<T, std::io::Error>) -> Result<T> {
        result.map_err(RTuiError::Io)
    }
}

// Helper macros for common error patterns
#[macro_export]
macro_rules! invalid_parameter {
    ($msg:expr) => {
        return Err($crate::error::RTuiError::invalid_parameter($msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::error::RTuiError::invalid_parameter(format!($fmt, $($arg)*)))
    };
}

#[macro_export]
macro_rules! invalid_state {
    ($msg:expr) => {
        return Err($crate::error::RTuiError::invalid_state($msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::error::RTuiError::invalid_state(format!($fmt, $($arg)*)))
    };
}

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
        let err = RTuiError::invalid_parameter("test message");
        assert!(matches!(err, RTuiError::InvalidParameter { .. }));
        assert_eq!(err.to_string(), "Invalid parameter: test message");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let rtui_err: RTuiError = io_err.into();
        assert!(matches!(rtui_err, RTuiError::Io(_)));
    }

    #[test]
    #[cfg(feature = "ffi")]
    fn test_ffi_error_conversion() {
        let rtui_err = RTuiError::invalid_parameter("test");
        let ffi_err: crate::ffi::RTuiError = rtui_err.into();
        assert_eq!(ffi_err, crate::ffi::RTuiError::InvalidParameter);
    }

    #[test]
    fn test_macros() {
        fn test_function() -> Result<()> {
            invalid_parameter!("test error");
        }

        let result = test_function();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RTuiError::InvalidParameter { .. }
        ));
    }
}
