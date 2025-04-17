use std::env;
use std::fs;
use std::process::Command;

use nostr_sdk::prelude::*;

use nostratui::cli::Flags;
use nostratui::client::NostrClient;
use nostratui::app;
use nostratui::config::Config;
use nostratui::cache;


#[tokio::main]
async fn main() -> Result<()> {
    //Get Flags
    let flags = Flags::from_args();

    // Load config
    let (mut config, config_path) = Config::load()?;

    // Initialize client and connect relays
    let mut client = NostrClient::new(config.key.clone());
    client.connect_relays(config.relays.clone()).await?;

    if flags.post() {
        //client post
        match edit_string() {
            Ok(note) => client.post_note(note).await?,
            Err(e) => eprintln!("Error creating post: {}", e),
        }
    } else {
        // Get last login time
        let last_login = config.get_last_login();
        println!("{:?}",last_login.clone().as_u64());

        // Get contacts
        if config.contacts.is_empty() {
            config.contacts = client.fetch_contacts()
                .await
                .into_iter()
                .map(|pk| pk.to_bech32().unwrap())
                .collect();
        }
        client.set_contacts(config.contacts.clone()).await?;

        // Get posts to read, add to cache
        let new_posts = client.fetch_notes_since(last_login).await?;
        cache::save_posts_to_cache(new_posts);

        // Save new config
        config.update_last_login();
        config.save(&config_path)?;

        app::start_tui();
    }
    Ok(())
}

fn edit_string() -> Result<String> {
    let editor = env::var("EDITOR")
        .unwrap_or_else(|_| "vi".to_string());

    let mut temp_path = env::temp_dir();
    temp_path.push("note");

    let status = Command::new(editor)
        .arg(&temp_path)
        .status()?;

    if !status.success() {
        return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Editor exited with non-zero status"
        )));
    }

    let content = fs::read_to_string(&temp_path)?;
    let _ = fs::remove_file(&temp_path);
    Ok(content)
}
