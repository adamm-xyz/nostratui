use std::io;
use ratatui::{
    widgets::{Block, Borders, List, ListItem},
    layout::{Layout, Constraint, Direction},
    style::{Style, Color, Modifier},
    Terminal, Frame,
    text::Line,
    prelude::{Span},
};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
    event::{DisableMouseCapture, EnableMouseCapture},
};

use crate::models::Post;
use crate::views::widgets::StatefulList;

pub fn setup_terminal() -> io::Result<Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn restore_terminal(terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()
}

pub fn render_ui<B: ratatui::backend::Backend>(
    f: &mut Frame<B>,
    stateful_list: &mut StatefulList<Post>,
    status: String,
) {
        // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    // Calculate the available width for text wrapping
    let available_width = chunks[0].width.saturating_sub(4); // Subtract border width and some padding

    // Create the feed of posts
    let items: Vec<ListItem> = stateful_list.items
        .iter()
        .map(|post| {
            // Create the header line with username and timestamp
            let header = Line::from(vec![
                Span::styled(
                    format!("{} posted at {}", post.user, post.datetime),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                )
            ]);
            
            // Create wrapped content by manually splitting the text
            let content_lines = wrap_text(&post.content, available_width as usize);
            
            // Combine them into a multi-line item with spacing
            let mut all_lines = vec![
                header,
                Line::from(""), // Empty line for spacing
            ];
            
            // Add each wrapped line as a separate Line
            for line in content_lines {
                all_lines.push(Line::from(line));
            }
            all_lines.push(Line::from("")); // Empty line for spacing at the end

            ListItem::new(all_lines)
                .style(Style::default())
        })
        .collect();

    // Create a List from the items and highlight the currently selected one
    let list = List::new(items)
        .block(Block::default().title(status).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Gray)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        );

    // Render the list with its state
    f.render_stateful_widget(list, chunks[0], &mut stateful_list.state);
}

// Helper function to manually wrap text to a specified width
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut wrapped_lines = Vec::new();
    
    for line in text.lines() {
        // Skip empty lines
        if line.trim().is_empty() {
            wrapped_lines.push(String::new());
            continue;
        }
        
        // Convert to character indices to handle UTF-8 properly
        let chars: Vec<char> = line.chars().collect();
        
        if chars.len() <= width {
            wrapped_lines.push(line.to_string());
            continue;
        }
        
        let mut start = 0;
        while start < chars.len() {
            let end = if start + width >= chars.len() {
                chars.len()
            } else {
                // Try to find a good break point
                let mut break_point = start + width;
                while break_point > start && !chars[break_point - 1].is_whitespace() {
                    break_point -= 1;
                }
                
                // If no good break found, just cut at width
                if break_point == start {
                    start + width
                } else {
                    break_point
                }
            };
            
            let chunk: String = chars[start..end].iter().collect();
            wrapped_lines.push(chunk.trim().to_string());
            
            // Move past any whitespace
            start = end;
            while start < chars.len() && chars[start].is_whitespace() {
                start += 1;
            }
        }
    }
    
    wrapped_lines
}
