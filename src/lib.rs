use std::sync::{Arc, atomic::AtomicBool};

use gilrs::Gilrs;

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
