use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, Arc};
use std::time::SystemTime;

use eframe::egui::{Context, Ui};
use eframe::epaint::Color32;
use gilrs::{Axis, Gilrs};
use log::{error, warn};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::util::{create_dir_if_not_exists, get_exe_parent_dir};

pub mod button_graph;
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

/// Starts an event loop that listens for controller events and writes them to a file.
///
/// This function is intended to run on a separate thread.
///
/// # Arguments
///
/// * `should_run`: Thread-safe boolean value that determines whether the event loop should continue
///
/// # Errors
///
/// This function will error if something goes wrong creating the CSV files or writing to them.
///
/// returns: ()
pub fn listen_for_events(should_run: &Arc<AtomicBool>) -> io::Result<()> {
    // if this fails, the event loop can never run
    let mut gilrs = Gilrs::new().expect("failed to initialize controller processor");

    let data_folder = get_exe_parent_dir().join("data");
    create_dir_if_not_exists(&data_folder)?;
    let timestamp_string = chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d_%H-%M-%S.csv")
        .to_string();

    // csv file paths
    let button_csv_path = data_folder.join("buttons_".to_owned() + &timestamp_string);
    let stick_csv_path = data_folder.join("sticks_".to_owned() + &timestamp_string);

    // csv writers
    let mut button_csv_writer = csv::Writer::from_path(button_csv_path)?;
    let mut stick_csv_writer = csv::Writer::from_path(stick_csv_path)?;

    // time map
    let mut time_map: HashMap<String, SystemTime> = HashMap::new();

    // stick state
    let mut left_stick_state = ControllerStickState::default();
    let mut right_stick_state = ControllerStickState::default();

    let start_time = SystemTime::now();

    while should_run.load(std::sync::atomic::Ordering::Relaxed) {
        while let Some(gilrs::Event {
            event,
            time: event_time,
            ..
        }) = gilrs.next_event()
        {
            match event {
                gilrs::EventType::AxisChanged(axis, value, ..) => {
                    match axis {
                        Axis::LeftStickX => left_stick_state.x = value,
                        Axis::LeftStickY => left_stick_state.y = value,
                        Axis::RightStickX => right_stick_state.x = value,
                        Axis::RightStickY => right_stick_state.y = value,
                        _ => {
                            warn!("unhandled axis event: {:?}", event);
                        }
                    }
                    let stick_event = ControllerStickEvent {
                        time: event_time
                            .duration_since(start_time)
                            .expect("time went backwards!")
                            .as_secs_f64(),
                        left_x: left_stick_state.x,
                        left_y: left_stick_state.y,
                        right_x: right_stick_state.x,
                        right_y: right_stick_state.y,
                    };
                    if let Err(e) = stick_csv_writer.serialize(&stick_event) {
                        error!(
                            "failed to write stick event <{:?}> to csv with following error: {:?}",
                            stick_event, e
                        );
                    }
                    stick_csv_writer.flush()?;
                }
                gilrs::EventType::ButtonChanged(button, value, ..) => {
                    let name = format!("{:?}", button);

                    if value == 0.0 {
                        let down_time = time_map.remove(&name).unwrap_or_else(SystemTime::now);
                        let button_event = ControllerButtonEvent {
                            press_time: down_time
                                .duration_since(start_time)
                                .expect("time went backwards!")
                                .as_secs_f64(),
                            release_time: event_time
                                .duration_since(start_time)
                                .expect("time went backwards!")
                                .as_secs_f64(),
                            button,
                        };
                        if let Err(e) = button_csv_writer.serialize(&button_event) {
                            error!(
                                "failed to write button event <{:?}> to csv with following error: {:?}",
                                button_event, e
                            );
                        }
                        button_csv_writer.flush()?;
                    } else {
                        // only insert if it doesn't have a value (aka has the default value)
                        let map_time_opt = time_map.get(&name);
                        if map_time_opt.unwrap_or(&SystemTime::UNIX_EPOCH)
                            == &SystemTime::UNIX_EPOCH
                        {
                            time_map.insert(name, event_time);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
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
