use std::fs::File;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{io, thread};

use eframe::egui::plot::{Legend, Plot, Points, Value, Values};
use eframe::egui::{self, Ui};
use human_panic::setup_panic;
use log::{debug, error, info, warn, LevelFilter};
use simplelog::{Config, WriteLogger};
use xlogger::{open_dialog_in_data_folder, BoxedResult, ControllerStickEvent};

use crate::util::{create_dir_if_not_exists, get_exe_parent_dir};

mod util;

#[derive(Default)]
struct XloggerApp {
    should_run: Arc<AtomicBool>,
    saved_text: String,
    show_stick_window: bool,
    visualize_path: Option<PathBuf>,
    slider_timestamp: u64,
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
            ui.horizontal(|ui| {
                if ui.button("Visualize Sticks").clicked() {
                    // opens to the data folder
                    // if it doesn't exist, RFD defaults to the Documents folder
                    if let Some(path) = open_dialog_in_data_folder() {
                        self.show_stick_window = true;
                        self.visualize_path = Some(path);
                    }
                };
                if ui.button("Visualize Buttons").clicked() {
                    if let Some(path) = open_dialog_in_data_folder() {
                        thread::spawn(move || match Self::visualize_button_data(path) {
                            Ok(exit_status) => {
                                info!("Visualization exited with status {}", exit_status)
                            }
                            Err(e) => error!("{:?}", e),
                        });
                    }
                }
            });
            // show sticks plot or handle error
            if let Err(e) = self.visualize_stick_data(ui) {
                error!("{:?}", e);
            }
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

    fn visualize_stick_data(&mut self, ui: &mut Ui) -> io::Result<Option<egui::Response>> {
        if self.visualize_path.is_none() {
            return Ok(None);
        }
        // at this point, we know it's not None
        let path = self.visualize_path.as_ref().unwrap();
        let (ls_events, rs_events) = csv::Reader::from_path(path)?
            .deserialize::<ControllerStickEvent>()
            .filter_map(|result| match result {
                Ok(event) => Some(event),
                Err(e) => {
                    warn!("A CSV record threw an error: {:?}", e);
                    None
                }
            })
            .fold((Vec::new(), Vec::new()), |mut acc, element| {
                acc.0.push(Value::new(element.left_x, element.left_y));
                acc.1.push(Value::new(element.right_x, element.right_y));
                (acc.0, acc.1)
            });
        // TODO: add a slider for this offset (and a warning about performance)
        let forward_offset = 200;
        let point_radius = 1.0;

        let ls_sliced = &ls_events[self.slider_timestamp as usize
            ..(self.slider_timestamp as usize + forward_offset).min(ls_events.len())];
        let ls_points = Points::new(Values::from_values(ls_sliced.to_vec())).radius(point_radius);

        let rs_sliced = &rs_events[self.slider_timestamp as usize
            ..(self.slider_timestamp as usize + forward_offset).min(rs_events.len())];
        // this moves the points to the right so that this data is not on top of the previous data
        let translated_vec = rs_sliced
            .iter()
            .map(|element| Value::new(element.x + 2.5, element.y))
            .collect::<Vec<Value>>();
        let rs_points = Points::new(Values::from_values(translated_vec)).radius(point_radius);
        ui.add(egui::Slider::new(
            &mut self.slider_timestamp,
            0..=(ls_events.len() - forward_offset).try_into().unwrap(),
        ));
        // TODO: add a line toggle
        Ok(Some(
            Plot::new("Stick Data")
                .data_aspect(1.0)
                .legend(Legend::default())
                .show(ui, |plot_ui| {
                    plot_ui.points(ls_points.name("Left Stick"));
                    plot_ui.points(rs_points.name("Right Stick"));
                })
                .response,
        ))
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
