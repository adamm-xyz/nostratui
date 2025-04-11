use std::env;
use std::fs;
use std::process::Command;
use std::time::Duration;

use nostr_sdk::prelude::*;

use nostratui::cli::Flags;

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

#[tokio::main]
async fn main() -> Result<()> {
    //Generate keys and construct client
    let env_key = env::var("NOSTR_KEY").unwrap();
    let keys = Keys::parse(&env_key)?;
    let my_pub_key = keys.public_key();

    let client = Client::new(keys.clone());

    println!("Connecting to relays...");

    // Add and connect to relays
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.wine").await?;
    client.add_relay("wss://relay.rip").await?;
    client.connect().await;
    println!("Connected!");


    println!("Getting followers...");
    // Get followers
    let filter = Filter::new().author(my_pub_key).kind(Kind::ContactList);
    let events = client.fetch_events(filter, Duration::from_secs(10)).await?;
    let mut followers = vec![];
    if let Some(event) = events.first() {
        let tags = &event.tags;
        
        for tag in tags.iter() {
            if let Some(follower) = tag.content() {
                let follower_pkey = PublicKey::from_hex(follower).unwrap();
                followers.push(follower_pkey);
            }
        }
    }
    println!("Follow list populated!");

    println!("Fetching notes from past 24 hours...");
    let mut new_posts: Vec<String> = vec![];
    for follower in followers {
        let filter = Filter::new().author(follower).kind(Kind::TextNote).since(Timestamp::now() - Timestamp::from_secs(60*60*24));
        let events = client.fetch_events(filter, Duration::from_secs(30)).await?;
        for event in events {
            let content = &event.content;
            new_posts.push(content.to_string());
        }
    }
    println!("{} new posts", new_posts.len());
    println!("Fetched!");
    println!("Starting ratatui!");


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

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    stateful_list: &mut StatefulList<String>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, stateful_list))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Esc => return Ok(()),
                KeyCode::Down | KeyCode::Char('j') => stateful_list.next(),
                KeyCode::Up | KeyCode::Char('k') => stateful_list.previous(),
                //KeyCode::Char('n') => #create post,
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

async fn post(client: Client) -> Result<()> {
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
    let _ = fs::remove_file(&temp_path);
    content.expect("blah")
}
