use std::fs::File;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{io, thread};

use eframe::egui;
use human_panic::setup_panic;
use log::{debug, error, info, LevelFilter};
use simplelog::{Config, WriteLogger};
use xlogger::BoxedResult;

use crate::util::{create_dir_if_not_exists, get_exe_parent_dir};

mod util;

#[derive(Default)]
struct XloggerApp {
    should_run: Arc<AtomicBool>,
    saved_text: String,
}

impl eframe::App for XloggerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let should_run_value = self.should_run.load(Ordering::Relaxed);
            let text = if should_run_value { "Stop" } else { "Start" };
            ui.horizontal(|ui| {
                if ui.button(text).clicked() {
                    let (log_message, saved_text) = if should_run_value {
                        ("stopped listening to controllers", "Saved!".to_owned())
                    } else {
                        // also starts the event loop thread
                        let _closure = {
                            let should_run = self.should_run.clone();
                            thread::spawn(move || {
                                if let Err(e) = xlogger::listen_for_events(&should_run) {
                                    error!("{:?}", e);
                                    should_run.store(false, Ordering::Relaxed);
                                }
                            });
                        };
                        ("started listening to controllers", "".to_owned())
                    };
                    self.saved_text = saved_text;
                    info!("{}", log_message);
                    self.should_run.store(!should_run_value, Ordering::Relaxed);
                }
                ui.label(&self.saved_text);
            });
            if ui.button("Visualize").clicked() {
                // opens to the data folder
                // if it doesn't exist, RFD defaults to the Documents folder
                if let Some(path) = rfd::FileDialog::new()
                    .set_directory(get_exe_parent_dir().join("data"))
                    .pick_file()
                {
                    thread::spawn(move || match Self::visualize_button_data(path) {
                        Ok(exit_status) => {
                            info!("Visualization exited with status {}", exit_status)
                        }
                        Err(e) => error!("{:?}", e),
                    });
                }
            };
        });
    }
}

impl XloggerApp {
    /// Visualizes the data in the given file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to visualize.
    ///
    /// returns: `io::Result<ExitStatus>`
    fn visualize_button_data(path: PathBuf) -> io::Result<ExitStatus> {
        info!("visualizing data from {}", path.display());
        let visualize_script = get_exe_parent_dir().join("visualize").join("visualize");
        debug!("visualize script: {}", visualize_script.display());
        let mut child_proc = std::process::Command::new(&visualize_script)
            .arg(path)
            .spawn()?;
        let exit_status = child_proc.wait()?;
        if !exit_status.success() {
            error!(
                "Visualization script exited with non-zero status: {}",
                exit_status
            );
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Visualization script exited with non-zero status: {:?}",
                    exit_status
                ),
            ))
        } else {
            Ok(exit_status)
        }
    }
}

fn main() {
    // traditionally, this is used for CLI's
    // in the case that this GUI does crash, this
    // will auto-generate a log which is what I
    // care about
    setup_panic!(human_panic::Metadata {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "dablenparty".into(),
        homepage: "N/A".into(),
    });
    if let Err(e) = init_logger() {
        // do not allow the program to continue without logging
        panic!("Something went wrong initializing logging: {}", e);
    };
    let should_run = Arc::new(AtomicBool::new(false));

    let app = XloggerApp {
        should_run,
        ..Default::default()
    };
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
fn init_logger() -> BoxedResult<()> {
    let mut file_path = get_exe_parent_dir();
    let filename = chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d %H-%M-%S.log")
        .to_string();
    file_path.push("logs");
    create_dir_if_not_exists(&file_path)?;
    file_path.push(filename);
    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create(&file_path)?,
    )?;
    Ok(())
}
