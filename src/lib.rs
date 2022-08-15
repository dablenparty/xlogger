use std::path::PathBuf;

use eframe::egui::{Context, Ui};
use eframe::epaint::Color32;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::util::get_exe_parent_dir;

pub mod button_graph;
pub mod gilrs_loop;
pub mod stick_graph;
pub mod util;

/// Helper type for a Result that can trap any boxed error
pub type BoxedResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Contains functions for displaying a struct with egui
pub trait EguiView {
    /// Display the struct with egui
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context to use
    /// * `is_open` - Mutable reference to the boolean that determines if the window is open
    fn show(&mut self, ctx: &Context, is_open: &mut bool);
    /// Constructs the UI for the struct
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui ui to use
    fn ui(&mut self, ui: &mut Ui);
}

/// A helper struct for working with a pair of MPSC sender & receiver channels
#[derive(Debug)]
pub struct CrossbeamChannelPair<T> {
    /// The sender channel
    pub tx: crossbeam_channel::Sender<T>,
    /// The receiver channel
    pub rx: crossbeam_channel::Receiver<T>,
}

impl<T> Clone for CrossbeamChannelPair<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rx: self.rx.clone(),
        }
    }
}

impl<T> Default for CrossbeamChannelPair<T> {
    fn default() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded::<T>();
        Self { tx, rx }
    }
}

/// Represents data for a controller connection event
#[derive(Debug, Clone)]
pub struct ControllerConnectionEvent {
    /// `true` if the controller is connected, `false` otherwise
    pub connected: bool,
    /// The controller ID
    pub controller_id: gilrs::GamepadId,
    /// The controller name
    pub gamepad_name: String,
}

/// Represents a controller button event
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ControllerButtonEvent {
    /// When the button was pressed
    pub press_time: f64,
    /// When the button was released
    pub release_time: f64,
    /// The button that was pressed
    pub button: gilrs::Button,
}

/// Represents a controller stick event. This struct tracks both sticks at once.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ControllerStickEvent {
    /// The time at which the event occurred.
    pub time: f64,
    /// The x-axis value of the left stick.
    pub left_x: f32,
    /// The y-axis value of the left stick.
    pub left_y: f32,
    /// The x-axis value of the right stick.
    pub right_x: f32,
    /// The y-axis value of the right stick.
    pub right_y: f32,
}

/// Represents the state of a dual-stick controller.
#[derive(Debug, Default)]
struct ControllerStickState {
    /// The x-axis value of the stick.
    x: f32,
    /// The y-axis value of the stick.
    y: f32,
}

/// Opens a file dialog to the applications data folder.
/// If the folder doesn't exist, it defaults to the Documents folder.
///
/// If a file is selected, it returns the path to the file. Otherwise,
/// it returns None.
///
/// returns: `Option<PathBuf>`
pub fn open_dialog_in_data_folder() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_directory(get_exe_parent_dir().join("data"))
        .pick_file()
}

/// An enum representing a text state.
#[derive(Debug)]
pub enum TextState {
    Success,
    Error,
    Warning,
    Default,
}

/// Text that has a state associated with it.
///
/// By default, the colors are as follows:
///
/// * Success: green
/// * Error: red
/// * Warning: yellow
#[derive(Debug)]
pub struct StatefulText {
    /// The text to display.
    pub text: String,
    /// The state of the text.
    pub state: TextState,
    success_color: Color32,
    error_color: Color32,
    warning_color: Color32,
    default_color: Color32,
}

impl Default for StatefulText {
    fn default() -> Self {
        Self {
            text: String::default(),
            state: TextState::Default,
            success_color: Color32::GREEN,
            error_color: Color32::RED,
            warning_color: Color32::YELLOW,
            default_color: Color32::WHITE,
        }
    }
}

impl StatefulText {
    /// Creates a new `StatefulText` from some String and a `TextState`.
    ///
    /// # Arguments
    ///
    /// * `text`: The text to display.
    /// * `state`: The state of the text.
    ///
    /// returns: `StatefulText`
    pub fn new(text: String, state: TextState) -> Self {
        Self {
            text,
            state,
            success_color: Color32::GREEN,
            error_color: Color32::RED,
            warning_color: Color32::YELLOW,
            default_color: Color32::GRAY,
        }
    }

    /// Adds the text to the UI.
    ///
    /// # Arguments
    ///
    /// * `ui`: The UI to add the text to.
    pub fn show(&self, ui: &mut Ui) {
        let color = match self.state {
            TextState::Success => self.success_color,
            TextState::Error => self.error_color,
            TextState::Warning => self.warning_color,
            TextState::Default => self.default_color,
        };
        ui.colored_label(color, &self.text);
    }
}

#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter)]
pub enum ControllerType {
    Default = 1,
    Xbox = 2,
    PlayStation = 3,
}

impl Default for ControllerType {
    fn default() -> Self {
        Self::Default
    }
}

impl ControllerType {
    /// Returns the name of the button based on its `ControllerType`.
    ///
    /// # Arguments
    ///
    /// * `button`: The button to get the name of.
    ///
    /// returns: `String`
    pub fn get_button_name(&self, button: gilrs::Button) -> String {
        match self {
            ControllerType::Default => format!("{:?}", button),
            ControllerType::Xbox => get_xbox_button(button),
            ControllerType::PlayStation => get_playstation_button(button),
        }
    }
}

fn get_xbox_button(button: gilrs::Button) -> String {
    match button {
        gilrs::Button::South => "A".to_string(),
        gilrs::Button::East => "B".to_string(),
        gilrs::Button::North => "Y".to_string(),
        gilrs::Button::West => "X".to_string(),
        gilrs::Button::LeftTrigger => "LB".to_string(),
        gilrs::Button::LeftTrigger2 => "LT".to_string(),
        gilrs::Button::RightTrigger => "RB".to_string(),
        gilrs::Button::RightTrigger2 => "RT".to_string(),
        gilrs::Button::LeftThumb => "LS".to_string(),
        gilrs::Button::RightThumb => "RS".to_string(),
        _ => format!("{:?}", button),
    }
}

fn get_playstation_button(button: gilrs::Button) -> String {
    // TODO: use symbols
    match button {
        gilrs::Button::South => "X".to_string(),
        gilrs::Button::East => "O".to_string(),
        gilrs::Button::North => "Triangle".to_string(),
        gilrs::Button::West => "Square".to_string(),
        gilrs::Button::LeftTrigger => "L1".to_string(),
        gilrs::Button::LeftTrigger2 => "L2".to_string(),
        gilrs::Button::RightTrigger => "R1".to_string(),
        gilrs::Button::RightTrigger2 => "R2".to_string(),
        gilrs::Button::LeftThumb => "LS".to_string(),
        gilrs::Button::RightThumb => "RS".to_string(),
        _ => format!("{:?}", button),
    }
}
