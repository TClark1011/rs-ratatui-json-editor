use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, CurrentScreen, CurrentlyEditing};

pub fn ui(frame: &mut Frame, app: &App) {
    let vertical_panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "Create New Json",
        Style::default().fg(Color::Green),
    ))
    .block(title_block);

    frame.render_widget(title, vertical_panels[0]); // render title to top panel

    let (current_screen_navigation_str, current_screen_navigation_fg) = match app.current_screen {
        CurrentScreen::Main => ("Normal Mode", Color::Green),
        CurrentScreen::Editing => ("Editing Mode", Color::Yellow),
        CurrentScreen::Exiting => ("Exiting", Color::LightRed),
    };

    let current_screen_navigation_text = Span::styled(
        format!(" {current_screen_navigation_str}"),
        Style::default().fg(current_screen_navigation_fg),
    )
    .to_owned();

    let (currently_editing_navigation_str, currently_editing_navigation_fg) =
        match app.currently_editing {
            Some(CurrentlyEditing::Value) => ("Editing Json Value", Color::Green),
            Some(CurrentlyEditing::Key) => ("Editing Json Key", Color::LightGreen),
            None => ("Not Editing Anything", Color::DarkGray),
        };
    let currently_editing_navigation_text = Span::styled(
        currently_editing_navigation_str,
        Style::default().fg(currently_editing_navigation_fg),
    );

    let all_navigation_text = vec![
        current_screen_navigation_text,
        Span::styled(" | ", Style::default().fg(Color::White)),
        currently_editing_navigation_text,
    ];

    let active_mode_footer = Paragraph::new(Line::from(all_navigation_text))
        .block(Block::default().borders(Borders::ALL));

    let current_keys_hint = Span::styled(
        format!(
            " {}",
            match app.current_screen {
                CurrentScreen::Main => "(q) quit / (e) new pair",
                CurrentScreen::Editing => "(ESC) cancel / (Tab) switch / (Enter) submit",
                CurrentScreen::Exiting => "(y) confirm / (n) cancel",
            }
        ),
        Style::default().fg(Color::Red),
    );

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL));

    let footer_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical_panels[2]);

    frame.render_widget(active_mode_footer, footer_panels[0]);
    frame.render_widget(key_notes_footer, footer_panels[1]);
}

fn popup(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // divide the layout vertically into 3 pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // divide the center vertical piece into 3 horizontal pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // return the center piece
}
