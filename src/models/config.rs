use std::fs::{File, OpenOptions};
use std::io::{Write,BufReader};
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

use crate::error::NostratuiError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub key: String,
    pub relays: Vec<String>,
    pub contacts: Vec<(String,String)>,
    pub last_login: Option<u64>,
}

impl Config {
    pub fn load() -> Result<Self,NostratuiError> {
        let config_path = dirs::home_dir()
            .ok_or_else(|| NostratuiError::Config("Could not find home directory".to_string()))?
            .join(".config/nostratui/config.json");
        
        let file = File::open(&config_path)
            .with_context(|| format!("Failed to open config file at {:?}", config_path))?;

        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)
            .context("Failed to prase config JSON")?;
        
        Ok(config)
    }
    
    pub fn save(&self) -> Result<(),NostratuiError> {
        let config_path = dirs::home_dir()
            .ok_or_else(|| NostratuiError::Config("Could not find home directory".to_string()))?
            .join(".config/nostratui/config.json");

        let json = serde_json::to_string_pretty(&self)
            .context("Failed to serialize config to JSON")?;

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&config_path)
            .with_context(|| format!("Failed to open conf file for writing at {:?}",config_path))?;

        file.write_all(json.as_bytes())
            .context("Failed to write config data")?;

        Ok(())
    }
    
    pub fn get_last_login(&self) -> Timestamp {
        match self.last_login {
            Some(login_date) => Timestamp::from_secs(login_date),
            None => Timestamp::now() - Timestamp::from_secs(60*60*24*7) // Default to 7 days ago
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
