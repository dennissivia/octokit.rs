use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct OctokitError {
    details: String,
}

impl OctokitError {
    fn new(msg: &str) -> OctokitError {
        OctokitError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for OctokitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for OctokitError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<serde_json::Error> for OctokitError {
    fn from(err: serde_json::Error) -> Self {
        OctokitError::new(err.description())
    }
}

impl From<reqwest::Error> for OctokitError {
    fn from(err: reqwest::Error) -> Self {
        OctokitError::new(err.description())
    }
}

impl From<hex::FromHexError> for OctokitError {
    fn from(err: hex::FromHexError) -> Self {
        OctokitError::new(err.description())
    }
}

impl From<openssl::error::ErrorStack> for OctokitError {
    fn from(err: openssl::error::ErrorStack) -> Self {
        OctokitError::new(err.description())
    }
}
