use std::collections::HashMap;
use std::sync::{atomic::AtomicBool, Arc};
use std::time::SystemTime;

use gilrs::{Axis, Gilrs};
use log::{error, warn};
use serde::Serialize;

use crate::util::{create_dir_if_not_exists, get_exe_parent_dir};

mod util;

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ControllerButtonEvent {
    press_time: f64,
    release_time: f64,
    button: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ControllerStickEvent {
    time: f64,
    left_x: f32,
    left_y: f32,
    right_x: f32,
    right_y: f32,
}

#[derive(Debug, Default)]
struct ControllerStickState {
    x: f32,
    y: f32,
}

/// Runs the main event loop for xlogger. This function is designed to be run on a separate thread.
///
/// This entails listening for events from the gamepads and logging them to the CSV data file.
///
/// # Arguments
///
/// * `should_run`: A thread-safe boolean that is set to false when the event loop should stop.
///
/// returns: ()
pub fn run_event_loop(should_run: Arc<AtomicBool>) {
    let mut gilrs = Gilrs::new().unwrap();

    loop {
        while let Some(gilrs::Event { event, id, .. }) = gilrs.next_event() {
            if !should_run.load(std::sync::atomic::Ordering::Relaxed) {
                continue;
            }
            println!("{:?} {:?}", id, event);
        }
    }
}

pub fn listen_for_events(should_run: Arc<AtomicBool>) {
    // TODO: return a Result so the errors can be handled externally
    let mut gilrs = Gilrs::new().expect("failed to initialize controller processor");

    let data_folder = get_exe_parent_dir().join("data");
    create_dir_if_not_exists(&data_folder);
    let timestamp_string = chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d_%H-%M-%S.csv")
        .to_string();

    // csv file paths
    let button_csv_path = data_folder.join("buttons_".to_owned() + &timestamp_string);
    let stick_csv_path = data_folder.join("sticks_".to_owned() + &timestamp_string);

    // csv writers
    let mut button_csv_writer =
        csv::Writer::from_path(button_csv_path).expect("failed to create button csv writer");
    let mut stick_csv_writer =
        csv::Writer::from_path(stick_csv_path).expect("failed to create stick csv writer");

    // time map
    let mut time_map: HashMap<String, SystemTime> = HashMap::new();

    // stick state
    let mut left_stick_state = ControllerStickState::default();
    let mut right_stick_state = ControllerStickState::default();

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
                    if let Err(e) = stick_csv_writer.serialize(ControllerStickEvent {
                        time: event_time
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .expect("time went backwards!")
                            .as_secs_f64(),
                        left_x: left_stick_state.x,
                        left_y: left_stick_state.y,
                        right_x: right_stick_state.x,
                        right_y: right_stick_state.y,
                    }) {
                        error!(
                            "failed to write stick event to csv with following error: {:?}",
                            e
                        );
                    }
                }
                gilrs::EventType::ButtonChanged(button, value, ..) => {
                    let name = format!("{:?}", button);

                    if value == 0.0 {
                        let now = &SystemTime::now();
                        let down_time = time_map.get(&name).unwrap_or(now);
                        button_csv_writer
                            .serialize(ControllerButtonEvent {
                                press_time: down_time
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .expect("time went backwards!")
                                    .as_secs_f64(),
                                release_time: event_time
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .expect("time went backwards!")
                                    .as_secs_f64(),
                                button: name.clone(),
                            })
                            .unwrap_or_else(|e| {
                                error!(
                                "failed to write button event to csv with following error: {:?}",
                                e
                            );
                            });
                        time_map.insert(name, event_time);
                        button_csv_writer.flush().unwrap_or_else(|e| {
                            error!(
                                "failed to flush button csv writer with following error: {:?}",
                                e
                            )
                        });
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
}