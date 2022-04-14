use gilrs::{Event, EventType, Gilrs};

fn main() {
    let mut gilrs = Gilrs::new().unwrap();

    loop {
        while let Some(Event { event, time: event_time, .. }) = gilrs.next_event() {
            if let EventType::ButtonChanged(_, _, _) = event {
                println!("{:?} {:?}", event, event_time);
            }
        }
    }
}
