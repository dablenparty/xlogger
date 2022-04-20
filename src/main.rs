use gilrs::{Event, EventType, Gilrs};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ControllerEvent {
    press_time: f64,
    release_time: f64,
    button: String,
}

fn get_data_folder() -> PathBuf {
    std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("."))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("data")
}

fn main() {
    let filename = chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d_%H-%M-%S.csv")
        .to_string();

    // get the data folder and create it, ignoring the error if it already exists
    let mut csv_path = get_data_folder();
    fs::create_dir(&csv_path).unwrap_or_else(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            return;
        }
        eprintln!("{:?}", e);
        std::process::exit(1);
    });
    // append the filename to the data folder to get the full path
    csv_path.push(filename);

    let mut time_map: HashMap<String, SystemTime> = HashMap::new();
    let csv_lock = Arc::new(Mutex::new(csv::Writer::from_writer(
        File::create(&csv_path).unwrap(),
    )));

    let _ = {
        // clone the writer and path into the closure
        let csv_lock = csv_lock.clone();
        let csv_path = csv_path.clone();
        ctrlc::set_handler(move || {
            println!("received ctrl-c");
            // this ensures that the writer has been properly flushed before the program exits
            csv_lock
                .lock()
                .unwrap_or_else(|_| {
                    eprintln!("failed to lock csv_writer");
                    std::process::exit(1);
                })
                .flush()
                .unwrap();
            // pass the data file to the visualize script
            let visualize_script = std::env::current_exe()
                .unwrap_or_else(|_| PathBuf::from("."))
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("visualize")
                .join("visualize");
            // spawn the visualize script and wait for it to finish
            let mut proc_handle = std::process::Command::new(visualize_script)
                .arg(&csv_path)
                .spawn()
                .unwrap_or_else(|_| {
                    eprintln!("failed to spawn visualize script");
                    std::process::exit(1);
                });
            // wait for the process to finish
            let exit_status = proc_handle.wait().unwrap_or_else(|_| {
                eprintln!("failed to wait for visualize script");
                std::process::exit(1);
            });

            if !exit_status.success() {
                eprintln!("visualize script exited with non-zero status");
                std::process::exit(1);
            }

            std::process::exit(0);
        })
        .expect("Error setting the Ctrl-C handler");
    };

    let mut gilrs = Gilrs::new().unwrap();

    println!("data file: {:?}", csv_path);
    println!("At any time, click into this window and press Ctrl-C to exit this program smoothly");

    loop {
        while let Some(Event {
            event,
            time: event_time,
            ..
        }) = gilrs.next_event()
        {
            if let EventType::ButtonChanged(button, value, ..) = event {
                let name = format!("{:?}", button);
                if value == 0.0 {
                    let down_time = time_map.remove(&name).unwrap();
                    let duration = event_time.duration_since(down_time).unwrap();
                    time_map.insert(name.clone(), SystemTime::UNIX_EPOCH);
                    println!("{} released after {:?}", name, duration);
                    // this should be ok since the lock will always be acquired by this thread
                    // the only time it could be acquired by another thread is if the program
                    // is exiting, in which case the lock will be dropped and the writer will
                    // be flushed
                    let mut csv_writer = csv_lock.lock().unwrap_or_else(|_| {
                        eprintln!("failed to lock csv_writer");
                        std::process::exit(1);
                    });
                    csv_writer
                        .serialize(ControllerEvent {
                            press_time: down_time
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64(),
                            release_time: event_time
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64(),
                            button: name,
                        })
                        .unwrap();
                    csv_writer.flush().unwrap();
                } else {
                    let map_time_opt = time_map.get(&name);
                    if map_time_opt.unwrap_or(&SystemTime::UNIX_EPOCH) == &SystemTime::UNIX_EPOCH {
                        time_map.insert(name.clone(), event_time);
                        println!("{} pressed", name);
                    }
                }
            }
        }
    }
}
