use std::error::Error;
use std::fmt;
use zip::result::ZipError;

#[derive(Debug)]
pub struct RomAnalyzerError {
    details: String,
}

impl RomAnalyzerError {
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

impl From<ZipError> for RomAnalyzerError {
    fn from(err: ZipError) -> RomAnalyzerError {
        RomAnalyzerError::new(&format!("Zip Error: {}", err))
    }
}

impl From<std::io::Error> for RomAnalyzerError {
    fn from(err: std::io::Error) -> RomAnalyzerError {
        RomAnalyzerError::new(&format!("IO Error: {}", err))
    }
}
