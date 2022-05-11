#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{io, thread};

use eframe::egui::plot::{Legend, Line, Plot, Points, Value, Values};
use eframe::egui::{self, Slider, Ui};
use eframe::IconData;
use human_panic::setup_panic;
use image::ImageResult;
use log::{debug, error, info, LevelFilter};
use simplelog::{Config, WriteLogger};
use xlogger::{open_dialog_in_data_folder, BoxedResult, ControllerStickEvent, StatefulText};

use crate::util::{create_dir_if_not_exists, get_exe_parent_dir};

mod util;

#[derive(Clone)]
struct ControllerCsvData {
    left_values: Vec<Value>,
    right_values: Vec<Value>,
}

#[derive(Default)]
struct XloggerApp {
    should_run: Arc<AtomicBool>,
    saved_text: StatefulText,
    show_stick_window: bool,
    stick_csv_data: Option<ControllerCsvData>,
    visualize_path: Option<PathBuf>,
    slider_timestamp: usize,
    show_stick_lines: bool,
    stick_data_offset: u8,
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
                        {
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
                    self.saved_text.text = saved_text;
                    info!("{}", log_message);
                    self.should_run.store(!should_run_value, Ordering::Relaxed);
                }
                self.saved_text.show(ui);
            });
            ui.horizontal(|ui| {
                if ui.button("Visualize Sticks").clicked() {
                    // opens to the data folder
                    // if it doesn't exist, RFD defaults to the Documents folder
                    if let Some(path) = open_dialog_in_data_folder() {
                        self.show_stick_window = true;
                        self.stick_csv_data = None;
                        self.visualize_path = Some(path);
                    }
                };
                if ui.button("Visualize Buttons").clicked() {
                    if let Some(path) = open_dialog_in_data_folder() {
                        thread::spawn(move || match Self::visualize_button_data(path) {
                            Ok(exit_status) => {
                                info!("Visualization exited with status {}", exit_status);
                            }
                            Err(e) => error!("{:?}", e),
                        });
                    }
                }
            });
            // show sticks plot or handle error
            if let Err(e) = self.visualize_stick_data(ui) {
                error!(
                    "Something went wrong deserializing data at {:#?}:",
                    self.visualize_path
                );
                error!("{}", e);
                self.visualize_path = None;
                self.stick_csv_data = None;
                self.show_stick_window = false;
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
        if exit_status.success() {
            Ok(exit_status)
        } else {
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
        }
    }

    /// Visualizes the stick data in the given file.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI to draw to.
    ///
    /// # Errors
    ///
    /// This function errors if there is an issue reading the CSV file defined by `self.visualize_path`
    /// or if there is an issue deserializing the data such as malformed, missing, or extra columns.
    ///
    /// returns: `Result<Option<egui::Response>, Box<dyn Error>>`
    fn visualize_stick_data(
        &mut self,
        ui: &mut Ui,
    ) -> Result<Option<egui::Response>, Box<dyn Error>> {
        if self.visualize_path.is_none() {
            return Ok(None);
        }
        // at this point, we know it's not None
        let path = self.visualize_path.as_ref().unwrap();
        // try to get the cached CSV data. if it doesn't exist, read it from the file
        let (ls_events, rs_events) = if let Some(data) = &self.stick_csv_data {
            let data = data.clone();
            (data.left_values, data.right_values)
        } else {
            let (left_values, right_values) = csv::Reader::from_path(path)?
                .deserialize::<ControllerStickEvent>()
                .try_fold::<_, _, Result<(Vec<Value>, Vec<Value>), Box<dyn Error>>>(
                    (Vec::new(), Vec::new()),
                    |mut acc, result| {
                        let event = result?;
                        acc.0.push(Value::new(event.left_x, event.left_y));
                        acc.1.push(Value::new(event.right_x, event.right_y));
                        Ok((acc.0, acc.1))
                    },
                )?;
            let data = ControllerCsvData {
                left_values,
                right_values,
            };
            self.stick_csv_data = Some(data.clone());
            (data.left_values, data.right_values)
        };
        let ls_sliced = &ls_events[self
            .slider_timestamp
            .saturating_sub(self.stick_data_offset.into())
            ..self.slider_timestamp];
        let ls_values = Values::from_values(ls_sliced.to_vec());

        let rs_sliced = &rs_events[self
            .slider_timestamp
            .saturating_sub(self.stick_data_offset.into())
            ..self.slider_timestamp];
        // this moves the points to the right so that this data is not on top of the previous data
        let translated_vec = rs_sliced
            .iter()
            .map(|element| Value::new(element.x + 2.5, element.y))
            .collect::<Vec<Value>>();
        let rs_values = Values::from_values(translated_vec);
        ui.horizontal(|ui| {
            ui.label("Time");
            // usize should always convert to u64
            ui.add(Slider::new(&mut self.slider_timestamp, 0..=ls_events.len()));
            ui.checkbox(&mut self.show_stick_lines, "Show lines");
            if ls_events.len() == usize::MAX {
                ui.label("Warning: too much data to visualize! not all of it will be shown");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Number of points displayed");
            ui.add(Slider::new(&mut self.stick_data_offset, u8::MIN..=u8::MAX))
                .on_hover_text("Higher values may cause performance issues");
        });
        Ok(Some(
            Plot::new("Stick Data")
                .data_aspect(1.0)
                .legend(Legend::default())
                .show(ui, |plot_ui| {
                    let point_radius = 1.0;

                    if self.show_stick_lines {
                        plot_ui.line(Line::new(ls_values).name("Left Stick"));
                        plot_ui.line(Line::new(rs_values).name("Right Stick"));
                    } else {
                        plot_ui.points(
                            Points::new(ls_values)
                                .radius(point_radius)
                                .name("Left Stick"),
                        );
                        plot_ui.points(
                            Points::new(rs_values)
                                .radius(point_radius)
                                .name("Right Stick"),
                        );
                    }
                })
                .response,
        ))
    }
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

fn get_icon_data() -> ImageResult<IconData> {
    let path = concat!(env!("OUT_DIR"), "/icon.ico");
    let icon = image::open(path)?.to_rgba8();
    let (width, height) = icon.dimensions();

    Ok(IconData {
        width,
        height,
        rgba: icon.into_raw(),
    })
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
        stick_data_offset: 50,
        ..XloggerApp::default()
    };
    let native_options = match get_icon_data() {
        Ok(icon_data) => eframe::NativeOptions {
            icon_data: Some(icon_data),
            ..eframe::NativeOptions::default()
        },

        Err(e) => {
            error!("Failed to load icon with error: {}", e);
            eframe::NativeOptions::default()
        }
    };
    eframe::run_native(
        "xlogger",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(app)
        }),
    );
}
