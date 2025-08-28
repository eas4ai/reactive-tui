//! Error handling for FFI

use std::os::raw::c_int;

/// Error codes for FFI functions
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RTuiError {
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
    /// Panic occurred (bug in library)
    Panic = -99,
    /// Unknown error
    Unknown = -100,
}

impl RTuiError {
    /// Convert to C integer
    pub fn to_c_int(self) -> c_int {
        self as c_int
    }
    
    /// Check if error is success
    pub fn is_success(self) -> bool {
        self == RTuiError::Success
    }
}

impl From<std::io::Error> for RTuiError {
    fn from(e: std::io::Error) -> Self {
        use std::io::ErrorKind;
        match e.kind() {
            ErrorKind::NotFound => RTuiError::NotFound,
            ErrorKind::PermissionDenied => RTuiError::NotSupported,
            ErrorKind::AlreadyExists => RTuiError::AlreadyExists,
            ErrorKind::InvalidInput => RTuiError::InvalidParameter,
            ErrorKind::InvalidData => RTuiError::InvalidUtf8,
            ErrorKind::OutOfMemory => RTuiError::OutOfMemory,
            _ => RTuiError::Unknown,
        }
    }
}