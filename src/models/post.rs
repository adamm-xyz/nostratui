use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub user: String,
    pub timestamp: u64,
    pub datetime: String,
    pub content: String,
    pub id: String,
    pub root_id: Option<String>,
    pub reply_id: Option<String>,
    #[serde(default)]
    pub mentions: Vec<String>,
    #[serde(default)]
    pub participants: Vec<String>,
}

impl Post {
    pub fn is_reply(&self) -> bool {
        self.reply_id.is_some()
    }

    pub fn is_root(&self) -> bool {
        self.root_id.is_none() && self.reply_id.is_none()
    }

    pub fn is_thread_reply(&self) -> bool {
        self.root_id.is_some() && self.reply_id.is_some()
    }
}
