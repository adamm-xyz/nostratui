use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NostratuiError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Nostr SDK error: {0}")]
    NostrSdk(String),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Key parsing error: {0}")]
    KeyParsing(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Cache error: {0}")]
    Cache(String),
}

impl From<Box<dyn std::error::Error>> for NostratuiError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        NostratuiError::Network(err.to_string())
    }
}
