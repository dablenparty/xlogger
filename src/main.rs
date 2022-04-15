use gilrs::{Event, EventType, Gilrs};

fn main() {
    let mut gilrs = Gilrs::new().unwrap();

    loop {
        while let Some(Event { event, time: event_time, .. }) = gilrs.next_event() {
            if let EventType::ButtonChanged(button, ..) = event {
                println!("{:?} {:?}", button, event_time);
            }
        }
    }
}
