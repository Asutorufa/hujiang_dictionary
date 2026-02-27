use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Http(reqwest::Error),
    Serialization(serde_json::Error),
    Api(String),
    Internal(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(e) => write!(f, "HTTP error: {}", e),
            Error::Serialization(e) => write!(f, "Serialization error: {}", e),
            Error::Api(e) => write!(f, "API error: {}", e),
            Error::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::Internal(e)
    }
}
