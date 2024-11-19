use std::io;

use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

use crate::app::{
    App, AppScreen, Binding, EditFocus, ExitFocus, JsonData, JsonValue, JsonValueType, TextField,
};

const COLOR_ACCENT: Color = Color::LightYellow;
const COLOR_SURFACE: Color = Color::DarkGray;

pub fn ui(frame: &mut Frame, app: &mut App) -> Result<(), io::Error> {
    let vertical_panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let header = compose_header(app);
    frame.render_widget(header, vertical_panels[0]); // render title to top panel

    let footer = compose_footer(app);
    frame.render_widget(footer, vertical_panels[2]);

    let pairs_list = compose_pairs_list(&app.pairs);
    frame.render_stateful_widget(pairs_list, vertical_panels[1], &mut app.list_ui_state);

    if let Some(target_delete_key) = &app.target_delete_key {
        render_delete_confirm_popup(frame, target_delete_key);
    }

    if app.edit_popup_focus.is_some() {
        if !app.type_list_open {
            render_editing_popup(frame, app)?;
        } else {
            render_type_selection_popup(frame, app);
        }
    }

    match app.get_current_screen() {
        AppScreen::Preview => {
            let preview = compose_preview_screen(app)?;

            frame.render_widget(Clear, vertical_panels[1]);
            frame.render_widget(preview, vertical_panels[1]);
        }
        AppScreen::Exiting => {
            frame.render_widget(Clear, frame.area()); //this clears the entire screen and anything already drawn
            render_exit_popup(frame, app);
        }
        _ => {}
    }

    return Ok(());
}

fn compose_header(app: &App) -> Paragraph {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    Paragraph::new(Text::styled(
        match app.get_current_screen() {
            AppScreen::Preview => "Preview",
            _ => "JSON Editor",
        },
        Style::default().fg(Color::Green),
    ))
    .block(title_block)
}

fn compose_footer(app: &App) -> Paragraph {
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

    Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL))
}

