use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Post {
    pub user: String,
    pub timestamp: u64,
    pub datetime: String,
    pub content: String,
    pub id: String,
}
