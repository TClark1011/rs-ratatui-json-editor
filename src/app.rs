use core::fmt;
use std::fs;
use std::io::{self, Write};
use std::{
    fmt::{Display, Formatter},
    fs::File,
};

use indexmap::IndexMap;
use ratatui::{crossterm::event::KeyCode, widgets::ListState};

pub struct App {
    pub key_input: String,
    pub value_input: String,
    pub pairs: JsonData,
    pub edit_popup_focus: Option<EditFocus>,
    pub exit_popup_focus: Option<ExitFocus>,
    pub available_bindings: Vec<ActionBinding>,
    pub list_ui_state: ListState,
    pub selected_value_type: JsonValueType,
    pub type_list_ui_state: ListState,
    pub type_list_open: bool,
    pub target_delete_key: Option<String>,
    pub target_write_file: Option<String>,
    current_screen: AppScreen,
}

impl App {
    pub fn all_value_types() -> Vec<JsonValueType> {
        vec![
            JsonValueType::String,
            JsonValueType::Number,
            JsonValueType::Boolean,
            JsonValueType::Null,
        ]
    }

    pub fn new(input_file_path: Option<String>) -> Result<App, AppError> {
        let input_file_contents = input_file_path
            .clone()
            .map(fs::read_to_string)
            .map(Result::ok)
            .flatten();

        if input_file_path.is_some() && input_file_contents.is_none() {
            return Err(AppError::InputFileNotFound(input_file_path.unwrap()));
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
                    edit_popup_focus: None,
                    exit_popup_focus: None,
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
        match new_screen {
            AppScreen::Editing => {
                self.edit_popup_focus = Some(EditFocus::Key);
            }
            AppScreen::Exiting => {
                self.exit_popup_focus = Some(ExitFocus::Input);
            }
            _ => {}
        }
        self.current_screen = new_screen;
    }

    pub fn update_state(&mut self) {
        self.available_bindings = match self.current_screen {
            AppScreen::Main => {
                let delete_modal_is_open = self.target_delete_key.is_some();
                if delete_modal_is_open {
                    vec![
                        (Binding::Static(KeyCode::Char('y')), InputAction::DeleteYes),
                        (Binding::Static(KeyCode::Char('n')), InputAction::DeleteNo),
                    ]
                } else {
                    let mut result = vec![
                        (
                            Binding::Static(KeyCode::Char('e')),
                            InputAction::OpenNewPairPopup,
                        ),
                        (Binding::Static(KeyCode::Char('q')), InputAction::Quit),
                        (Binding::Static(KeyCode::Char('p')), InputAction::Preview),
                    ];

                    if !self.pairs.is_empty() && !delete_modal_is_open {
                        result.push((Binding::Static(KeyCode::Enter), InputAction::CursorSelect));
                        result.push((Binding::Static(KeyCode::Down), InputAction::CursorDown));
                        result.push((Binding::Static(KeyCode::Up), InputAction::CursorUp));

                        if self.list_ui_state.selected().is_some() {
                            result.push((Binding::Static(KeyCode::Esc), InputAction::CursorCancel));
                            result.push((
                                Binding::Static(KeyCode::Backspace),
                                InputAction::RequestPairDelete,
                            ));
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
                    (Binding::Static(KeyCode::Enter), InputAction::EditingSubmit),
                    (
                        Binding::Static(KeyCode::Tab),
                        InputAction::EditingToggleField,
                    ),
                    (Binding::Static(KeyCode::Esc), InputAction::EditingCancel),
                    (Binding::Static(KeyCode::Up), InputAction::EditingUp),
                    (Binding::Static(KeyCode::Down), InputAction::EditingDown),
                    (Binding::Static(KeyCode::Left), InputAction::EditingLeft),
                    (Binding::Static(KeyCode::Right), InputAction::EditingRight),
                ];

                match self.edit_popup_focus {
                    Some(EditFocus::Value) => {
                        result.push((
                            Binding::Static(KeyCode::Backspace),
                            InputAction::BackspaceFieldText(TextField::Value),
                        ));
                        result.push((
                            Binding::TextEntry,
                            InputAction::EnterFieldText(TextField::Value),
                        ));

                        if let JsonValueType::Boolean = self.selected_value_type {
                            result.push((
                                Binding::Static(KeyCode::Char('t')),
                                InputAction::EditingBoolToggle,
                            ));
                        }
                    }
                    Some(EditFocus::Key) => {
                        result.push((
                            Binding::Static(KeyCode::Backspace),
                            InputAction::BackspaceFieldText(TextField::Key),
                        ));
                        result.push((
                            Binding::TextEntry,
                            InputAction::EnterFieldText(TextField::Key),
                        ));
                    }
                    _ => {}
                }

                result
            }
            AppScreen::Exiting => {
                let mut result = vec![
                    (Binding::Static(KeyCode::Esc), InputAction::ExitCancel),
                    (Binding::Static(KeyCode::Up), InputAction::ExitUp),
                    (Binding::Static(KeyCode::Down), InputAction::ExitDown),
                    (Binding::Static(KeyCode::Left), InputAction::ExitLeft),
                    (Binding::Static(KeyCode::Right), InputAction::ExitRight),
                    (
                        Binding::Static(KeyCode::Enter),
                        InputAction::ExitCursorSelect,
                    ),
                ];

                match self.exit_popup_focus {
                    Some(ExitFocus::Input) => {
                        result.push((
                            Binding::Static(KeyCode::Backspace),
                            InputAction::BackspaceFieldText(TextField::OutputFile),
                        ));
                        result.push((
                            Binding::TextEntry,
                            InputAction::EnterFieldText(TextField::OutputFile),
                        ));
                    }
                    _ => {}
                }

                result
            }
            AppScreen::Preview => vec![(Binding::Static(KeyCode::Esc), InputAction::ExitPreview)],
        };
    }

    pub fn select_value_type(&mut self, new_type: JsonValueType) {
        match new_type {
            JsonValueType::Boolean => {
                self.value_input = "false".to_string();
            }
            JsonValueType::Null => {
                self.value_input = "null".to_string();
            }
            JsonValueType::String => {
                self.value_input = "".to_string();
            }
            JsonValueType::Number => {
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
                JsonValueType::Null => JsonValue::Null,
            },
        );
    }

    pub fn clear_editing_state(&mut self) {
        self.key_input.clear();
        self.value_input.clear();
        self.edit_popup_focus = None;
    }

    pub fn open_item_edit(&mut self, index: usize) -> Result<(), OpenItemEditError> {
        match self.pairs.get_index(index) {
            None => Err(OpenItemEditError::InvalidIndex(index)),
            // Some(key, JsonValue::String(value)) => {}
            Some((key, json_value)) => {
                self.key_input = key.clone();
                self.value_input = match json_value {
                    JsonValue::String(value) => value.clone(),
                    JsonValue::Null => "null".to_string(),
                    JsonValue::Boolean(value) => value.to_string(),
                    JsonValue::Number(value) => value.to_string(),
                };
                self.goto_screen(AppScreen::Editing);
                self.edit_popup_focus = Some(EditFocus::Value);

                Ok(())
            }
        }
    }

    pub fn serialize(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.pairs)
    }

    pub fn write(&self) -> Result<(), AppError> {
        let serialized = self
            .serialize()
            .map_err(|e| AppError::UnableToSave(AppWriteError::Serde(e)))?;

        match &self.target_write_file {
            Some(path) => {
                let mut file =
                    File::create(path).map_err(|e| AppError::UnableToSave(AppWriteError::Io(e)))?;

                file.write_all(serialized.as_bytes())
                    .map_err(|e| AppError::UnableToSave(AppWriteError::Io(e)))?;
            }
            _ => {}
        };

        Ok(())
    }
}

pub enum AppScreen {
    Main,
    Editing,
    Exiting,
    Preview,
}

pub enum EditFocus {
    Key,
    Value,
    Type,
}

#[derive(Clone, Copy)]
pub enum ExitFocus {
    Input,
    Positive,
    Negative,
}

#[derive(Clone, Copy)]
pub enum TextField {
    Key,
    Value,
    OutputFile,
}

#[derive(Clone, Copy)]
pub enum InputAction {
    Quit,
    ExitCancel,
    OpenNewPairPopup,
    EditingSubmit,
    EditingCancel,
    EditingToggleField,
    EditingUp,
    EditingDown,
    EditingLeft,
    EditingRight,
    ExitUp,
    ExitDown,
    ExitLeft,
    ExitRight,
    ExitCursorSelect,
    EditingBoolToggle,
    CursorUp,
    CursorDown,
    CursorCancel,
    CursorSelect,
    RequestPairDelete,
    DeleteYes,
    DeleteNo,
    ExitPreview,
    Preview,
    EnterFieldText(TextField),
    BackspaceFieldText(TextField),
}

#[derive(Clone, Copy)]
pub enum Binding {
    Static(KeyCode),
    TextEntry,
}

impl Display for Binding {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Binding::Static(key_code) => write!(f, "{key_code}"),
            Binding::TextEntry => write!(f, "Text Entry"),
        }
    }
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
            InputAction::ExitPreview => Some("exit"),
            InputAction::Preview => Some("preview"),
            _ => None,
        }
    }
}

