use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;

use eframe::egui;
use log::{error, info, LevelFilter};
use simplelog::{Config, WriteLogger};

use crate::util::{create_dir_if_not_exists, get_exe_parent_dir};

mod util;

struct XloggerApp {
    should_run: Arc<AtomicBool>,
}

impl eframe::App for XloggerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                let _closure = {
                    let should_run = self.should_run.clone();
                    thread::spawn(move || xlogger::listen_for_events(should_run));
                };
            }
            if ui.button("Visualize").clicked() {
                // opens to the data folder
                // if it doesn't exist, RFD defaults to the Documents folder
                if let Some(path) = rfd::FileDialog::new()
                    .set_directory(get_exe_parent_dir().join("data"))
                    .pick_file()
                {
                    Self::visualize_data(path);
                }
            }
        });
    }
}

impl XloggerApp {
    fn visualize_data(path: PathBuf) {
        info!("visualizing data from {}", path.display());
        let visualize_script = get_exe_parent_dir().join("visualize").join("visualize");
        let proc_result = std::process::Command::new(visualize_script)
            .arg(path)
            .spawn();
        match proc_result {
            Ok(mut child) => {
                if let Ok(exit_status) = child.wait() {
                    if !exit_status.success() {
                        error!(
                            "Visualization script exited with non-zero status: {:?}",
                            exit_status
                        );
                    }
                } else {
                    error!("Process never started");
                }
            }
            Err(e) => {
                error!("Failed to spawn 'visualize' process: {}", e);
            }
        };
    }
}

fn main() {
    init_logger();
    let should_run = Arc::new(AtomicBool::new(false));

    let app = XloggerApp { should_run };
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "xlogger",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(app)
        }),
    );
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
