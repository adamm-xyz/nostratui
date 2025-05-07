use std::env;
use std::fs;
use std::process::Command;
use std::sync::Arc;
use anyhow::Result;
use nostr_sdk::prelude::Timestamp;

use crate::models::{NostrClient, Post};
use crate::error::NostratuiError;

pub async fn fetch_new_posts(client: &Arc<NostrClient>, last_login: Timestamp) -> Result<Vec<Post>> {
    client.fetch_notes_since(last_login).await
}

pub async fn post_note(client: &NostrClient, content: String) -> Result<()> {
    client.post_note(content).await
}

pub fn create_post_via_editor() -> Result<String> {
    let editor = env::var("EDITOR")
        .unwrap_or_else(|_| "vi".to_string());

    let mut temp_path = env::temp_dir();
    temp_path.push("note");

    let status = Command::new(editor)
        .arg(&temp_path)
        .status()?;

    if !status.success() {
        return Err(NostratuiError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Editor exited with non-zero status"
                )
        ).into());
    }

    let content = fs::read_to_string(&temp_path)?;
    let _ = fs::remove_file(&temp_path);
    Ok(content)
}

