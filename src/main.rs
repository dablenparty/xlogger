use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread;

use eframe::{egui, epi};
use gilrs::{EventType, Gamepad, GamepadId, Gilrs};

struct XloggerApp {
    should_run: Arc<AtomicBool>,
}

impl XloggerApp {
    fn new(should_run: Arc<AtomicBool>) -> Self {
        Self { should_run }
    }
}

impl epi::App for XloggerApp {
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let should_run_value = self.should_run.load(std::sync::atomic::Ordering::Relaxed);
            let text = if should_run_value { "Stop" } else { "Start" };
            if ui.button(text).clicked() {
                self.should_run
                    .store(!should_run_value, std::sync::atomic::Ordering::Relaxed);
            }
        });
    }

    fn name(&self) -> &str {
        "xlogger"
    }
}

fn main() {
    let should_run = Arc::new(AtomicBool::new(false));

    let _ = {
        let should_run = should_run.clone();
        thread::spawn(move || {
            let mut gilrs = Gilrs::new().unwrap();

            loop {
                while let Some(gilrs::Event { event, id, .. }) = gilrs.next_event() {
                    if !should_run.load(std::sync::atomic::Ordering::Relaxed) {
                        continue;
                    }
                    println!("{:?} {:?}", id, event);
                }
            }
        });
    };

    let app = XloggerApp::new(should_run);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
