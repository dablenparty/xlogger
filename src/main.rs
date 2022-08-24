#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::fs::File;
use std::process;

use eframe::egui;
use eframe::IconData;
use human_panic::setup_panic;
use image::ImageResult;
use log::{error, info, warn, LevelFilter};
use simplelog::{Config, WriteLogger};
use xlogger::button_graph::ControllerButtonGraph;
use xlogger::gilrs_loop::GilrsEventLoop;
use xlogger::gilrs_loop::GELEvent;
use xlogger::stick_graph::ControllerStickGraph;
use xlogger::util::{create_dir_if_not_exists, get_exe_parent_dir};
use xlogger::ControllerConnectionEvent;
use xlogger::{open_dialog_in_data_folder, BoxedResult, EguiView, StatefulText};

#[derive(Default)]
struct XloggerApp {
    saved_text: StatefulText,
    stick_graphs: Vec<(bool, ControllerStickGraph)>,
    button_graphs: Vec<(bool, ControllerButtonGraph)>,
    event_loop: GilrsEventLoop,
    connected_controllers: Vec<ControllerConnectionEvent>,
    event_loop_is_recording: bool,
}

impl eframe::App for XloggerApp {
    fn on_exit(&mut self, _gl: &eframe::glow::Context) {
        // TODO: confirm exit if event loop is recording
        info!("Closing GILRS event loop");
        self.event_loop.stop_listening();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.event_loop.is_running() {
                egui::Window::new("Critical Error").title_bar(true).show(ctx, |ui| {
                    StatefulText::new("The GILRS event loop is not running. Please restart the application.\n\nIf the issue persists, check the logs for more information.".to_string(), xlogger::TextState::Error).show(ui);
                });
            }
            let text = if self.event_loop_is_recording { "Stop" } else { "Start" };
            ui.horizontal(|ui| {
                if ui.button(text).clicked() {
                    let (log_message, saved_text) = if self.event_loop_is_recording {
                        self.event_loop_is_recording = false;
                        if let Err(e) = self.event_loop.event_channels.tx.send(GELEvent::StopRecording) {
                            error!("Failed to send stop recording event: {:?}", e);
                        }
                        ("stopped listening to controllers", "Saved!".to_owned())
                    } else {
                        self.event_loop_is_recording = true;
                        if let Err(e) = self.event_loop.event_channels.tx.send(GELEvent::StartRecording) {
                            error!("Failed to send start recording event: {:?}", e);
                        }
                        ("started listening to controllers", "".to_owned())
                    };
                    self.saved_text.text = saved_text;
                    info!("{}", log_message);
                }
                self.saved_text.show(ui);
            });
            ui.horizontal(|ui| {
                // TODO: find a way to display errors back to the user
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
            // TODO: make event type an enum, highlight controller in list on input
            for e in self.event_loop.channels.rx.try_iter() {
                if e.connected {
                    self.connected_controllers.push(e);
                } else {
                    self.connected_controllers
                        .retain(|c| c.controller_id != e.controller_id);
                }
            }
            ui.vertical(|ui| {
                ui.label(format!(
                    "Connected controllers: {}",
                    self.connected_controllers.len()
                ));
                for e in &self.connected_controllers {
                    ui.label(format!("[{}] {}", e.controller_id, e.gamepad_name));
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
/// In debug mode, the log level is set to debug for the terminal and info for the file.
///  In release mode, there is no terminal logger and the log level is set to info for the file.
#[cfg(not(debug_assertions))]
fn init_logger() -> BoxedResult<()> {
    // release mode
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

/// Initializes the logging library
///
/// In debug mode, the log level is set to debug for the terminal and info for the file.
///  In release mode, there is no terminal logger and the log level is set to info for the file.
#[cfg(debug_assertions)]
fn init_logger() -> BoxedResult<()> {
    use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode};
    // debug mode

    let mut file_path = get_exe_parent_dir();
    let filename = chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d %H-%M-%S.log")
        .to_string();
    file_path.push("logs");
    create_dir_if_not_exists(&file_path)?;
    file_path.push(filename);
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(&file_path)?,
        ),
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Always,
        ),
    ])?;
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

    let mut app = XloggerApp::default();
    if let Err(e) = app.event_loop.listen_for_events() {
        error!("{:?}", e);
        process::exit(1);
    }
    // loads initial controllers into UI on first render
    if let Err(e) = app
        .event_loop
        .event_channels
        .tx
        .send(GELEvent::GetAllControllers)
    {
        error!("{:?}", e);
    }
    let native_options = get_icon_data().map_or_else(
        |err| {
            warn!("Failed to load icon with error: {}", err);
            eframe::NativeOptions::default()
        },
        |icon_data| eframe::NativeOptions {
            icon_data: Some(icon_data),
            ..eframe::NativeOptions::default()
        },
    );
    eframe::run_native(
        "xlogger",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(app)
        }),
    );
}
