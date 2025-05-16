use nostratui::{
    cli::Flags,
    models::{NostrClient, Config, cache::is_cache_empty},
    controllers::{start_app, init_feed, create_post_via_editor, post_note}
};
use nostr_sdk::Timestamp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get Flags
    let flags = Flags::from_args();

    // Load config
    let mut config = Config::load()?;
    let last_login = config.get_last_login();

    // Initialize client and connect relays
    let mut client = NostrClient::new(config.key.clone()).unwrap();
    client.set_relays(config.relays.clone());
    let _client_connected = client.connect_relays().await?;

    match true {
        _ if flags.post() => {
            // Post a new note
            match create_post_via_editor() {
                Ok(note) => post_note(&client, note).await?,
                Err(e) => eprintln!("Error creating post: {}", e),
            }
        },
        _ if flags.fetch() => {
            // Fetch and update feed
            init_feed(&mut client, &mut config, last_login).await?
        },
        _ => {
            // Start TUI application
            if config.last_login.is_none()  || is_cache_empty().expect("no posts") {
                init_feed(&mut client, &mut config, Timestamp::from_secs(60*60*24*7)).await?;
            }

            client.set_contacts(config.contacts.clone()).await?;
            start_app(client, config).await?;
        }
    }
    
    Ok(())
}
