use std::collections::HashMap;

use ratatui::crossterm::event::KeyCode;

// TODO: rename this enum
pub enum CurrentScreen {
    Main,
    Editing,
    Exiting,
}

pub enum CurrentlyEditing {
    Key,
    Value,
}

pub enum Action {
    OpenNewPairPopup,
    Quit,
    YesPrint,
    NoPrint,
    EditingSubmit,
    EditingCancel,
    EditingToggleField,
    EditingBackspace,
}

impl Action {
    pub fn description(&self) -> Option<&str> {
        match self {
            Action::OpenNewPairPopup => Some("new pair"),
            Action::Quit => Some("quit"),
            _ => None,
        }
    }
}

pub type KeyBinding = (KeyCode, Action);

pub struct App {
    pub key_input: String,
    pub value_input: String,
    pub pairs: HashMap<String, String>,
    pub currently_editing: Option<CurrentlyEditing>,
    pub available_bindings: Vec<KeyBinding>,
    pub current_screen: CurrentScreen,
}

impl App {
    pub fn new() -> App {
        let mut result = App {
            key_input: String::new(),
            value_input: String::new(),
            pairs: HashMap::new(),
            current_screen: CurrentScreen::Main,
            currently_editing: None,
            available_bindings: Vec::new(),
        };

        result.update_bindings();

        result
    }

    pub fn goto_screen(&mut self, new_screen: CurrentScreen) {
        self.current_screen = new_screen;
        self.update_bindings();
    }

    fn update_bindings(&mut self) {
        self.available_bindings = match self.current_screen {
            CurrentScreen::Main => {
                vec![
                    (KeyCode::Char('e'), Action::OpenNewPairPopup),
                    (KeyCode::Char('q'), Action::Quit),
                ]
            }
            CurrentScreen::Editing => vec![
                (KeyCode::Enter, Action::EditingSubmit),
                (KeyCode::Tab, Action::EditingToggleField),
                (KeyCode::Esc, Action::EditingCancel),
                (KeyCode::Backspace, Action::EditingBackspace),
            ],
            CurrentScreen::Exiting => vec![
                (KeyCode::Char('y'), Action::YesPrint),
                (KeyCode::Char('n'), Action::NoPrint),
            ],
        };
    }

    pub fn save_key_value(&mut self) {
        self.pairs
            .insert(self.key_input.clone(), self.value_input.clone());
    }

    pub fn clear_editing_state(&mut self) {
        self.key_input.clear();
        self.value_input.clear();
        self.currently_editing = None;
    }

    pub fn toggle_editing(&mut self) {
        if let Some(edit_mode) = &self.currently_editing {
            match edit_mode {
                CurrentlyEditing::Key => self.currently_editing = Some(CurrentlyEditing::Value),
                CurrentlyEditing::Value => self.currently_editing = Some(CurrentlyEditing::Key),
            }
        } else {
            self.currently_editing = Some(CurrentlyEditing::Key)
        }
    }

    pub fn print_json(&self) -> serde_json::Result<()> {
        let output = serde_json::to_string(&self.pairs)?;
        println!("{output}");

        Ok(())
    }
}
