use std::io;
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum NostratuiError {
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("Nostr SDK error: {0}")]
    NostrSdk(String),
    
    #[error("JSON error: {0}")]
    Json(String),
    
    #[error("Key parsing error: {0}")]
    KeyParsing(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Anyhow error: {0}")]
    Anyhow(String),
}

impl From<Box<dyn std::error::Error>> for NostratuiError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        NostratuiError::Network(err.to_string())
    }
}

impl From<serde_json::Error> for NostratuiError {
    fn from(err: serde_json::Error) -> Self {
        NostratuiError::Json(err.to_string())
    }
}

impl From<io::Error> for NostratuiError {
    fn from(err: io::Error) -> Self {
        NostratuiError::Io(err.to_string())
    }
}

impl From<anyhow::Error> for NostratuiError {
    fn from(err: anyhow::Error) -> Self {
        NostratuiError::Anyhow(err.to_string())
    }
}
