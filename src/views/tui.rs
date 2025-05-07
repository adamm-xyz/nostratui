use std::io;
use ratatui::{
    widgets::{Block, Borders, List, ListItem},
    layout::{Layout, Constraint, Direction},
    style::{Style, Color, Modifier},
    Terminal, Frame,
    text::Line,
    prelude::{Span, Text},
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
            
            // Create the content as a separate line with proper wrapping
            let content = Text::raw(&post.content);
            
            // Combine them into a multi-line item with spacing
            let mut all_lines = vec![
                header,
                Line::from(""), // Empty line for spacing
            ];
            all_lines.extend(content.lines);
            all_lines.push(Line::from(""));

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
