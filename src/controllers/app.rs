use std::env;
use std::fs;
use std::process::Command;
use std::sync::Arc;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::Terminal;
use nostr_sdk::Timestamp;
use std::io;

use crate::models::{NostrClient, Config, Post};
use crate::views::{tui, StatefulList};
use crate::models::cache;
use crate::error::NostratuiError;

pub async fn init_feed(client: &mut NostrClient, config: &mut crate::models::Config, fetch_time: Timestamp) -> Result<(),NostratuiError> {
    // Get contacts
    let conf_contacts = config.contacts.clone();
    client.set_contacts(config.contacts.clone()).await?;
    if conf_contacts.is_empty() {
        config.contacts = client.get_contacts()
            .into_iter()
            .map(|c| c.to_string_tuple())
            .collect();
    }

    // Get posts to read, add to cache
    let new_posts = client.fetch_notes_since(fetch_time).await?;
    crate::models::cache::save_posts_to_cache(new_posts)?;

    // Save new config
    config.update_last_login();
    config.save()?;
    Ok(())
}

pub async fn start_app(client: NostrClient, config: Config) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    let mut terminal = tui::setup_terminal()?;

    // Get new posts
    let mut posts = cache::load_cached_posts()
        .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
    posts.sort_by_key(|post| std::cmp::Reverse(post.timestamp));
    
    // Create our stateful list
    let mut stateful_list = StatefulList::with_items(posts);

    let arc_client = Arc::new(client);

    // Run the app
    let res = run_app(&mut terminal, &mut stateful_list, Arc::clone(&arc_client), config).await;

    // Restore terminal
    tui::restore_terminal(&mut terminal)?;
    
    if let Err(err) = res {
        eprintln!("{:?}", err);
    }
    
    Ok(())
}

pub async fn run_app(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    stateful_list: &mut StatefulList<Post>,
    client: Arc<NostrClient>,
    config: Config,
) -> Result<(),NostratuiError> {
    let mut refresh_in_progress = false;
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<Post>>(1);

    loop {
        let status_message = if refresh_in_progress {
            String::from("Refreshing...")
        } else {
            String::from("Feed")
        };

        terminal.draw(|f| tui::render_ui(f, stateful_list, status_message ))?;

        if let Ok(new_posts) = rx.try_recv() {
            cache::save_posts_to_cache(new_posts.clone())?;
            stateful_list.add_items(new_posts);
            stateful_list.items.sort_by_key(|post| std::cmp::Reverse(post.timestamp));
            refresh_in_progress = false;
        }
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => stateful_list.next(),
                    KeyCode::Up | KeyCode::Char('k') => stateful_list.previous(),
                    KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => stateful_list.jump_down(10),
                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => stateful_list.jump_up(10),
                    KeyCode::Char('g') => stateful_list.first(),
                    KeyCode::Char('G') => stateful_list.last(),
                    KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if !refresh_in_progress {
                            refresh_in_progress = true;

                            let last_login = config.get_last_login();
                            let task_client = Arc::clone(&client);
                            let task_tx = tx.clone();

                            tokio::spawn(async move {
                                match fetch_new_posts(&task_client, last_login).await {
                                    Ok(new_posts) => {
                                        let _ = task_tx.send(new_posts).await;
                                    },
                                    Err(e) => {
                                        eprintln!("Error fetching notes: {:?}", e);
                                    }
                                }
                            });
                        }
                    },
                    KeyCode::Char('n') => {
                        // Use the helper function to handle terminal restoration and setup
                        if let Ok(()) = tui::with_restored_terminal(terminal, || {
                            // Create and post the note
                            match create_post_via_editor() {
                                Ok(note) => {
                                    let client_clone = Arc::clone(&client);
                                    tokio::spawn(async move {
                                        if let Err(e) = post_note(&client_clone, note).await {
                                            eprintln!("Error posting note: {:?}", e);
                                        }
                                    });
                                },
                                Err(e) => eprintln!("Error creating post: {:?}", e),
                            }
                        }) {
                            // Terminal was successfully restored and re-setup
                        }
                    },
                    /*
                       KeyCode::Enter => {
                    // View post details functionality can be implemented here
                    }
                    */
                    _ => {}
                }
            }
        }

    }
}

pub async fn fetch_new_posts(client: &Arc<NostrClient>, last_login: Timestamp) -> Result<Vec<Post>, NostratuiError> {
    client.fetch_notes_since(last_login).await
}

pub async fn post_note(client: &NostrClient, content: String) -> Result<(), NostratuiError> {
    client.post_note(content).await
}

pub fn create_post_via_editor() -> Result<String,NostratuiError> {
    let editor = env::var("EDITOR")
        .unwrap_or_else(|_| "vi".to_string());

    let mut temp_path = env::temp_dir();
    temp_path.push("note");

    let status = Command::new(editor)
        .arg(&temp_path)
        .status()?;

    if !status.success() {
        return Err(NostratuiError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Editor exited with non-zero status"
                ).to_string()
        ).into());
    }

    let content = fs::read_to_string(&temp_path)?;
    let _ = fs::remove_file(&temp_path);
    Ok(content)
}

