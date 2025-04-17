use std::env;
use std::fs;
use std::path::{PathBuf, Path};

use crate::app::Post;

pub fn get_cache_file() -> PathBuf {
    // Check the XDG_CACHE_HOME environment variable first
    let base_cache_dir = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Default to ~/.cache if XDG_CACHE_HOME is not set
            let home = env::var_os("HOME").expect("HOME not set");
            Path::new(&home).join(".cache")
        });

    let app_cache_dir = base_cache_dir.join("nostratui");

    // Create the directory if it doesn't exist
    fs::create_dir_all(&app_cache_dir).expect("Failed to create cache directory");

    app_cache_dir.join("posts.json")
}

pub fn load_cached_posts() -> Vec<Post> {
    let cache_path = get_cache_file();
    if let Ok(data) = fs::read_to_string(cache_path) {
        serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    }
}

pub fn save_posts_to_cache(new_posts: Vec<Post>) {
    let mut cached_posts = load_cached_posts();
    for post in new_posts {
        if !cached_posts.iter().any(|p| p.id == post.id) {
            cached_posts.push(post);
        }
    }

    let cache_path = get_cache_file();
    if let Ok(json) = serde_json::to_string(&cached_posts) {
        let _ = fs::write(cache_path, json);
    }
}
