use std::env;
use std::fs;
use std::process::Command;

use nostr_sdk::prelude::*;

use nostratui::cli::Flags;
use nostratui::client::NostrClient;
use nostratui::app;
use nostratui::config::Config;


#[tokio::main]
async fn main() -> Result<()> {
    //Get Flags
    let flags = Flags::from_args();

    // Load config
    let (mut config, config_path) = Config::load()?;

    // Update last login time
    let last_login = config.get_last_login();
    config.update_last_login();
    config.save(&config_path)?;

    let mut client = NostrClient::new(config.key.clone());
    client.connect_relays(config.relays.clone()).await?;

    if flags.post() {
        //client post
        let note = edit_string();
        client.post_note(note).await
    } else {
        if config.contacts.is_empty() {
            config.contacts = client.fetch_contacts()
                .await
                .into_iter()
                .map(|pk| pk.to_bech32().unwrap())
                .collect();
        }
        client.set_contacts(config.contacts).await;
        app::start_tui(client, last_login).await
    }

}

fn edit_string() -> String {
    let editor = env::var("EDITOR")
        .unwrap_or_else(|_| "vi".to_string());

    let mut temp_path = env::temp_dir();
    temp_path.push("note");

    Command::new(editor)
        .arg(&temp_path)
        .status()
        .expect("Error: Editor exited with non-zero status");

    let content = fs::read_to_string(&temp_path);
    let _ = fs::remove_file(&temp_path);
    content.expect("blah")
}
