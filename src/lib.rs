pub mod models;
pub mod views;
pub mod controllers;
pub mod cli;
pub mod error;

// Re-exports for convenience
pub use models::{NostrClient, Config, Post};
pub use controllers::{start_app, init_feed};
