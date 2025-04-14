use std::env;
use std::fs::File;
use std::fs;
use std::io::BufReader;
use std::process::Command;


use serde::{Deserialize, Serialize};

use nostr_sdk::prelude::*;

use nostratui::cli::Flags;
use nostratui::client::NostrClient;

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

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        // Start with the first item selected
        if !items.is_empty() {
            state.select(Some(0));
        }
        StatefulList {
            state,
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    i
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

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
        let new_posts = client.fetch_notes_since(
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
        let res = run_app(&mut terminal, &mut stateful_list, client);

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

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    stateful_list: &mut StatefulList<String>,
    client: NostrClient,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, stateful_list))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Esc => return Ok(()),
                KeyCode::Down | KeyCode::Char('j') => stateful_list.next(),
                KeyCode::Up | KeyCode::Char('k') => stateful_list.previous(),
                KeyCode::Char('n') => {
                    let client_clone = client.clone();
                    tokio::spawn(async move {
                        if let Err(e) = &client_clone.post_note("test".to_string()).await {
                            eprintln!("Error posting: {:?}", e);
                        }
                    });
                },
                KeyCode::Enter => {
                    // Here you could handle what happens when an item is selected
                    // For now, we'll just continue the loop
                }
                _ => {}
            }
        }
    }
}

fn ui<B: ratatui::backend::Backend>(
    f: &mut Frame<B>,
    stateful_list: &mut StatefulList<String>,
) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    // Create the list items
    let items: Vec<ListItem> = stateful_list.items
        .iter()
        .map(|i| {
            ListItem::new(i.as_str())
                .style(Style::default())
        })
        .collect();

    // Create a List from the items and highlight the currently selected one
    let list = List::new(items)
        .block(Block::default().title("posts").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Gray)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("> ");

    // Render the list with its state
    f.render_stateful_widget(list, chunks[0], &mut stateful_list.state);
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
