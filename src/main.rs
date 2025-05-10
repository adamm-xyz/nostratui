use nostratui::{
    cli::Flags,
    models::{NostrClient, Config},
    controllers::{start_app, init_feed, create_post_via_editor, post_note}
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get Flags
    let flags = Flags::from_args();

    // Load config
    let mut config = Config::load()?;

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
            init_feed(&mut client, &mut config).await?
        },
        _ => {
            // Start TUI application
            if config.last_login.is_none() {
                init_feed(&mut client, &mut config).await?;
            }

            client.set_contacts(config.contacts.clone()).await?;
            start_app(client, config).await?;
        }
    }
    
    Ok(())
}
