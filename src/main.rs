use gilrs::{Event, EventType, Gilrs};

fn main() {
    let mut gilrs = Gilrs::new().unwrap();

    loop {
        while let Some(Event { event, time: event_time, ..}) = gilrs.next_event() {
            match event {
                EventType::AxisChanged(_, _, _) => {}
                _ => {
                    println!("{:?} {:?}", event, event_time);
                }
            }
        }
    }
}
