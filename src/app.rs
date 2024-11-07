use indexmap::IndexMap;
use ratatui::{crossterm::event::KeyCode, widgets::ListState};

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
    Quit,
    ExitYesPrint,
    ExitNoPrint,
    ExitCancel,
    OpenNewPairPopup,
    EditingSubmit,
    EditingCancel,
    EditingToggleField,
    EditingBackspace,
    CursorUp,
    CursorDown,
    CursorCancel,
    CursorSelect,
}

impl InputAction {
    pub fn description(&self) -> Option<&str> {
        match self {
            InputAction::OpenNewPairPopup => Some("new pair"),
            InputAction::Quit => Some("quit"),
            InputAction::EditingCancel => Some("cancel"),
            InputAction::EditingToggleField => Some("switch"),
            InputAction::EditingSubmit => Some("submit"),
            _ => None,
        }
    }
}

pub type KeyBinding = (KeyCode, InputAction);

pub enum OpenItemEditError {
    InvalidIndex,
}

pub struct App {
    pub key_input: String,
    pub value_input: String,
    pub pairs: IndexMap<String, String>,
    pub currently_editing: Option<CurrentlyEditing>,
    pub available_bindings: Vec<KeyBinding>,
    pub list_ui_state: ListState,
    current_screen: AppScreen,
}

impl App {
    pub fn new() -> App {
        let mut result = App {
            key_input: String::new(),
            value_input: String::new(),
            pairs: IndexMap::new(),
            currently_editing: None,
            available_bindings: Vec::new(),
            list_ui_state: ListState::default(),
            current_screen: AppScreen::Main,
        };

        result.update_screen_related_settings();

        result
    }

    pub fn get_current_screen(&self) -> &AppScreen {
        &self.current_screen
    }

    pub fn goto_screen(&mut self, new_screen: AppScreen) {
        self.current_screen = new_screen;
        self.update_screen_related_settings();
    }

    fn update_screen_related_settings(&mut self) {
        self.available_bindings = match self.current_screen {
            AppScreen::Main => {
                vec![
                    (KeyCode::Char('e'), InputAction::OpenNewPairPopup),
                    (KeyCode::Char('q'), InputAction::Quit),
                    (KeyCode::Up, InputAction::CursorUp),
                    (KeyCode::Down, InputAction::CursorDown),
                    (KeyCode::Esc, InputAction::CursorCancel),
                    (KeyCode::Enter, InputAction::CursorSelect),
                ]
            }
            AppScreen::Editing => {
                self.list_ui_state.select(None);
                self.currently_editing = Some(CurrentlyEditing::Key);
                vec![
                    (KeyCode::Enter, InputAction::EditingSubmit),
                    (KeyCode::Tab, InputAction::EditingToggleField),
                    (KeyCode::Esc, InputAction::EditingCancel),
                    (KeyCode::Backspace, InputAction::EditingBackspace),
                ]
            }
            AppScreen::Exiting => vec![
                (KeyCode::Char('y'), InputAction::ExitYesPrint),
                (KeyCode::Char('n'), InputAction::ExitNoPrint),
                (KeyCode::Esc, InputAction::ExitCancel),
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

    pub fn open_item_edit(&mut self, index: usize) -> Result<(), OpenItemEditError> {
        match self.pairs.get_index(index) {
            None => Err(OpenItemEditError::InvalidIndex),
            Some((key, value)) => {
                self.key_input = key.clone();
                self.value_input = value.clone();
                self.goto_screen(AppScreen::Editing);
                self.currently_editing = Some(CurrentlyEditing::Value);

                Ok(())
            }
        }
    }

    pub fn print_json(&self) -> serde_json::Result<()> {
        let output = serde_json::to_string(&self.pairs)?;
        println!("{output}");

        Ok(())
    }
}
