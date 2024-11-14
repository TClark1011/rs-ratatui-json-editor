use std::{error::Error, io};

use app::{ActionBinding, App, AppError, AppScreen, Binding, CurrentlyEditing, InputAction};
use clap::Parser;
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

#[derive(Parser)]
#[command(about)]
struct CliArgs {
    /// The input file to read from
    input_file: Option<String>,

    /// Whether to run in "dry" mode (no changes will be written to the output file)
    #[arg(long)]
    dry: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();

    let mut app = App::new(args.input_file)
        .map_err(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        })
        .unwrap();

    // Prepare the terminal for the application
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // Run the application
    let app_result = run_app(&mut terminal, &mut app);

    // Restore the terminal to its original state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match app_result {
        Ok(should_save) => {
            if !args.dry && should_save {
                app.write()?;
            }
            return Ok(());
        }
        Err(err) => {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<bool, AppError> {
    loop {
        app.update_state();
        terminal
            .draw(|frame| ui(frame, app))
            .map_err(AppError::FailedToDraw)?;

        if let Event::Key(key_event) = event::read().map_err(AppError::FailedToReadEvent)? {
            match handle_input(app, key_event) {
                Ok(Some(should_save)) => {
                    return Ok(should_save);
                }
                Err(err) => {
                    return Err(err);
                }
                _ => {}
            }
        }
    }
}

/// Interpreting `Ok` return values
/// - `None` - continue running the app
/// - `Some(bool)` - Exit the app, the bool value
/// indicates whether changes should be saved
fn handle_input(app: &mut App, key_event: KeyEvent) -> Result<Option<bool>, AppError> {
    if key_event.kind == event::KeyEventKind::Release {
        // we only want to listen to `Press` events
        return Ok(None);
    }

    let mut text_entry_action: Option<InputAction> = None;
    let mut matching_action_binding_res: Option<ActionBinding> = None;

    for (binding, action) in app.available_bindings.iter() {
        match binding {
            Binding::Static(key_code) => {
                if key_code == &key_event.code {
                    matching_action_binding_res = Some((*binding, *action));
                    break;
                }
            }
            Binding::TextEntry => {
                if let KeyCode::Char(_) = key_event.code {
                    text_entry_action = Some(*action);
                }
            }
        }
    }

    // We only want to use the text entry binding if no binding
    // was found for the current key event
    matching_action_binding_res = matching_action_binding_res.or_else(|| {
        if let Some(action) = text_entry_action {
            Some((Binding::TextEntry, action))
        } else {
            None
        }
    });

    if let Some((_, action)) = matching_action_binding_res {
        match action {
            InputAction::EnterKeyText => {
                if let KeyCode::Char(character) = key_event.code {
                    app.key_input.push(character)
                }
            }
            InputAction::EnterValueText => {
                if let KeyCode::Char(character) = key_event.code {
                    app.value_input.push(character)
                }
            }
            InputAction::ExitYesSave => {
                return Ok(Some(true));
            }
            InputAction::ExitNoSave => {
                return Ok(Some(false));
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
                    app.open_item_edit(selected_index)
                        .map_err(AppError::FailedToOpenPairEdit)?;
                }
            }
            InputAction::RequestPairDelete => {
                if let Some(selected_index) = app.list_ui_state.selected() {
                    let entry = match app.pairs.get_index_entry(selected_index) {
                        Some(entry) => entry,
                        None => return Err(AppError::NoEntryAtIndex(selected_index)),
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
            InputAction::Preview => {
                app.goto_screen(AppScreen::Preview);
            }
            InputAction::ExitPreview => {
                app.goto_screen(AppScreen::Main);
            }
        }
    };

    Ok(None)
}
