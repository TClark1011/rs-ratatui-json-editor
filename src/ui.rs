use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppScreen, CurrentlyEditing};

pub fn ui(frame: &mut Frame, app: &mut App) {
    let vertical_panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    //# Header
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "Create New Json",
        Style::default().fg(Color::Green),
    ))
    .block(title_block);

    frame.render_widget(title, vertical_panels[0]); // render title to top panel

    //# Footer
    let (current_screen_navigation_str, current_screen_navigation_fg) =
        match app.get_current_screen() {
            AppScreen::Main => ("Normal Mode", Color::Green),
            AppScreen::Editing => ("Editing Mode", Color::Yellow),
            AppScreen::Exiting => ("Exiting", Color::LightRed),
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
            app.available_bindings
                .iter()
                .filter_map(|(key_code, action)| {
                    let key_label = match key_code {
                        KeyCode::Enter => "Enter",
                        kc => &format!("{kc}"),
                    };

                    return Some(format!("({}) {}", key_label, action.description()?));
                })
                .collect::<Vec<_>>()
                .join(" | ")
        ),
        Style::default().fg(Color::Blue),
    );

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL));

    let footer_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical_panels[2]);

    frame.render_widget(active_mode_footer, footer_panels[0]);
    frame.render_widget(key_notes_footer, footer_panels[1]);

    let mut list_items = Vec::<ListItem>::new();

    //# Existing Pairs List
    for key in app.pairs.keys() {
        list_items.push(ListItem::new(Line::from(Span::styled(
            format!(
                "\"{: <25}: \"{}\"",
                format!("{key}\""),
                app.pairs.get(key).unwrap()
            ),
            Style::default().fg(Color::Yellow),
        ))))
    }
    frame.render_stateful_widget(
        List::new(list_items).highlight_style(Style::default().bg(Color::White).fg(Color::Black)),
        vertical_panels[1],
        &mut app.list_ui_state,
    );

    //# Editing Popup
    if let Some(editing) = &app.currently_editing {
        let popup_block = Block::default()
            .title("Enter a new key-value pair")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let area = centered_rect(60, 25, frame.area());

        let popup_panels = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let mut key_block = Block::default().title("Key").borders(Borders::ALL);
        let mut value_block = Block::default().title("Value").borders(Borders::ALL);

        let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);

        match editing {
            CurrentlyEditing::Key => key_block = key_block.style(active_style),
            CurrentlyEditing::Value => value_block = value_block.style(active_style),
        }

        frame.render_widget(popup_block, area);

        let key_text = Paragraph::new(app.key_input.clone()).block(key_block);
        frame.render_widget(key_text, popup_panels[0]);

        let value_text = Paragraph::new(app.value_input.clone()).block(value_block);
        frame.render_widget(value_text, popup_panels[1]);
    }

    //# Exit Popup
    if let AppScreen::Exiting = app.get_current_screen() {
        frame.render_widget(Clear, frame.area()); //this clears the entire screen and anything already drawn

        let popup_block = Block::default()
            .title("Y/N")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let exit_text = Text::styled(
            "Would you like to output the buffer as json? (y/n)",
            Style::default().fg(Color::Red),
        );
        // the `trim: false` will stop the text from being cut off when over the edge of the block
        let exit_paragraph = Paragraph::new(exit_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = centered_rect(60, 25, frame.area());
        frame.render_widget(exit_paragraph, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
