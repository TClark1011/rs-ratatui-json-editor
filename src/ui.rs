use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppScreen, Binding, CurrentlyEditing, JsonValue, JsonValueType};

const COLOR_ACCENT: Color = Color::LightYellow;
const COLOR_SURFACE: Color = Color::DarkGray;

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
        match app.get_current_screen() {
            AppScreen::Preview => "Preview",
            _ => "JSON Editor",
        },
        Style::default().fg(Color::Green),
    ))
    .block(title_block);

    frame.render_widget(title, vertical_panels[0]); // render title to top panel

    //# Footer
    let current_keys_hint = Span::styled(
        format!(
            " {}",
            app.available_bindings
                .iter()
                .filter_map(|(binding, action)| {
                    let key_label = match binding {
                        Binding::Static(KeyCode::Enter) => "Enter",
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

    frame.render_widget(key_notes_footer, vertical_panels[2]);

    //# Existing Pairs List
    let mut list_items = Vec::<ListItem>::new();

    for key in app.pairs.keys() {
        list_items.push(ListItem::new(Line::from(Span::styled(
            format!(
                "\"{: <25}: {}",
                format!("{key}\""),
                match app.pairs.get(key) {
                    Some(value) => match value {
                        JsonValue::String(s) => format!("\"{}\"", s),
                        JsonValue::Boolean(b) => format!("{}", b),
                        JsonValue::Number(n) => format!("{}", n),
                        JsonValue::Null => "null".to_string(),
                    },
                    None => "null".to_string(),
                }
            ),
            Style::default().fg(COLOR_ACCENT),
        ))))
    }

    let pairs_block = Block::default().padding(Padding::horizontal(1));
    frame.render_stateful_widget(
        List::new(list_items)
            .block(pairs_block)
            .highlight_style(Style::default().bg(COLOR_ACCENT).fg(Color::Black)),
        vertical_panels[1],
        &mut app.list_ui_state,
    );

    //# Delete Confirm Popup
    if let Some(target_delete_key) = &app.target_delete_key {
        let popup_block = Block::default()
            .title(" Delete?")
            .borders(Borders::NONE)
            .style(Style::default().bg(COLOR_SURFACE));

        let area = centered_rect(30, 30, frame.area());

        let panels = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Fill(1)])
            .margin(1)
            .split(area);

        let [control_hint_panel] = Layout::vertical([Constraint::Length(1)])
            .flex(Flex::Center)
            .areas(panels[1]);

        let message_paragraph = Paragraph::new(format!(
            "Are you sure you want to delete the key: \"{target_delete_key}\"?"
        ));

        let control_hint_text = Paragraph::new("(y/n)").centered();

        frame.render_widget(popup_block, area);
        frame.render_widget(message_paragraph, panels[0]);
        frame.render_widget(control_hint_text, control_hint_panel);
    }

    if let Some(editing) = &app.currently_editing {
        if !app.type_list_open {
            //# Editing Popup
            let popup_block = Block::default()
                .title("Enter a new key-value pair")
                .borders(Borders::NONE)
                .style(Style::default().bg(COLOR_SURFACE));

            let area = centered_rect(60, 50, frame.area());

            let popup_vertical_panels = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100), Constraint::Length(3)])
                .margin(1)
                .split(area);
            let popup_panels = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(popup_vertical_panels[0]);

            let mut key_block = Block::default().title("Key").borders(Borders::ALL);
            let mut value_block = Block::default().title("Value").borders(Borders::ALL);
            let mut type_block = Block::default().title("Type").borders(Borders::ALL);

            let active_style = Style::default().bg(COLOR_ACCENT).fg(Color::Black);

            match editing {
                CurrentlyEditing::Key => key_block = key_block.style(active_style),
                CurrentlyEditing::Value => value_block = value_block.style(active_style),
                CurrentlyEditing::Type => type_block = type_block.style(active_style),
            }

            frame.render_widget(popup_block, area);

            let key_text = Paragraph::new(app.key_input.clone()).block(key_block);
            frame.render_widget(key_text, popup_panels[0]);

            let value_text = Paragraph::new(app.value_input.clone()).block(value_block);
            frame.render_widget(value_text, popup_panels[1]);

            let type_text = Paragraph::new(match app.selected_value_type {
                JsonValueType::String => "String",
                JsonValueType::Boolean => "Boolean",
                JsonValueType::Number => "Number",
                JsonValueType::Null => "null",
            })
            .block(type_block);
            frame.render_widget(type_text, popup_vertical_panels[1]);
        } else {
            // # Editing Type Selection Popup
            let type_popup_block = Block::default()
                .title("Select type of new value")
                .borders(Borders::NONE)
                .style(Style::default().bg(COLOR_SURFACE));

            let type_popup_area = centered_rect(60, 30, frame.area());

            let type_popup_panels = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)])
                .margin(1)
                .split(type_popup_area);

            let type_list_ui = List::new(App::get_value_type_vec().iter().map(|value_type| {
                Line::from(Span::styled(
                    format!(" {value_type} "),
                    Style::default().fg(COLOR_ACCENT),
                ))
            }))
            .highlight_style(Style::default().bg(COLOR_ACCENT).fg(COLOR_SURFACE));

            frame.render_widget(type_popup_block, type_popup_area);
            frame.render_stateful_widget(
                type_list_ui,
                type_popup_panels[0],
                &mut app.type_list_ui_state,
            );
        }
    }

    //# Preview Screen
    if let AppScreen::Preview = app.get_current_screen() {
        let serialized = serde_json::to_string_pretty(&app.pairs).unwrap();

        let text = Paragraph::new(serialized);

        frame.render_widget(Clear, vertical_panels[1]);
        frame.render_widget(text, vertical_panels[1]);
    }

    //# Exit Popup
    if let AppScreen::Exiting = app.get_current_screen() {
        frame.render_widget(Clear, frame.area()); //this clears the entire screen and anything already drawn

        let popup_block = Block::default()
            .title(" Save?")
            .style(Style::default().bg(COLOR_SURFACE));

        let exit_text = Text::styled(
            " Would you like to save your changes before exiting? (y/n)",
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
