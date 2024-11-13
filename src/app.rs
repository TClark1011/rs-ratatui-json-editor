use core::fmt;
use std::fs;
use std::io::{self, Write};
use std::{
    fmt::{Display, Formatter},
    fs::File,
};

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
    Type,
}

pub enum InputAction {
    Quit,
    ExitYesSave,
    ExitNoSave,
    ExitCancel,
    OpenNewPairPopup,
    EditingSubmit,
    EditingCancel,
    EditingToggleField,
    EditingBackspace,
    EditingUp,
    EditingDown,
    EditingLeft,
    EditingRight,
    EditingBoolToggle,
    CursorUp,
    CursorDown,
    CursorCancel,
    CursorSelect,
    RequestPairDelete,
    DeleteYes,
    DeleteNo,
}

impl InputAction {
    pub fn description(&self) -> Option<&str> {
        match self {
            InputAction::OpenNewPairPopup => Some("new"),
            InputAction::Quit => Some("quit"),
            InputAction::EditingCancel => Some("cancel"),
            InputAction::EditingToggleField => Some("switch"),
            InputAction::EditingSubmit => Some("submit"),
            InputAction::CursorSelect => Some("select"),
            InputAction::CursorDown => Some("down"),
            InputAction::CursorUp => Some("up"),
            InputAction::CursorCancel => Some("cancel"),
            InputAction::EditingBoolToggle => Some("toggle"),
            InputAction::RequestPairDelete => Some("delete"),
            InputAction::DeleteYes => Some("yes"),
            InputAction::DeleteNo => Some("no"),
            _ => None,
        }
    }
}

pub type KeyBinding = (KeyCode, InputAction);

pub enum OpenItemEditError {
    InvalidIndex,
}

#[derive(Clone, Copy)]
pub enum JsonValueType {
    Number,
    String,
    Boolean,
}

impl Display for JsonValueType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            JsonValueType::Number => write!(f, "Number"),
            JsonValueType::String => write!(f, "String"),
            JsonValueType::Boolean => write!(f, "Boolean"),
        }
    }
}

pub enum JsonValue {
    Number(f64),
    String(String),
    Boolean(bool),
}

impl JsonValue {
    pub fn from_serde(serde_value: serde_json::Value) -> Result<Self, JsonValueFromSerdeError> {
        match serde_value {
            serde_json::Value::Number(n) => Ok(JsonValue::Number(n.as_f64().unwrap_or(0.0))),
            serde_json::Value::String(s) => Ok(JsonValue::String(s)),
            serde_json::Value::Bool(b) => Ok(JsonValue::Boolean(b)),
            _ => Err(JsonValueFromSerdeError::UnsupportedType),
        }
    }
}

impl serde::Serialize for JsonValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            JsonValue::Number(n) => {
                if n.fract() != 0.0 {
                    serializer.serialize_f64(*n)
                } else if *n < 0.0 {
                    serializer.serialize_i64(*n as i64)
                } else {
                    serializer.serialize_u64(*n as u64)
                }
            }
            JsonValue::String(s) => serializer.serialize_str(s),
            JsonValue::Boolean(b) => serializer.serialize_bool(*b),
        }
    }
}

type JsonData = IndexMap<String, JsonValue>;

pub struct App {
    pub key_input: String,
    pub value_input: String,
    pub pairs: JsonData,
    pub currently_editing: Option<CurrentlyEditing>,
    pub available_bindings: Vec<KeyBinding>,
    pub list_ui_state: ListState,
    pub selected_value_type: JsonValueType,
    pub type_list_ui_state: ListState,
    pub type_list_open: bool,
    pub target_delete_key: Option<String>,
    pub target_write_file: Option<String>,
    current_screen: AppScreen,
}

impl App {
    pub fn new(input_file_path: Option<String>) -> Result<App, AppError> {
        let input_file_contents = input_file_path
            .clone()
            .map(fs::read_to_string)
            .map(Result::ok)
            .flatten();

        if input_file_path.is_some() && input_file_contents.is_none() {
            return Err(AppError::InputFileNotFound);
        }

        let parsed_data: Option<serde_json::Value> = input_file_contents
            .map(|s| serde_json::from_str(s.as_str()))
            .map(Result::ok)
            .flatten();

        let data_read_opt: Option<JsonData> = match parsed_data {
            None => Some(IndexMap::new()),
            Some(serde_json::Value::Object(data)) => {
                let mut ret = JsonData::new();

                let parse_attempt: Result<(), JsonValueFromSerdeError> =
                    data.into_iter().try_for_each(|(key, value)| {
                        let json_value = JsonValue::from_serde(value)?;
                        ret.insert(key, json_value);
                        Ok(())
                    });

                if parse_attempt.is_err() {
                    None
                } else {
                    Some(ret)
                }
            }
            _ => None,
        };

        match data_read_opt {
            None => Err(AppError::InvalidInputJson),
            Some(data) => {
                let mut result = App {
                    key_input: String::new(),
                    value_input: String::new(),
                    pairs: data,
                    currently_editing: None,
                    available_bindings: Vec::new(),
                    list_ui_state: ListState::default(),
                    current_screen: AppScreen::Main,
                    selected_value_type: JsonValueType::String,
                    type_list_ui_state: ListState::default(),
                    type_list_open: false,
                    target_delete_key: None,
                    target_write_file: input_file_path,
                };
                result.update_state();

                Ok(result)
            }
        }
    }

    pub fn get_current_screen(&self) -> &AppScreen {
        &self.current_screen
    }

