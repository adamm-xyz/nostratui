use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;
use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;

use crate::models::{NostrClient, Config, Post};
use crate::views::{tui, StatefulList};
use crate::controllers::post_controller;
use crate::models::cache;

pub async fn init_feed(client: &mut NostrClient, config: &mut crate::models::Config) -> Result<()> {
    // Get contacts
    let conf_contacts = config.contacts.clone();
    client.set_contacts(config.contacts.clone()).await?;
    if conf_contacts.is_empty() {
        config.contacts = client.get_contacts()
            .into_iter()
            .map(|c| c.to_string_tuple())
            .collect();
    }

    // Get last login time
    let last_login = config.get_last_login();

    // Get posts to read, add to cache
    let new_posts = client.fetch_notes_since(last_login).await?;
    crate::models::cache::save_posts_to_cache(new_posts);

    // Save new config
    config.update_last_login();
    config.save()?;
    Ok(())
}

pub async fn start_app(client: NostrClient, config: Config) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    let mut terminal = tui::setup_terminal()?;

    // Get new posts
    let mut posts = cache::load_cached_posts();
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

pub async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    stateful_list: &mut StatefulList<Post>,
    client: Arc<NostrClient>,
    config: Config,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| tui::render_ui(f, stateful_list, String::from("Feed")))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Esc => return Ok(()),
                KeyCode::Down | KeyCode::Char('j') => stateful_list.next(),
                KeyCode::Up | KeyCode::Char('k') => stateful_list.previous(),
                KeyCode::Char('r') => {
                    let last_login = config.get_last_login();
                    terminal.draw(|f| tui::render_ui(f, stateful_list, String::from("Refreshing...")))?;
                    
                    match post_controller::fetch_new_posts(&client, last_login).await {
                        Err(e) => eprintln!("Error fetching notes: {:?}", e),
                        Ok(new_posts) => {
                            cache::save_posts_to_cache(new_posts.clone());
                            stateful_list.add_items(new_posts);
                            stateful_list.items.sort_by_key(|post| std::cmp::Reverse(post.timestamp));
                        }
                    }
                },
                /*
                KeyCode::Char('n') => {
                    // Post note functionality can be implemented here using post_controller
                },
                KeyCode::Enter => {
                    // View post details functionality can be implemented here
                }
                */
                _ => {}
            }
        }
    }
}
