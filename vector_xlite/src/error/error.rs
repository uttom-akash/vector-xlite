use std::fmt;
use std::io;

#[derive(Debug)]
pub enum VecXError {
    ExtensionLoadError(String),
    SqlError(String),
    InvalidQueryError(String),
    DataParsingError(String),
    IoError(String),
    Other(String),
}

impl fmt::Display for VecXError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VecXError::ExtensionLoadError(s) => write!(f, "extension load error: {}", s),
            VecXError::SqlError(s) => write!(f, "sql error: {}", s),
            VecXError::InvalidQueryError(s) => write!(f, "invalid query error: {}", s),
            VecXError::DataParsingError(s) => write!(f, "data parsing error: {}", s),
            VecXError::IoError(s) => write!(f, "io error: {}", s),
            VecXError::Other(s) => write!(f, "error: {}", s),
        }
    }
}

impl std::error::Error for VecXError {}

impl From<rusqlite::Error> for VecXError {
    fn from(e: rusqlite::Error) -> Self {
        VecXError::SqlError(e.to_string())
    }
}

// impl Into<VecXError> for rusqlite::Error {
//     fn into(self) -> VecXError {
//         VecXError::SqlError(self.to_string())
//     }
// }

impl From<io::Error> for VecXError {
    fn from(e: io::Error) -> Self {
        VecXError::IoError(e.to_string())
    }
}

impl From<r2d2::Error> for VecXError {
    fn from(e: r2d2::Error) -> Self {
        VecXError::Other(e.to_string())
    }
}
