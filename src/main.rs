use std::env;
use std::fs;
use std::io;
use std::process::Command;

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};


use nostr_sdk::prelude::*;

use nostratui::cli::Flags;
use nostratui::client::NostrClient;
use nostratui::app::{StatefulList,run_app};
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
