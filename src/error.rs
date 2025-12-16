//! Defines custom error types for ROM-Analyzer, providing a centralized way
//! to handle and propagate errors throughout the application.

use std::error::Error;
use std::fmt;

use zip::result::ZipError;

#[derive(Debug)]
pub enum RomAnalyzerError {
    /// File format or extension is not supported
    UnsupportedFormat(String),
    /// ROM data is too small for analysis
    DataTooSmall {
        file_size: usize,
        required_size: usize,
        details: String,
    },
    /// Header data is invalid or corrupted
    InvalidHeader(String),
    /// Failed to parse specific data fields
    ParsingError(String),
    /// Checksum validation failed
    ChecksumMismatch(String),
    /// Error processing archive files (ZIP, CHD, etc.)
    ArchiveError(String),
    /// I/O operation failed
    IoError(std::io::Error),
    /// ZIP archive operation failed
    ZipError(ZipError),
    /// CHD archive operation failed
    ChdError(chd::Error),
    /// Generic error with custom message
    Generic(String),
}

impl RomAnalyzerError {
    /// Creates a new generic [`RomAnalyzerError`] with the given message.
    ///
    /// # Arguments
    ///
    /// * `msg` - A string slice that describes the error.
    ///
    /// # Returns
    ///
    /// A new [`RomAnalyzerError::Generic`] instance.
    pub fn new(msg: &str) -> RomAnalyzerError {
        RomAnalyzerError::Generic(msg.to_string())
    }
}

impl fmt::Display for RomAnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RomAnalyzerError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            RomAnalyzerError::DataTooSmall {
                file_size,
                required_size,
                details,
            } => {
                write!(
                    f,
                    "ROM data too small: {} bytes, requires at least {} bytes. {}",
                    file_size, required_size, details
                )
            }
            RomAnalyzerError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
            RomAnalyzerError::ParsingError(msg) => write!(f, "Parsing error: {}", msg),
            RomAnalyzerError::ChecksumMismatch(msg) => write!(f, "Checksum mismatch: {}", msg),
            RomAnalyzerError::ArchiveError(msg) => write!(f, "Archive error: {}", msg),
            RomAnalyzerError::IoError(err) => write!(f, "IO error: {}", err),
            RomAnalyzerError::ZipError(err) => write!(f, "ZIP error: {}", err),
            RomAnalyzerError::ChdError(err) => write!(f, "CHD error: {}", err),
            RomAnalyzerError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for RomAnalyzerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RomAnalyzerError::IoError(err) => Some(err),
            RomAnalyzerError::ZipError(err) => Some(err),
            RomAnalyzerError::ChdError(err) => Some(err),
            _ => None,
        }
    }
}

/// Converts a `zip::result::ZipError` into a [`RomAnalyzerError`].
impl From<ZipError> for RomAnalyzerError {
    fn from(err: ZipError) -> RomAnalyzerError {
        RomAnalyzerError::ZipError(err)
    }
}

/// Converts a `std::io::Error` into a [`RomAnalyzerError`].
impl From<std::io::Error> for RomAnalyzerError {
    fn from(err: std::io::Error) -> RomAnalyzerError {
        RomAnalyzerError::IoError(err)
    }
}

/// Converts a `Box<dyn Error>` into a [`RomAnalyzerError`].
impl From<Box<dyn Error>> for RomAnalyzerError {
    fn from(err: Box<dyn Error>) -> RomAnalyzerError {
        RomAnalyzerError::Generic(err.to_string())
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
        match err {
            RomAnalyzerError::Generic(msg) => assert_eq!(msg, error_msg),
            _ => panic!("Expected Generic variant"),
        }
    }

    #[test]
    fn test_display_trait() {
        let error_msg = "Display test";
        let err = RomAnalyzerError::Generic(error_msg.to_string());
        assert_eq!(format!("{}", err), error_msg);
    }

    #[test]
    fn test_display_unsupported_format() {
        let err = RomAnalyzerError::UnsupportedFormat("test.ext".to_string());
        assert_eq!(format!("{}", err), "Unsupported format: test.ext");
    }

    #[test]
    fn test_display_data_too_small() {
        let err = RomAnalyzerError::DataTooSmall {
            file_size: 100,
            required_size: 200,
            details: "Header missing".to_string(),
        };
        assert_eq!(
            format!("{}", err),
            "ROM data too small: 100 bytes, requires at least 200 bytes. Header missing"
        );
    }

    #[test]
    fn test_from_zip_error() {
        let zip_err = ZipError::FileNotFound;
        let zip_err_display = format!("{}", zip_err);
        let err: RomAnalyzerError = zip_err.into();
        match err {
            RomAnalyzerError::ZipError(_) => assert_eq!(
                format!("{}", err),
                format!("ZIP error: {}", zip_err_display)
            ),
            _ => panic!("Expected ZipError variant"),
        }
    }

    #[test]
    fn test_from_io_error() {
        let io_err = IoError::new(ErrorKind::NotFound, "File not found");
        let err: RomAnalyzerError = io_err.into();
        match err {
            RomAnalyzerError::IoError(_) => assert!(format!("{}", err).contains("IO error")),
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_error_source_method() {
        // Test that source() returns the wrapped error for IoError
        let io_err = IoError::new(ErrorKind::NotFound, "File not found");
        let rom_err = RomAnalyzerError::IoError(io_err);
        assert!(rom_err.source().is_some());
        assert_eq!(rom_err.source().unwrap().to_string(), "File not found");

        // Test that source() returns the wrapped error for ZipError
        let zip_err = ZipError::FileNotFound;
        let rom_err = RomAnalyzerError::ZipError(zip_err);
        assert!(rom_err.source().is_some());

        // Test that source() returns None for non-wrapped errors
        let rom_err = RomAnalyzerError::Generic("test".to_string());
        assert!(rom_err.source().is_none());

        let rom_err = RomAnalyzerError::UnsupportedFormat("test".to_string());
        assert!(rom_err.source().is_none());

        let rom_err = RomAnalyzerError::DataTooSmall {
            file_size: 100,
            required_size: 200,
            details: "test".to_string(),
        };
        assert!(rom_err.source().is_none());

        let rom_err = RomAnalyzerError::InvalidHeader("test".to_string());
        assert!(rom_err.source().is_none());
    }

    #[test]
    fn test_error_source_chd_error() {
        // Test ChdError source by creating an invalid CHD and checking the error
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let chd_path = dir.path().join("test.chd");
        std::fs::write(&chd_path, b"invalid chd data").unwrap();

        // Try to analyze the invalid CHD file
        let result = crate::archive::chd::analyze_chd_file(&chd_path);
        assert!(result.is_err());

        if let Err(RomAnalyzerError::ChdError(chd_err)) = result {
            // If we get a ChdError, verify source() works
            let rom_err = RomAnalyzerError::ChdError(chd_err);
            assert!(rom_err.source().is_some(), "ChdError should have a source");
        } else {
            panic!("Expected ChdError, but got {:?}", result.unwrap_err());
        }
    }
}