pub type ActionBinding = (Binding, InputAction);

#[derive(Debug)]
pub enum OpenItemEditError {
    InvalidIndex(usize),
}

impl Display for OpenItemEditError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            OpenItemEditError::InvalidIndex(idx) => write!(f, "Invalid index {idx}"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum JsonValueType {
    Number,
    String,
    Boolean,
    Null,
}

impl Display for JsonValueType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            JsonValueType::Number => write!(f, "Number"),
            JsonValueType::String => write!(f, "String"),
            JsonValueType::Boolean => write!(f, "Boolean"),
            JsonValueType::Null => write!(f, "null"),
        }
    }
}

#[derive(Clone)]
pub enum JsonValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl JsonValue {
    pub fn from_serde(serde_value: serde_json::Value) -> Result<Self, JsonValueFromSerdeError> {
        match serde_value {
            serde_json::Value::Number(n) => Ok(JsonValue::Number(n.as_f64().unwrap_or(0.0))),
            serde_json::Value::String(s) => Ok(JsonValue::String(s)),
            serde_json::Value::Bool(b) => Ok(JsonValue::Boolean(b)),
            serde_json::Value::Null => Ok(JsonValue::Null),
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
                    // if it is not a whole number serialize as float
                    serializer.serialize_f64(*n)
                } else if *n < 0.0 {
                    // if its negative serialize as a signed integer
                    serializer.serialize_i64(*n as i64)
                } else {
                    // if its positive serialize as an unsigned integer
                    serializer.serialize_u64(*n as u64)
                }
            }
            JsonValue::String(s) => serializer.serialize_str(s),
            JsonValue::Boolean(b) => serializer.serialize_bool(*b),
            JsonValue::Null => serializer.serialize_none(),
        }
    }
}

pub type JsonData = IndexMap<String, JsonValue>;

#[derive(Debug)]
pub enum AppError {
    InputFileNotFound(String),
    InvalidInputJson,
    FailedToOpenPairEdit(OpenItemEditError),
    NoEntryAtIndex(usize),
    UnableToSave(AppWriteError),
    FailedToDraw(io::Error),
    FailedToReadEvent(io::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AppError::InputFileNotFound(path) => write!(f, "No file found at path: {path}"),
            AppError::InvalidInputJson => write!(f, "Invalid input JSON"),
            AppError::FailedToOpenPairEdit(e) => write!(f, "Failed to open pair for editing: {e}"),
            AppError::UnableToSave(e) => write!(f, "Failed to write file: {e}"),
            AppError::FailedToDraw(e) => write!(f, "An error occurred while rendering the UI: {e}"),
            AppError::FailedToReadEvent(e) => {
                write!(f, "An error occurred while reading input: {e}")
            }
            AppError::NoEntryAtIndex(usize) => write!(f, "No entry exists at index {usize}"),
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
