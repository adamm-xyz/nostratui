use std::io;
use crate::client::NostrClient;
use nostr_sdk::Timestamp;

use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState},
    layout::{Layout, Constraint, Direction},
    style::{Style, Color, Modifier},
    Terminal, Frame,
    text::Line,
    prelude::{Span,CrosstermBackend},
};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
    event::{self, Event, KeyCode},
    event::{DisableMouseCapture, EnableMouseCapture},
};

pub struct Post {
    pub user: String,
    pub time: u64,
    pub content: String,
}

// Handle TUI setup and teardown
pub async fn start_tui(client: NostrClient, login_date: Timestamp) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Get new posts
    let mut new_posts = client
        .fetch_notes_since(login_date).await?;
    new_posts.sort_by_key(|post| std::cmp::Reverse(post.time));
    
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
        eprintln!("{:?}", err);
    }
    
    Ok(())
}

pub fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    stateful_list: &mut StatefulList<Post>
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, stateful_list))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Esc => return Ok(()),
                KeyCode::Down | KeyCode::Char('j') => stateful_list.next(),
                KeyCode::Up | KeyCode::Char('k') => stateful_list.previous(),
                /*
                KeyCode::Char('n') => {
                    let client_clone = client.clone();
                    tokio::spawn(async move {
                        if let Err(e) = &client_clone.post_note("test".to_string()).await {
                            eprintln!("Error posting: {:?}", e);
                        }
                    });
                },
                KeyCode::Enter => {
                }
                */
                _ => {}
            }
        }
    }
}

fn ui<B: ratatui::backend::Backend>(
    f: &mut Frame<B>,
    stateful_list: &mut StatefulList<Post>,
) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    // Create the feed of posts
    let items: Vec<ListItem> = stateful_list.items
        .iter()
        .map(|post| {
            // Create the header line with username and timestamp
            let header = Line::from(vec![
                Span::styled(
                    format!("{} posted at {}", post.user, post.time),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                )
            ]);
            
            // Create the content as a separate line with proper wrapping
            let content = Line::from(vec![
                Span::raw(&post.content)
            ]);
            
            // Combine them into a multi-line item with spacing
            ListItem::new(vec![
                header,
                Line::from(""), // Empty line for spacing
                content,
                Line::from(""), // Empty line for spacing between posts
            ])
            .style(Style::default())
        })
        .collect();

    // Create a List from the items and highlight the currently selected one
    let list = List::new(items)
        .block(Block::default().title("Feed").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Gray)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        );

    // Render the list with its state
    f.render_stateful_widget(list, chunks[0], &mut stateful_list.state);
}

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
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

