use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::SystemTime;

use gilrs::{Axis, Event, EventType, Gilrs};
use log::{debug, info, warn};
use serde::Serialize;
use simplelog::*;

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

struct ControllerStickState {
    x: f32,
    y: f32,
}

fn main() {
    init_logger();

    debug!("creating csv path");
    let filename = chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d_%H-%M-%S.csv")
        .to_string();

    // get the data folder and create it, ignoring the error if it already exists
    let csv_path = get_data_folder();
    create_dir_if_not_exists(&csv_path);
    // append the filename to the data folder to get the full path
    let button_file_path = csv_path.join("buttons_".to_owned() + &filename);
    let stick_file_path = csv_path.join("sticks_".to_owned() + &filename);

    let button_csv_lock = Arc::new(Mutex::new(csv::Writer::from_writer(
        File::create(&button_file_path).unwrap(),
    )));
    let stick_csv_lock = Arc::new(Mutex::new(csv::Writer::from_writer(
        File::create(&stick_file_path).unwrap(),
    )));

    debug!("setting exit handler");
    let _ = {
        // clone the writer and path into the closure
        let button_csv_lock = button_csv_lock.clone();
        let stick_csv_lock = stick_csv_lock.clone();
        let button_file_path = button_file_path.clone();
        ctrlc::set_handler(move || {
            info!("received ctrl-c");
            let button_csv_writer = button_csv_lock.lock().unwrap_or_else(|_| {
                error!("failed to lock button_csv_writer");
                std::process::exit(1);
            });
            let stick_csv_writer = stick_csv_lock.lock().unwrap_or_else(|_| {
                error!("failed to lock stick_csv_writer");
                std::process::exit(1);
            });
            exit_handler(button_csv_writer, stick_csv_writer, &button_file_path);
        })
        .expect("Error setting the Ctrl-C handler");
    };

    info!("creating gilrs (controller listener)");
    // create the controller listener
    let mut gilrs = Gilrs::new().expect("failed to create gilrs");
    // this map will be used to store the last time a button was pressed
    let mut time_map: HashMap<String, SystemTime> = HashMap::new();
    // create the stick state structs
    let mut left_stick_state = ControllerStickState { x: 0.0, y: 0.0 };
    let mut right_stick_state = ControllerStickState { x: 0.0, y: 0.0 };

    println!("data file: {:?}", csv_path);
    println!("At any time, click into this window and press Ctrl-C to exit this program smoothly");

    loop {
        while let Some(Event {
            event,
            time: event_time,
            ..
        }) = gilrs.next_event()
        {
            match event {
                EventType::AxisChanged(axis, value, ..) => {
                    // since there are SO MANY stick events, we log it at debug level only
                    debug!("axis changed: {:?} {:?}", axis, value);
                    match axis {
                        Axis::LeftStickX => left_stick_state.x = value,
                        Axis::LeftStickY => left_stick_state.y = value,
                        Axis::RightStickX => right_stick_state.x = value,
                        Axis::RightStickY => right_stick_state.y = value,
                        _ => {
                            warn!("unknown axis: {:?}", axis);
                        }
                    }
                    // log the event to the csv file
                    let mut csv_writer = stick_csv_lock.lock().unwrap_or_else(|_| {
                        error!("failed to lock csv_writer");
                        std::process::exit(1);
                    });
                    debug!("writing stick event to csv");
                    csv_writer
                        .serialize(ControllerStickEvent {
                            time: event_time
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64(),
                            left_x: left_stick_state.x,
                            left_y: left_stick_state.y,
                            right_x: right_stick_state.x,
                            right_y: right_stick_state.y,
                        })
                        .expect("failed to serialize csv");
                }
                EventType::ButtonChanged(button, value, ..) => {
                    debug!("matched event: {:?}", event);
                    let name = format!("{:?}", button);
                    // value == 0 means the button was released
                    if value == 0.0 {
                        // tracking how long a button is "held down" is done by subtracting the time the
                        // button was pressed from this (the time the button was released)
                        let down_time = time_map
                            .remove(&name)
                            .expect("a button was released without being pressed!");
                        let duration = event_time
                            .duration_since(down_time)
                            .expect("time went backwards");
                        // resets this key in the time map to the unix epoch as a placeholder for the
                        // next button press
                        time_map.insert(name.clone(), SystemTime::UNIX_EPOCH);
                        info!("button {} was released after {:?}", name, duration);
                        // this should be ok since the lock will always be acquired by this thread
                        // the only time it could be acquired by another thread is if the program
                        // is exiting, in which case the lock will be dropped and the writer will
                        // be flushed
                        let mut csv_writer = button_csv_lock.lock().unwrap_or_else(|_| {
                            error!("failed to lock csv_writer");
                            std::process::exit(1);
                        });
                        let record = ControllerButtonEvent {
                            press_time: down_time
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64(),
                            release_time: event_time
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64(),
                            button: name,
                        };
                        debug!("writing following to csv: {:?}", record);
                        csv_writer.serialize(record).unwrap();
                        csv_writer.flush().unwrap();
                    } else {
                        // value != 0 means the button was pressed (or is still pressed)
                        // if this is the first time the button was pressed, record the time
                        let map_time_opt = time_map.get(&name);
                        if map_time_opt.unwrap_or(&SystemTime::UNIX_EPOCH)
                            == &SystemTime::UNIX_EPOCH
                        {
                            time_map.insert(name.clone(), event_time);
                            debug!("{} pressed", name);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Initializes the logging library
///
/// The current implementation outputs `warn` and above to the console and `debug` and above to
/// a file.
fn init_logger() {
    let mut file_path = get_exe_parent_dir();
    let filename = chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d %H-%M-%S.log")
        .to_string();
    file_path.push("logs");
    create_dir_if_not_exists(&mut file_path);
    file_path.push(filename);
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create(&file_path)
                .expect(format!("Failed to create log file: {:?}", file_path).as_str()),
        ),
    ])
    .unwrap();
}

/// Creates a directory if it does not exist, failing if some other error occurs
///
/// # Arguments
///
/// * `file_path`: the path to the directory
///
/// returns: ()
fn create_dir_if_not_exists(file_path: &PathBuf) {
    fs::create_dir(&file_path).unwrap_or_else(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            return;
        }
        // initialization of the logging library uses this function, so we can't use the error!
        // macro here
        eprintln!("{:?}", e);
        std::process::exit(1);
    });
}

/// Gets the path to the directory containing the executable
///
/// returns: PathBuf
fn get_exe_parent_dir() -> PathBuf {
    std::env::current_exe()
        .unwrap_or_else(|_| {
            warn!("failed to get current executable path");
            PathBuf::from("./xlogger.exe")
        })
        .parent()
        .unwrap_or_else(|| {
            warn!("failed to get parent of executable");
            Path::new(".")
        }).to_path_buf()
}

/// Get the data folder. It resides alongside the executable.
///
/// For example, if the executable is at `/path/to/executable`, the data folder is `/path/to/data`.
///
/// returns: PathBuf
fn get_data_folder() -> PathBuf {
    let mut folder = get_exe_parent_dir();
    folder.push("data");
    folder
}

/// The exit handler for the program. This is designed to be passed to the `ctrlc::set_handler`
/// function.
///
/// The writer guards passed in here are dropped when the function returns and are used to ensure
/// that the writers are flushed before the program exits.
///
/// # Arguments
///
/// * `button_writer_guard`: A mutex guard on the csv writer for the buttons file.
///
/// * `stick_writer_guard`: A mutex guard on the csv writer for the sticks file.
///
/// * `csv_path`: The path to the csv file.
///
/// returns: ()
fn exit_handler(
    mut button_writer_guard: MutexGuard<csv::Writer<File>>,
    mut stick_writer_guard: MutexGuard<csv::Writer<File>>,
    csv_path: &PathBuf,
) {
    // this ensures that the writers have been properly flushed before the program exits
    debug!("flushing csv writers");
    button_writer_guard.flush().unwrap();
    stick_writer_guard.flush().unwrap();
    // pass the button data file to the visualize script
    let mut visualize_script = get_exe_parent_dir();
    visualize_script.push("visualize");
    visualize_script.push("visualize");
    info!("launching visualization script");
    debug!("visualize path: {:?}", visualize_script);
    // spawn the visualize script and wait for it to finish
    let mut proc_handle = std::process::Command::new(visualize_script)
        .arg(&csv_path)
        .spawn()
        .unwrap_or_else(|_| {
            error!("failed to spawn visualize script");
            std::process::exit(1);
        });
    // wait for the process to finish
    let exit_status = proc_handle.wait().unwrap_or_else(|_| {
        error!("visualize script never started");
        std::process::exit(1);
    });

    if !exit_status.success() {
        error!("visualize script didn't finish successfully");
        std::process::exit(1);
    }

    info!("visualization script finished successfully");
    std::process::exit(0);
}
