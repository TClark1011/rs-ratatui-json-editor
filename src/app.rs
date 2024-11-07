use std::collections::HashMap;

use ratatui::crossterm::event::KeyCode;

pub enum AppScreen {
    Main,
    Editing,
    Exiting,
}

pub enum CurrentlyEditing {
    Key,
    Value,
}

pub enum InputAction {
    OpenNewPairPopup,
    Quit,
    YesPrint,
    NoPrint,
    EditingSubmit,
    EditingCancel,
    EditingToggleField,
    EditingBackspace,
}

impl InputAction {
    pub fn description(&self) -> Option<&str> {
        match self {
            InputAction::OpenNewPairPopup => Some("new pair"),
            InputAction::Quit => Some("quit"),
            _ => None,
        }
    }
}

pub type KeyBinding = (KeyCode, InputAction);

pub struct App {
    pub key_input: String,
    pub value_input: String,
    pub pairs: HashMap<String, String>,
    pub currently_editing: Option<CurrentlyEditing>,
    pub available_bindings: Vec<KeyBinding>,
    pub current_screen: AppScreen,
}

impl App {
    pub fn new() -> App {
        let mut result = App {
            key_input: String::new(),
            value_input: String::new(),
            pairs: HashMap::new(),
            current_screen: AppScreen::Main,
            currently_editing: None,
            available_bindings: Vec::new(),
        };

        result.update_bindings();

        result
    }

    pub fn goto_screen(&mut self, new_screen: AppScreen) {
        self.current_screen = new_screen;
        self.update_bindings();
    }

    fn update_bindings(&mut self) {
        self.available_bindings = match self.current_screen {
            AppScreen::Main => {
                vec![
                    (KeyCode::Char('e'), InputAction::OpenNewPairPopup),
                    (KeyCode::Char('q'), InputAction::Quit),
                ]
            }
            AppScreen::Editing => vec![
                (KeyCode::Enter, InputAction::EditingSubmit),
                (KeyCode::Tab, InputAction::EditingToggleField),
                (KeyCode::Esc, InputAction::EditingCancel),
                (KeyCode::Backspace, InputAction::EditingBackspace),
            ],
            AppScreen::Exiting => vec![
                (KeyCode::Char('y'), InputAction::YesPrint),
                (KeyCode::Char('n'), InputAction::NoPrint),
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