fn compose_pairs_list(pairs: &JsonData) -> List {
    let mut list_items = Vec::<ListItem>::new();

    for key in pairs.keys() {
        list_items.push(ListItem::new(Line::from(Span::styled(
            format!(
                "\"{: <25}: {}",
                format!("{key}\""),
                match pairs.get(key) {
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

    List::new(list_items)
        .block(pairs_block)
        .highlight_style(Style::default().bg(COLOR_ACCENT).fg(Color::Black))
}

fn render_delete_confirm_popup(frame: &mut Frame, target_delete_key: &str) {
    let popup_block = Block::default()
        .title(" Delete?")
        .borders(Borders::NONE)
        .style(Style::default().bg(COLOR_SURFACE));

    let area = compose_popup(
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        frame.area(),
    );

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

fn render_editing_popup(frame: &mut Frame, app: &App) -> Result<(), io::Error> {
    let popup_block = Block::default()
        .title(" Enter a new key-value pair")
        .borders(Borders::NONE)
        .style(Style::default().bg(COLOR_SURFACE));

    let area = compose_popup(Constraint::Length(64), Constraint::Length(8), frame.area());

    let popup_vertical_panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(3)])
        .margin(1)
        .split(area);
    let popup_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(popup_vertical_panels[0]);

    let mut key_style = Style::default();
    let mut value_style = Style::default();
    let mut type_style = Style::default();
    match app.edit_popup_focus {
        Some(EditFocus::Key) => key_style = key_style.bg(COLOR_ACCENT).fg(Color::Black),
        Some(EditFocus::Value) => value_style = value_style.bg(COLOR_ACCENT).fg(Color::Black),
        Some(EditFocus::Type) => type_style = type_style.bg(COLOR_ACCENT).fg(Color::Black),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No editing mode selected",
            ));
        }
    }

    for field in app.error_fields.iter() {
        match field {
            TextField::Key => key_style = key_style.fg(Color::Red),
            TextField::Value => value_style = value_style.fg(Color::Red),
            _ => {}
        }
    }

    let key_block = Block::default()
        .title("Key")
        .borders(Borders::ALL)
        .style(key_style);
    let value_block = Block::default()
        .title("Value")
        .borders(Borders::ALL)
        .style(value_style);
    let type_block = Block::default()
        .title("Type")
        .borders(Borders::ALL)
        .style(type_style);

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

    return Ok(());
}

fn render_type_selection_popup(frame: &mut Frame, app: &mut App) {
    let value_types = App::all_value_types();

    let title = " Select type of new value";

    let type_popup_block = Block::default()
        .title(title)
        .borders(Borders::NONE)
        .style(Style::default().bg(COLOR_SURFACE));

    let type_popup_area = compose_popup(
        Constraint::Length(title.len() as u16 + 8),
        Constraint::Length(value_types.len() as u16 + 2),
        frame.area(),
    );

    let type_popup_panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .margin(1)
        .split(type_popup_area);

    let type_list_ui = List::new(value_types.iter().map(|value_type| {
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

fn compose_preview_screen(app: &App) -> Result<Paragraph, io::Error> {
    match serde_json::to_string_pretty(&app.pairs) {
        Ok(serialized) => {
            let text = Paragraph::new(serialized);
            return Ok(text);
        }
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize JSON data: {}", e),
            ));
        }
    }
}

fn render_exit_popup(frame: &mut Frame, app: &App) {
    let popup_block = Block::default().style(Style::default().bg(COLOR_SURFACE));

    let row_heights = [1, 3, 1];
    let total_height = row_heights.iter().sum::<u16>();

    let area = compose_popup(
        Constraint::Length(60),
        Constraint::Length(total_height),
        frame.area(),
    );

    let vertical_panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            row_heights
                .iter()
                .map(|h| Constraint::Length(*h))
                .collect::<Vec<_>>(),
        )
        .split(area);

    let exit_text = Text::styled(
        " Would you like to save your changes before exiting?",
        Style::default(),
    );

    let mut positive_button = Block::default();
    let mut negative_button = Block::default();

    let active_style = Style::default().bg(COLOR_ACCENT).fg(Color::Black);

    let mut input_style = Style::default();

    match app.exit_popup_focus {
        Some(ExitFocus::Input) => {
            input_style = input_style.bg(COLOR_ACCENT).fg(Color::Black);
        }
        Some(ExitFocus::Positive) => {
            positive_button = positive_button.style(active_style);
        }
        Some(ExitFocus::Negative) => {
            negative_button = negative_button.style(active_style);
        }
        None => {}
    };

    for error_field in app.error_fields.iter() {
        match error_field {
            TextField::OutputFile => {
                input_style = input_style.fg(Color::Red);
                break;
            }
            _ => {}
        }
    }

    let input_block = Block::default()
        .title("Save To")
        .borders(Borders::ALL)
        .style(input_style);

    let middle_row_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(40),
            Constraint::Fill(1),
        ])
        .split(vertical_panels[1]);

    let input_text = Paragraph::new(match app.target_write_file.clone() {
        None => String::from(""),
        Some(path) => path,
    })
    .block(input_block);

    let positive_label = "save";
    let negative_label = "discard";

    let action_row_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(2),
            Constraint::Length(negative_label.len() as u16 + 2),
            Constraint::Fill(1),
            Constraint::Length(positive_label.len() as u16 + 2),
            Constraint::Fill(2),
        ])
        .split(vertical_panels[2]);

    // the `trim: false` will stop the text from being cut off when over the edge of the block
    let message = Paragraph::new(exit_text).wrap(Wrap { trim: false });

    let positive_text = Paragraph::new(positive_label)
        .block(positive_button)
        .centered();
    let negative_text = Paragraph::new(negative_label)
        .block(negative_button)
        .centered();

    frame.render_widget(popup_block, area);
    frame.render_widget(message, vertical_panels[0]);
    frame.render_widget(input_text, middle_row_panels[1]);
    frame.render_widget(negative_text, action_row_panels[1]);
    frame.render_widget(positive_text, action_row_panels[3]);
}

fn compose_popup(x_constraint: Constraint, y_constraint: Constraint, r: Rect) -> Rect {
    // divide the layout vertically into 3 pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), y_constraint, Constraint::Fill(1)])
        .split(r);

    // divide the center vertical piece into 3 horizontal pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), x_constraint, Constraint::Fill(1)])
        .split(popup_layout[1])[1] // return the center piece
}
