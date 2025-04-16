// src/config.rs
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Write,BufReader};
use std::path::PathBuf;
use nostr_sdk::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub key: String,
    pub relays: Vec<String>,
    pub contacts: Vec<String>,
    pub last_login: Option<u64>,
}

impl Config {
    pub fn load() -> Result<(Self, PathBuf)> {
        let config_path = dirs::home_dir()
            .expect("Could not find home directory")
            .join(".config/nostratui/config.json");
        
        let file = File::open(&config_path)?;
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;
        
        Ok((config, config_path))
    }
    
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
    
    pub fn get_last_login(&self) -> Timestamp {
        match self.last_login {
            Some(login_date) => Timestamp::from_secs(login_date),
            None => Timestamp::from_secs(60*60*24*7) // Default to 7 days ago
        }
    }
    
    pub fn update_last_login(&mut self) {
        let timestamp_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        self.last_login = Some(timestamp_now);
    }
}