    pub fn goto_screen(&mut self, new_screen: AppScreen) {
        if let AppScreen::Editing = new_screen {
            self.currently_editing = Some(CurrentlyEditing::Key);
        }
        self.current_screen = new_screen;
    }

    pub fn update_state(&mut self) {
        self.available_bindings = match self.current_screen {
            AppScreen::Main => {
                let delete_modal_is_open = self.target_delete_key.is_some();
                if delete_modal_is_open {
                    vec![
                        (KeyCode::Char('y'), InputAction::DeleteYes),
                        (KeyCode::Char('n'), InputAction::DeleteNo),
                    ]
                } else {
                    let mut result = vec![
                        (KeyCode::Char('e'), InputAction::OpenNewPairPopup),
                        (KeyCode::Char('q'), InputAction::Quit),
                    ];

                    if !self.pairs.is_empty() && !delete_modal_is_open {
                        result.push((KeyCode::Enter, InputAction::CursorSelect));
                        result.push((KeyCode::Down, InputAction::CursorDown));
                        result.push((KeyCode::Up, InputAction::CursorUp));

                        if self.list_ui_state.selected().is_some() {
                            result.push((KeyCode::Esc, InputAction::CursorCancel));
                            result.push((KeyCode::Backspace, InputAction::RequestPairDelete));
                        }
                    }

                    result
                }
            }
            AppScreen::Editing => {
                self.list_ui_state.select(None);

                if self.type_list_open && !self.type_list_ui_state.selected().is_some() {
                    self.type_list_ui_state.select_first();
                }
                let mut result = vec![
                    (KeyCode::Enter, InputAction::EditingSubmit),
                    (KeyCode::Tab, InputAction::EditingToggleField),
                    (KeyCode::Esc, InputAction::EditingCancel),
                    (KeyCode::Backspace, InputAction::EditingBackspace),
                    (KeyCode::Up, InputAction::EditingUp),
                    (KeyCode::Down, InputAction::EditingDown),
                    (KeyCode::Left, InputAction::EditingLeft),
                    (KeyCode::Right, InputAction::EditingRight),
                ];

                if let JsonValueType::Boolean = self.selected_value_type {
                    if let Some(CurrentlyEditing::Value) = self.currently_editing {
                        result.push((KeyCode::Char(' '), InputAction::EditingBoolToggle));
                    }
                }

                result
            }
            AppScreen::Exiting => vec![
                (KeyCode::Char('y'), InputAction::ExitYesSave),
                (KeyCode::Char('n'), InputAction::ExitNoSave),
                (KeyCode::Esc, InputAction::ExitCancel),
            ],
        };
    }

    pub fn select_value_type(&mut self, new_type: JsonValueType) {
        match new_type {
            JsonValueType::Boolean => {
                self.value_input = "false".to_string();
            }
            _ => {
                self.value_input = "".to_string();
            }
        }
        self.selected_value_type = new_type;
    }

    pub fn save_key_value(&mut self) {
        self.pairs.insert(
            self.key_input.clone(),
            match self.selected_value_type {
                JsonValueType::Number => JsonValue::Number(self.value_input.parse().unwrap_or(0.0)),
                JsonValueType::Boolean => {
                    JsonValue::Boolean(self.value_input.parse().unwrap_or(false))
                }
                JsonValueType::String => JsonValue::String(self.value_input.clone()),
            },
        );
    }

    pub fn clear_editing_state(&mut self) {
        self.key_input.clear();
        self.value_input.clear();
        self.currently_editing = None;
    }

    pub fn open_item_edit(&mut self, index: usize) -> Result<(), OpenItemEditError> {
        match self.pairs.get_index(index) {
            None => Err(OpenItemEditError::InvalidIndex),
            // Some(key, JsonValue::String(value)) => {}
            Some((key, json_value)) => {
                self.key_input = key.clone();
                match json_value {
                    JsonValue::String(value) => {
                        self.value_input = value.clone();
                    }
                    JsonValue::Boolean(bool) => {
                        self.value_input = bool.to_string();
                    }
                    JsonValue::Number(number) => {
                        self.value_input = number.to_string();
                    }
                };
                self.goto_screen(AppScreen::Editing);
                self.currently_editing = Some(CurrentlyEditing::Value);

                Ok(())
            }
        }
    }

    pub fn serialize(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.pairs)
    }

    pub fn get_value_type_vec() -> Vec<JsonValueType> {
        vec![
            JsonValueType::String,
            JsonValueType::Number,
            JsonValueType::Boolean,
        ]
    }

    pub fn write(&self) -> Result<(), AppWriteError> {
        let serialized = self.serialize().map_err(AppWriteError::Serde)?;

        match &self.target_write_file {
            Some(path) => {
                let mut file = File::create(path).map_err(AppWriteError::Io)?;

                file.write_all(serialized.as_bytes())
                    .map_err(AppWriteError::Io)?;
            }
            _ => {}
        };

        Ok(())
    }
}

#[derive(Debug)]
pub enum AppError {
    InputFileNotFound,
    InvalidInputJson,
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AppError::InputFileNotFound => write!(f, "Input file not found"),
            AppError::InvalidInputJson => write!(f, "Invalid input JSON"),
        }
    }
}

impl std::error::Error for AppError {}

pub enum JsonValueFromSerdeError {
    UnsupportedType,
}

#[derive(Debug)]
pub enum AppWriteError {
    Serde(serde_json::Error),
    Io(io::Error),
}

impl Display for AppWriteError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AppWriteError::Serde(e) => write!(f, "Serde error: {e}"),
            AppWriteError::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for AppWriteError {}
