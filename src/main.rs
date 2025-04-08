use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::env;
use std::fs;
use std::io::{Write,Read};
use std::process::Command;

use nostr_sdk::prelude::*;


#[tokio::main]
async fn main() -> Result<()> {
    //Generate keys and construct client
    let keys: Keys = Keys::generate();
    // let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;
    let client = Client::new(keys);

    // Add and connect to relays
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    println!("Connected to relay!");

    println!("Attempting to publish note...");
    //Publish a note
    let note = edit_string();
    let builder = EventBuilder::text_note(note).pow(20);
    let output = client.send_event_builder(builder).await?;

    //Inspect output
    println!("Event ID: {}", output.id().to_bech32()?);
    println!("Sent to: {:?}", output.success);
    println!("Not sent to: {:?}", output.failed);

    Ok(())
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
    fs::remove_file(&temp_path);
    content.expect("blah")
}
