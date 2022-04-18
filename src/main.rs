use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::sync::mpsc::channel;
use std::time::SystemTime;
use gilrs::{Event, EventType, Gilrs};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ControllerEvent {
    press_time: f64,
    release_time: f64,
    button: String,
}

fn main() {
    let mut gilrs = Gilrs::new().unwrap();

    let mut time_map: HashMap<String, SystemTime> = HashMap::new();
    let mut csv_writer = csv::Writer::from_writer(File::create("TEST.csv").unwrap());

    let (ctrlc_tx, ctrlc_rx) = channel();
    ctrlc::set_handler(move || {
        ctrlc_tx.send(()).expect("Could not send signal on channel");
    }).expect("Error setting the Ctrl-C handler");


    loop {
        ctrlc_rx.recv_timeout(std::time::Duration::from_millis(1)).and_then(|_| {
            println!("received ctrl-c");
            csv_writer.flush().unwrap();
            std::process::exit(0);
        }).unwrap_or(());
        while let Some(Event { event, time: event_time, .. }) = gilrs.next_event() {
            if let EventType::ButtonChanged(button, value, ..) = event {
                let name = format!("{:?}", button);
                if value == 0.0 {
                    let down_time = time_map.remove(&name).unwrap();
                    let duration = event_time.duration_since(down_time).unwrap();
                    time_map.insert(name.clone(), SystemTime::UNIX_EPOCH);
                    println!("{} released after {:?}", name, duration);
                    csv_writer.serialize(ControllerEvent {
                        press_time: down_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64(),
                        release_time: event_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64(),
                        button: name,
                    }).unwrap();
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
