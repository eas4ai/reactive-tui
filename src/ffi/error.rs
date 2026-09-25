//! Error handling for FFI

use std::os::raw::c_int;

/// Error codes for FFI functions
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactiveError {
    /// Operation succeeded
    Success = 0,
    /// Invalid parameter passed
    InvalidParameter = -1,
    /// Null pointer passed where not allowed
    NullPointer = -2,
    /// Buffer too small for operation
    BufferTooSmall = -3,
    /// Out of memory
    OutOfMemory = -4,
    /// Invalid UTF-8 string
    InvalidUtf8 = -5,
    /// Terminal not available or not supported
    TerminalNotAvailable = -6,
    /// Operation not supported
    NotSupported = -7,
    /// Resource already exists
    AlreadyExists = -8,
    /// Resource not found
    NotFound = -9,
    /// Invalid state for operation
    InvalidState = -10,
    /// Invalid pointer (use-after-free or corrupted)
    InvalidPointer = -11,
    /// Internal error occurred
    InternalError = -12,
    /// Panic occurred (bug in library)
    Panic = -99,
    /// Unknown error
    Unknown = -100,
}

impl ReactiveError {
    /// Convert to C integer
    pub fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if error is success
    pub fn is_success(self) -> bool {
        self == ReactiveError::Success
    }
}

impl From<std::io::Error> for ReactiveError {
    fn from(e: std::io::Error) -> Self {
        use std::io::ErrorKind;
        match e.kind() {
            ErrorKind::NotFound => ReactiveError::NotFound,
            ErrorKind::PermissionDenied => ReactiveError::NotSupported,
            ErrorKind::AlreadyExists => ReactiveError::AlreadyExists,
            ErrorKind::InvalidInput => ReactiveError::InvalidParameter,
            ErrorKind::InvalidData => ReactiveError::InvalidUtf8,
            ErrorKind::OutOfMemory => ReactiveError::OutOfMemory,
            _ => ReactiveError::Unknown,
        }
    }
}

/// Convert a Result to a ReactiveError, preserving error information
pub fn result_to_error<T>(result: Result<T, Box<dyn std::error::Error>>) -> ReactiveError {
    match result {
        Ok(_) => ReactiveError::Success,
        Err(e) => {
            // Log error in debug mode for better debugging
            #[cfg(debug_assertions)]
            log::debug!("FFI Error: {}", e);

            // Try to map specific error types
            let error_str = e.to_string().to_lowercase();
            if error_str.contains("null") || error_str.contains("pointer") {
                ReactiveError::NullPointer
            } else if error_str.contains("memory") || error_str.contains("allocation") {
                ReactiveError::OutOfMemory
            } else if error_str.contains("utf") || error_str.contains("encoding") {
                ReactiveError::InvalidUtf8
            } else if error_str.contains("terminal") {
                ReactiveError::TerminalNotAvailable
            } else if error_str.contains("not found") {
                ReactiveError::NotFound
            } else if error_str.contains("already exists") {
                ReactiveError::AlreadyExists
            } else if error_str.contains("not supported") {
                ReactiveError::NotSupported
            } else {
                ReactiveError::InternalError
            }
        }
    }
}

/// Convert a crate::error::Result to ReactiveError with better error mapping
pub fn crate_result_to_error<T>(result: crate::error::Result<T>) -> ReactiveError {
    match result {
        Ok(_) => ReactiveError::Success,
        Err(e) => {
            #[cfg(debug_assertions)]
            log::debug!("Crate Error: {}", e);

            // Map specific crate errors to FFI errors
            use crate::error::ReactiveError as CrateError;
            match e {
                CrateError::InvalidState { .. } => ReactiveError::InvalidState,
                _ => ReactiveError::InternalError,
            }
        }
    }
}
