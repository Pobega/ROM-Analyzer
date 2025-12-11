//! Defines custom error types for ROM-Analyzer, providing a centralized way
//! to handle and propagate errors throughout the application.

use std::error::Error;
use std::fmt;

use zip::result::ZipError;

#[derive(Debug)]
pub struct RomAnalyzerError {
    details: String,
}

impl RomAnalyzerError {
    /// Creates a new [`RomAnalyzerError`] with the given message.
    ///
    /// # Arguments
    ///
    /// * `msg` - A string slice that describes the error.
    ///
    /// # Returns
    ///
    /// A new `RomAnalyzerError` instance.
    pub fn new(msg: &str) -> RomAnalyzerError {
        RomAnalyzerError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for RomAnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for RomAnalyzerError {
    fn description(&self) -> &str {
        &self.details
    }
}

/// Converts a `zip::result::ZipError` into a `RomAnalyzerError`.
impl From<ZipError> for RomAnalyzerError {
    fn from(err: ZipError) -> RomAnalyzerError {
        RomAnalyzerError::new(&format!("Zip Error: {}", err))
    }
}

/// Converts a `std::io::Error` into a `RomAnalyzerError`.
impl From<std::io::Error> for RomAnalyzerError {
    fn from(err: std::io::Error) -> RomAnalyzerError {
        RomAnalyzerError::new(&format!("IO Error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_new_error() {
        let error_msg = "Test error message";
        let err = RomAnalyzerError::new(error_msg);
        assert_eq!(err.details, error_msg);
    }

    #[test]
    fn test_display_trait() {
        let error_msg = "Display test";
        let err = RomAnalyzerError::new(error_msg);
        assert_eq!(format!("{}", err), error_msg);
    }

    #[test]
    fn test_from_zip_error() {
        let zip_err = ZipError::FileNotFound;
        let zip_err_display = format!("{}", zip_err);
        let err: RomAnalyzerError = zip_err.into();
        assert_eq!(err.details, format!("Zip Error: {}", zip_err_display));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = IoError::new(ErrorKind::NotFound, "File not found");
        let err: RomAnalyzerError = io_err.into();
        assert!(err.details.contains("IO Error"));
        assert!(err.details.contains("File not found"));
    }
}
