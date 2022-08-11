#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use eframe::egui;
use eframe::IconData;
use human_panic::setup_panic;
use image::ImageResult;
use log::{error, info, LevelFilter};
use simplelog::{Config, WriteLogger};
use xlogger::button_graph::ControllerButtonGraph;
use xlogger::stick_graph::ControllerStickGraph;
use xlogger::util::{create_dir_if_not_exists, get_exe_parent_dir};
use xlogger::{open_dialog_in_data_folder, BoxedResult, StatefulText};

#[derive(Default)]
struct XloggerApp {
    should_run: Arc<AtomicBool>,
    saved_text: StatefulText,
    stick_graphs: Vec<(bool, ControllerStickGraph)>,
    button_graphs: Vec<(bool, ControllerButtonGraph)>,
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
                // TODO: find a way to display errors back to the user
                // TODO: add a button to show/hide the stick window
                if ui.button("Visualize Sticks").clicked() {
                    // opens to the data folder
                    // if it doesn't exist, RFD defaults to the Documents folder
                    if let Some(path) = open_dialog_in_data_folder() {
                        let mut graph = ControllerStickGraph::default();
                        if let Err(e) = graph.load(path) {
                            error!("{:?}", e);
                        } else {
                            self.stick_graphs.push((true, graph));
                        }
                    }
                };
                if ui.button("Visualize Buttons").clicked() {
                    if let Some(path) = open_dialog_in_data_folder() {
                        let mut graph = ControllerButtonGraph::default();
                        if let Err(e) = graph.load(path) {
                            error!("{:?}", e);
                        } else {
                            self.button_graphs.push((true, graph));
                        }
                    }
                }
            });
            self.stick_graphs
                .iter_mut()
                .for_each(|(show_graph, graph)| {
                    graph.show(ctx, show_graph);
                });
            // remove the stick graphs that are closed (they're set to show when they're created)
            self.stick_graphs.retain(|(show_graph, _)| *show_graph);
            self.button_graphs
                .iter_mut()
                .for_each(|(show_graph, graph)| {
                    graph.show(ctx, show_graph);
                });
            self.button_graphs.retain(|(show_graph, _)| *show_graph);
        });
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

/// Loads the icon data
///
/// # Errors
///
/// This function errors if there is an issue reading the icon data from the file.
///
/// returns: `ImageResult<IconData>`
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
