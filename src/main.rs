use std::{error::Error, io};

use app::{Action, App, CurrentScreen, CurrentlyEditing};
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
            Action::Quit => {
                app.goto_screen(CurrentScreen::Exiting);
            }
            Action::OpenNewPairPopup => {
                app.goto_screen(CurrentScreen::Editing);
                app.toggle_editing();
            }
            Action::EditingCancel => {
                app.clear_editing_state();
                app.goto_screen(CurrentScreen::Main);
            }
            Action::EditingToggleField => {
                app.toggle_editing();
            }
            Action::EditingSubmit => match app.currently_editing {
                Some(CurrentlyEditing::Key) => {
                    app.currently_editing = Some(CurrentlyEditing::Value);
                }
                Some(CurrentlyEditing::Value) => {
                    app.save_key_value();
                    app.clear_editing_state();
                    app.goto_screen(CurrentScreen::Main);
                }
                None => {}
            },
            Action::EditingBackspace => match app.currently_editing {
                Some(CurrentlyEditing::Key) => {
                    app.key_input.pop();
                }
                Some(CurrentlyEditing::Value) => {
                    app.value_input.pop();
                }
                None => {}
            },
            Action::YesPrint => {
                result = Some(true);
            }
            Action::NoPrint => {
                result = Some(false);
            }
        }
    } else if let CurrentScreen::Editing = app.current_screen {
        // Special case for typing into the inputs
        if let KeyCode::Char(character) = key_event.code {
            match app.currently_editing {
                Some(CurrentlyEditing::Key) => app.key_input.push(character),
                Some(CurrentlyEditing::Value) => app.value_input.push(character),
                None => {}
            }
        }
    };

    result
}
