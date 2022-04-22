use std::fs::File;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;

use eframe::{egui, epi};
use log::{info, LevelFilter};
use simplelog::{Config, WriteLogger};

use crate::util::{create_dir_if_not_exists, get_exe_parent_dir};

mod util;

struct XloggerApp {
    should_run: Arc<AtomicBool>,
}

impl epi::App for XloggerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let should_run_value = self.should_run.load(std::sync::atomic::Ordering::Relaxed);
            let text = if should_run_value { "Stop" } else { "Start" };
            if ui.button(text).clicked() {
                let log_message = if should_run_value {
                    "stopped listening to controllers"
                } else {
                    "started listening to controllers"
                };
                info!("{}", log_message);
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
    init_logger();
    let should_run = Arc::new(AtomicBool::new(false));

    let _ = {
        let should_run = should_run.clone();
        thread::spawn(move || xlogger::run_event_loop(should_run));
    };

    let app = XloggerApp { should_run };
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
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
    create_dir_if_not_exists(&file_path);
    file_path.push(filename);
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create(&file_path).unwrap_or_else(|e| {
            eprintln!("Failed to create log file: {:?}", file_path);
            eprintln!("{:?}", e);
            std::process::exit(1);
        }),
    )
    .expect("Failed to initialize logger");
}
