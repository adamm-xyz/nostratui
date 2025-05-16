use std::env;
use std::fs;
use std::path::{PathBuf, Path};

use crate::models::post::Post;
use crate::error::NostratuiError;

pub fn get_cache_file() -> Result<PathBuf, NostratuiError> {
    // Check the XDG_CACHE_HOME environment variable first
    let base_cache_dir = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = env::var_os("HOME")
                .ok_or(NostratuiError::Config("Home environment variable not set".to_string()))
                .unwrap_or_else(|_| PathBuf::from("").into());

            Path::new(&home).join(".cache")
        });

    let app_cache_dir = base_cache_dir.join("nostratui");

    // Create the directory if it doesn't exist
    fs::create_dir_all(&app_cache_dir)
        .map_err(|e| NostratuiError::Cache(format!("Failed to create cache directory: {}",e)))?;

    Ok(app_cache_dir.join("posts.json"))
}

pub fn load_cached_posts() -> Result<Vec<Post>,NostratuiError> {
    let cache_path = get_cache_file()?;
        match fs::read_to_string(&cache_path) {
        Ok(data) => {
            serde_json::from_str(&data)
                .map_err(|e| NostratuiError::Cache(format!("Failed to parse cache data: {}", e)))
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // If file doesn't exist, return empty vector
            Ok(Vec::new())
        },
        Err(e) => {
            Err(NostratuiError::Cache(format!("Failed to read cache file: {}", e)))
        }
    }
}

pub fn save_posts_to_cache(new_posts: Vec<Post>) -> Result<(), NostratuiError> {
    let mut cached_posts = load_cached_posts()?;

    for post in new_posts {
        if !cached_posts.iter().any(|p| p.id == post.id) {
            cached_posts.push(post);
        }
    }

    let cache_path = get_cache_file()?;
    let json = serde_json::to_string(&cached_posts)
        .map_err(|e| NostratuiError::Cache(format!("Failed to serialize posts: {}", e)))?;
    
    fs::write(cache_path, json)
        .map_err(|e| NostratuiError::Cache(format!("Failed to write cache file: {}", e)))?;
    
    Ok(())
}

pub fn is_cache_empty() -> Result<bool,NostratuiError> {
    let cache_files = load_cached_posts()?;
    Ok(cache_files.is_empty())
}
