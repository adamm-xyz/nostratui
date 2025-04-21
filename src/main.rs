use std::env;
use std::fs;
use std::process::Command;
use std::path::PathBuf;

use nostr_sdk::prelude::*;

use nostratui::cli::Flags;
use nostratui::client::NostrClient;
use nostratui::app;
use nostratui::config::Config;
use nostratui::cache;

use std::time::Duration;


#[tokio::main]
async fn main() -> Result<()> {
    //Get Flags
    let flags = Flags::from_args();

    // Load config
    let (config, config_path) = Config::load()?;

    // Initialize client and connect relays
    let client = NostrClient::new(config.key.clone());
    client.connect_relays(config.relays.clone()).await?;

    match true {
        _ if flags.post() => {
            //client post
            match edit_string() {
                Ok(note) => client.post_note(note).await?,
                Err(e) => eprintln!("Error creating post: {}", e),
            }
        },
        _ if flags.fetch() => {
            get_feed(client,config,config_path).await?
        },
        _ => {
            if config.last_login.is_none() {
                get_feed(client,config,config_path).await?;
            }

            app::start_tui().expect("UI crashed")
        }
    }
    Ok(())
}

async fn get_feed(mut client: NostrClient, mut config: Config, config_path: PathBuf) -> Result<()> {
    // Get contacts
    let mut contact_list = vec![];
    if config.contacts.is_empty() {
        println!("contacts empty!");
        let fetched_contacts = client.fetch_contacts().await;
        for contact in fetched_contacts {
            contact_list.push(contact.to_string_tuple());
        }
    }
    config.contacts = contact_list.clone();
    client.set_contacts(contact_list).await?;


    // Get last login time
    let last_login = config.get_last_login();

    // Get posts to read, add to cache
    let new_posts = client.fetch_notes_since(last_login).await?;
    cache::save_posts_to_cache(new_posts);

    // Save new config
    config.update_last_login();
    config.save(&config_path)?;
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
