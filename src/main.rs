use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, BufReader};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
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
    relays: Vec<String>,
    contacts: Vec<String>,
    last_login: Option<u64>,
}

pub fn get_last_login(config: &Config) -> Timestamp {
    match config.last_login {
        //If config has last login saved
        Some(login_date) => {
            Timestamp::from_secs(login_date)
        }
        //If the config does not have login date
        None => {
            Timestamp::from_secs(60*60*24*7)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    //Get Flags
    let flags = Flags::from_args();

   let config_path = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".config/nostratui/config.json"); 

    // Open the file
    let file = File::open(&config_path)?;
    let reader = BufReader::new(file);
    
    // Parse the JSON
    let mut config: Config = serde_json::from_reader(reader)?;
    
    // Extract values into variables
    let user_key = config.key.clone();
    let relays = config.relays.clone();
    let last_login = get_last_login(&config);

    //Save last login date as now and write it to config file
    let timestamp_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time traveler?")
        .as_secs() as u64;

    config.last_login = Some(timestamp_now);
    let json = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create(&config_path)?;
    file.write_all(json.as_bytes())?;

    let mut client = NostrClient::new(user_key);
    client.connect_relays(relays).await?;

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

            // Create/open the file for writing
            let file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&config_path)?;
            
            // Serialize the updated config to JSON and write it to the file
            serde_json::to_writer_pretty(file, &config)?;
        }
        client.set_contacts(config.contacts).await;
        start_tui(client, last_login).await
    }

}

async fn start_tui(client: NostrClient, login_date: Timestamp) -> Result<()> {
    // Get new posts
    let mut new_posts = client
        .fetch_notes_since(login_date).await?;
    new_posts.sort_by_key(|post| std::cmp::Reverse(post.time));
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
