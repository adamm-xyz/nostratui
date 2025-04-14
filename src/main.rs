use std::env;
use std::fs::File;
use std::fs;
use std::io::BufReader;
use std::process::Command;
use std::io;
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, ListState},
    layout::{Layout, Constraint, Direction},
    style::{Style, Color, Modifier},
    Terminal, Frame,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use serde::{Deserialize, Serialize};

use nostr_sdk::prelude::*;

use nostratui::cli::Flags;
use nostratui::client::NostrClient;
use nostratui::app::{StatefulList,run_app};


#[derive(Serialize, Deserialize, Debug)]
struct Config {
    key: String,
    relays: Vec<String>
}

#[tokio::main]
async fn main() -> Result<()> {
    //Get Flags
    let flags = Flags::from_args();

   let config_path = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".config/nostratui/config.json"); 

    // Open the file
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    
    // Parse the JSON
    let config: Config = serde_json::from_reader(reader)?;
    
    // Extract values into variables
    let user_key = config.key;
    let relays = config.relays;

    let client = NostrClient::new(user_key);
    client.connect_relays(relays).await?;

    if flags.post() {
        //client post
        let note = edit_string();
        client.post_note(note).await
    } else {
        // Get following
        client.fetch_following().await;
        // Get new posts
        let new_posts = client
            .fetch_notes_since(
                Timestamp::from_secs(60*60*24)).await?;

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Create our stateful list
        let mut stateful_list = StatefulList::with_items(new_posts);

        // Run the app
        let res = run_app(&mut terminal, &mut stateful_list);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        if let Err(err) = res {
            println!("{:?}", err);
        }
        /*

        */
        Ok(())
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
