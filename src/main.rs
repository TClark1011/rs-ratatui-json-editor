use std::{error::Error, io};

use app::{App, AppScreen, CurrentlyEditing, InputAction};
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::{Backend, CrosstermBackend};
use ratatui::Terminal;
use ui::ui;

mod app;
mod ui;
fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;

    let mut stderr = io::stderr();

    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Ok(do_print) = res {
        if do_print {
            app.print_json()?;
        }
    } else if let Err(err) = res {
        println!("{err:?}");
    };

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        app.update_state();
        terminal.draw(|frame| ui(frame, app))?;

        if let Event::Key(key_event) = event::read()? {
            if let Some(should_print) = handle_input(app, key_event) {
                // App has exited
                return Ok(should_print);
            };
        }
    }
}

fn handle_input(app: &mut App, key_event: KeyEvent) -> Option<bool> {
    let mut result: Option<bool> = None;
    if key_event.kind == event::KeyEventKind::Release {
        return result;
        // we only want to listen to `Press` events
    }

    let matching_key_bind_res = app
        .available_bindings
        .iter()
        .find(|(key_code, _)| key_code == &key_event.code);

    if let Some((_, action)) = matching_key_bind_res {
        match action {
            InputAction::ExitYesPrint => {
                result = Some(true);
            }
            InputAction::ExitNoPrint => {
                result = Some(false);
            }
            InputAction::ExitCancel => {
                app.goto_screen(AppScreen::Main);
            }
            InputAction::Quit => {
                app.goto_screen(AppScreen::Exiting);
            }
            InputAction::OpenNewPairPopup => {
                app.goto_screen(AppScreen::Editing);
            }
            InputAction::EditingCancel => {
                if app.type_list_open {
                    app.type_list_open = false;
                } else {
                    app.clear_editing_state();
                    app.goto_screen(AppScreen::Main);
                }
            }
            InputAction::EditingToggleField => match app.currently_editing {
                Some(CurrentlyEditing::Key) => {
                    app.currently_editing = Some(CurrentlyEditing::Value);
                }
                Some(CurrentlyEditing::Value) => {
                    app.currently_editing = Some(CurrentlyEditing::Key);
                }
                Some(CurrentlyEditing::Type) => {
                    app.currently_editing = Some(CurrentlyEditing::Key);
                }
                None => {}
            },
            InputAction::EditingSubmit => {
                if app.type_list_open {
                    if let Some(selected_index) = app.type_list_ui_state.selected() {
                        app.type_list_open = false;

                        let value_types = App::get_value_type_vec();
                        let corresponding_json_type = value_types.get(selected_index).unwrap();
                        app.select_value_type(*corresponding_json_type);
                    }
                } else {
                    match app.currently_editing {
                        Some(CurrentlyEditing::Key) => {
                            app.currently_editing = Some(CurrentlyEditing::Value);
                        }
                        Some(CurrentlyEditing::Value) => {
                            app.save_key_value();
                            app.clear_editing_state();
                            app.goto_screen(AppScreen::Main);
                        }
                        Some(CurrentlyEditing::Type) => {
                            app.type_list_open = true;
                        }
                        None => {}
                    };
                }
            }
            InputAction::EditingBackspace => match app.currently_editing {
                Some(CurrentlyEditing::Key) => {
                    app.key_input.pop();
                }
                Some(CurrentlyEditing::Value) => {
                    app.value_input.pop();
                }
                _ => {}
            },
            InputAction::EditingLeft => match app.currently_editing {
                Some(CurrentlyEditing::Value) => {
                    app.currently_editing = Some(CurrentlyEditing::Key);
                }
                Some(CurrentlyEditing::Type) => {
                    app.currently_editing = Some(CurrentlyEditing::Key);
                }
                _ => {}
            },
            InputAction::EditingRight => match app.currently_editing {
                Some(CurrentlyEditing::Key) => {
                    app.currently_editing = Some(CurrentlyEditing::Value);
                }
                Some(CurrentlyEditing::Type) => {
                    app.currently_editing = Some(CurrentlyEditing::Value);
                }
                _ => {}
            },
            InputAction::EditingUp => {
                if app.type_list_open {
                    app.type_list_ui_state.select_previous();
                } else {
                    match app.currently_editing {
                        Some(CurrentlyEditing::Type) => {
                            app.currently_editing = Some(CurrentlyEditing::Key);
                        }
                        _ => {}
                    }
                }
            }
            InputAction::EditingDown => {
                if app.type_list_open {
                    app.type_list_ui_state.select_next();
                } else {
                    match app.currently_editing {
                        Some(CurrentlyEditing::Key) => {
                            app.currently_editing = Some(CurrentlyEditing::Type);
                        }
                        Some(CurrentlyEditing::Value) => {
                            app.currently_editing = Some(CurrentlyEditing::Type);
                        }
                        _ => {}
                    }
                }
            }
            InputAction::EditingBoolToggle => {
                app.value_input = (!(app.value_input.parse::<bool>().unwrap())).to_string();
            }
            InputAction::CursorUp => {
                app.list_ui_state.select_previous();
            }
            InputAction::CursorDown => {
                app.list_ui_state.select_next();
            }
            InputAction::CursorCancel => {
                app.list_ui_state.select(None);
            }
            InputAction::CursorSelect => {
                if let Some(selected_index) = app.list_ui_state.selected() {
                    match app.open_item_edit(selected_index) {
                        Err(_) => return Some(false),
                        Ok(_) => {}
                    }
                }
            }
            InputAction::RequestPairDelete => {
                if let Some(selected_index) = app.list_ui_state.selected() {
                    let entry = match app.pairs.get_index_entry(selected_index) {
                        Some(entry) => entry,
                        None => return Some(false),
                    };
                    let key = entry.key();

                    app.target_delete_key = Some(key.into());
                }
            }
            InputAction::DeleteYes => {
                if let Some(target_key) = &app.target_delete_key {
                    app.pairs.shift_remove(target_key.as_str());
                    app.target_delete_key = None;
                }
            }
            InputAction::DeleteNo => {
                app.target_delete_key = None;
            }
        }
    } else if let AppScreen::Editing = app.get_current_screen() {
        // Special case for typing into the inputs
        if let KeyCode::Char(character) = key_event.code {
            match app.currently_editing {
                Some(CurrentlyEditing::Key) => app.key_input.push(character),
                Some(CurrentlyEditing::Value) => match app.selected_value_type {
                    app::JsonValueType::String => app.value_input.push(character),
                    app::JsonValueType::Number => {
                        if character.is_numeric() || character == '.' {
                            app.value_input.push(character);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    };

    result
}
