use std::collections::HashMap;
use std::fmt::Debug;
use std::time::SystemTime;
use gilrs::{Button, Event, EventType, Gilrs};

fn main() {
    let mut gilrs = Gilrs::new().unwrap();

    let mut time_map: HashMap<String, SystemTime> = HashMap::new();

    loop {
        while let Some(Event { event, time: event_time, .. }) = gilrs.next_event() {
            if let EventType::ButtonChanged(button, value, ..) = event {
                let name = format!("{:?}", button);
                if value == 0.0 {
                    let down_time = time_map.remove(&name).unwrap();
                    let duration = event_time.duration_since(down_time).unwrap();
                    println!("{} released after {:?}", name, duration);
                    time_map.insert(name, SystemTime::UNIX_EPOCH);
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
