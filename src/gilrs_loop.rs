use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::SystemTime,
};

use gilrs::{Axis, EventType, Gilrs};
use log::{debug, error, info, warn};

use crate::{
    util::{create_dir_if_not_exists, get_exe_parent_dir},
    ControllerButtonEvent, ControllerConnectionEvent, ControllerStickEvent, ControllerStickState,
    CrossbeamChannelPair,
};

/// Gilrs Event Loop Event
#[derive(Debug)]
pub enum GELEvent {
    GetAllControllers,
    StartRecording,
    StopRecording,
}

#[derive(Default)]
pub struct GilrsEventLoop {
    pub channels: CrossbeamChannelPair<ControllerConnectionEvent>,
    pub event_channels: CrossbeamChannelPair<GELEvent>,
    should_run: Arc<AtomicBool>,
    loop_handle: Option<JoinHandle<()>>,
}

/// Internal helper struct to represent a writer thread.
///
/// This could probably be easily refactored to be a helper struct for threads in general.
#[derive(Default)]
struct WriterThread {
    /// Marks whether the thread should continue running. Setting this to false will cause the thread to exit.
    should_run: Arc<AtomicBool>,
    /// Channel pair used to send events to the thread.
    channels: CrossbeamChannelPair<gilrs::Event>,
    /// Join handle for the thread. This is None if the thread is not running.
    thread_handle: Option<JoinHandle<()>>,
    /// Prefix for the file names.
    file_name_prefix: String,
}

impl WriterThread {
    /// Starts the writer thread.
    ///
    /// returns: `io::Result<()>`
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if one occurs while creating the CSV writers.
    fn start(&mut self) -> io::Result<()> {
        let (mut button_csv_writer, mut stick_csv_writer) =
            make_csv_writers(&self.file_name_prefix)?;

        let mut time_map: HashMap<gilrs::GamepadId, SystemTime> = HashMap::new();

        let mut left_stick_state = ControllerStickState::default();
        let mut right_stick_state = ControllerStickState::default();

        let start_time = SystemTime::now();

        self.should_run.store(true, Ordering::SeqCst);

        let thread_channels = self.channels.clone();
        let run = self.should_run.clone();

        let join_handle = thread::spawn(move || {
            while run.load(Ordering::SeqCst) {
                for next_event in thread_channels.rx.try_iter() {
                    let gilrs::Event {
                        event,
                        time: event_time,
                        id: gamepad_id,
                    } = next_event;
                    match event {
                        EventType::AxisChanged(axis, value, ..) => {
                            match axis {
                                Axis::LeftStickX => left_stick_state.x = value as f64,
                                Axis::LeftStickY => left_stick_state.y = value as f64,
                                Axis::RightStickX => right_stick_state.x = value as f64,
                                Axis::RightStickY => right_stick_state.y = value as f64,
                                _ => {
                                    warn!("unhandled axis event: {:?}", event);
                                    continue;
                                }
                            }
                            let stick_event = ControllerStickEvent {
                                time: event_time
                                    .duration_since(start_time)
                                    .expect("time went backwards!")
                                    .as_secs_f64(),
                                left_x: left_stick_state.x,
                                left_y: left_stick_state.y,
                                right_x: right_stick_state.x,
                                right_y: right_stick_state.y,
                            };
                            if let Err(e) = stick_csv_writer.serialize(&stick_event) {
                                error!(
                                "failed to write stick event <{:?}> to csv with following error: {:?}",
                                stick_event, e
                            );
                            }
                            if let Err(e) = stick_csv_writer.flush() {
                                error!(
                                "failed to flush stick event <{:?}> to csv with following error: {:?}",
                                stick_event, e
                            );
                            }
                        }
                        EventType::ButtonChanged(button, value, ..) => {
                            if value == 0.0 {
                                let down_time =
                                    time_map.remove(&gamepad_id).unwrap_or_else(SystemTime::now);
                                let button_event = ControllerButtonEvent {
                                    press_time: down_time
                                        .duration_since(start_time)
                                        .expect("time went backwards!")
                                        .as_secs_f64(),
                                    release_time: event_time
                                        .duration_since(start_time)
                                        .expect("time went backwards!")
                                        .as_secs_f64(),
                                    button,
                                };
                                if let Err(e) = button_csv_writer.serialize(&button_event) {
                                    error!(
                                "failed to write button event <{:?}> to csv with following error: {:?}",
                                button_event, e
                            );
                                }
                                if let Err(e) = button_csv_writer.flush() {
                                    error!(
                                "failed to flush button event <{:?}> to csv with following error: {:?}",
                                button_event, e
                                );
                                }
                            } else {
                                // only insert if it doesn't have a value (aka has the default value)
                                let map_time_opt = time_map.get(&gamepad_id);
                                if map_time_opt.unwrap_or(&SystemTime::UNIX_EPOCH)
                                    == &SystemTime::UNIX_EPOCH
                                {
                                    time_map.insert(gamepad_id, event_time);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
        self.thread_handle = Some(join_handle);
        Ok(())
    }

    /// Stops the writer thread. This will block until the thread has exited and is safe to call if the thread is not running.
    ///
    /// If an error occurs while stopping the thread, it is logged but the error is not returned.
    fn stop(&mut self) {
        if self.thread_handle.is_none() {
            return;
        }
        self.should_run.store(false, Ordering::SeqCst);
        if let Err(e) = self.thread_handle.take().unwrap().join() {
            error!("failed to join writer thread with following error: {:?}", e);
        }
    }
}

#[derive(Debug, Clone)]
pub enum GilrsEventLoopError {
    NoLoopHandle,
}

impl fmt::Display for GilrsEventLoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            GilrsEventLoopError::NoLoopHandle => write!(f, "no event loop join handle"),
        }
    }
}

impl std::error::Error for GilrsEventLoopError {}

impl GilrsEventLoop {
    /// Starts an event loop that listens for controller events and writes them to a file.
    ///
    /// This function is run on a separate thread.
    pub fn listen_for_events(&mut self) -> Result<(), GilrsEventLoopError> {
        if self.loop_handle.is_some() {
            return Err(GilrsEventLoopError::NoLoopHandle);
        }
        self.should_run.store(true, Ordering::Relaxed);
        let should_run = self.should_run.clone();
        let channels = self.channels.clone();
        let event_channels = self.event_channels.clone();
        self.loop_handle = Some(thread::spawn(move || {
            inner_listen(&should_run, &channels, &event_channels);
        }));
        Ok(())
    }

    /// Returns whether the event loop is currently running.
    pub fn is_running(&self) -> bool {
        self.loop_handle.is_some() && self.should_run.load(Ordering::Relaxed)
    }

    /// Stops the event loop. This will block until the event loop has stopped.
    pub fn stop_listening(&mut self) {
        if self.loop_handle.is_none() || !self.is_running() {
            return;
        }
        self.should_run.store(false, Ordering::Relaxed);
        if let Err(e) = self.loop_handle.take().unwrap().join() {
            error!("{:?}", e);
        }
    }
}

fn make_csv_writers(prefix: &str) -> io::Result<(csv::Writer<File>, csv::Writer<File>)> {
    let data_folder = get_exe_parent_dir().join("data");
    create_dir_if_not_exists(&data_folder)?;
    let timestamp_string = chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d_%H-%M-%S.csv")
        .to_string();

    // csv file paths
    let button_csv_path = data_folder.join(format!("{}buttons_{}.csv", prefix, timestamp_string));
    let stick_csv_path = data_folder.join(format!("{}sticks_{}.csv", prefix, timestamp_string));

    // csv writers
    let button_csv_writer = csv::Writer::from_path(button_csv_path)?;
    let stick_csv_writer = csv::Writer::from_path(stick_csv_path)?;
    Ok((button_csv_writer, stick_csv_writer))
}

fn inner_listen(
    should_run: &Arc<AtomicBool>,
    channels: &CrossbeamChannelPair<ControllerConnectionEvent>,
    event_channels: &CrossbeamChannelPair<GELEvent>,
) {
    // if this fails, the event loop can never run
    let mut gilrs = Gilrs::new().expect("failed to initialize controller processor");

    let mut writer_thread_map: HashMap<gilrs::GamepadId, WriterThread> = HashMap::new();

    gilrs.gamepads().for_each(|(gamepad_id, gamepad)| {
        let writer_thread = WriterThread {
            file_name_prefix: make_controller_name_prefix(gamepad),
            ..Default::default()
        };
        writer_thread_map.insert(gamepad_id, writer_thread);
    });

    while should_run.load(Ordering::Relaxed) {
        // get events
        for next_event in event_channels.rx.try_iter() {
            debug!("got event: {:?}", next_event);
            match next_event {
                GELEvent::GetAllControllers => {
                    gilrs.gamepads().for_each(|(id, gamepad)| {
                        if let Err(e) = channels.tx.send(ControllerConnectionEvent {
                            connected: true,
                            controller_id: id,
                            gamepad_name: gamepad.name().to_string(),
                        }) {
                            error!(
                                "Error sending controller connection to main thread: {:?}",
                                e
                            );
                        }
                    });
                }
                GELEvent::StartRecording => {
                    for (gamepad_id, writer_thread) in &mut writer_thread_map {
                        if let Err(e) = writer_thread.start() {
                            warn!("Error starting writer thread: {:?}", e);
                        }
                        info!("started recording gamepad {}", gamepad_id);
                    }
                }
                GELEvent::StopRecording => {
                    for (gamepad_id, writer_thread) in &mut writer_thread_map {
                        writer_thread.stop();
                        info!("stopped recording gamepad {}", gamepad_id);
                    }
                }
            }
        }
        while let Some(event) = gilrs.next_event() {
            let gilrs::Event {
                event: event_type,
                id: gamepad_id,
                ..
            } = event;
            match event_type {
                EventType::AxisChanged(..) | EventType::ButtonChanged(..) => {
                    if let Some(writer_thread) = writer_thread_map.get_mut(&gamepad_id) {
                        if let Err(e) = writer_thread.channels.tx.send(event) {
                            warn!("Error sending event to writer thread: {:?}", e);
                        }
                    }
                }
                EventType::Connected | EventType::Disconnected => {
                    let connected = matches!(event_type, EventType::Connected);
                    let gamepad = gilrs.gamepad(gamepad_id);
                    if connected {
                        // this shouldn't happen, but just in case
                        if let Some(existing_writer_thread) = writer_thread_map.get_mut(&gamepad_id)
                        {
                            existing_writer_thread.stop();
                        }
                        let writer_thread = WriterThread {
                            file_name_prefix: make_controller_name_prefix(gamepad),
                            ..Default::default()
                        };
                        writer_thread_map.insert(gamepad_id, writer_thread);
                    } else if let Some(mut writer_thread) = writer_thread_map.remove(&gamepad_id) {
                        writer_thread.stop();
                    }

                    let gamepad_name = gamepad.name().to_string();
                    let connection_event = ControllerConnectionEvent {
                        connected,
                        controller_id: gamepad_id,
                        gamepad_name,
                    };
                    if let Err(e) = channels.tx.send(connection_event.clone()) {
                        error!(
                            "failed to send connection event <{:?}> to channel with following error: {:?}",
                            connection_event, e
                        );
                    }
                }
                _ => {}
            }
        }
    }
    // stop the writer thread
    for (gamepad_id, writer_thread) in &mut writer_thread_map {
        info!("stopping recording for gamepad {:?}", gamepad_id);
        writer_thread.stop();
    }
}

/// Returns a file name prefix for the given gamepad.
///
/// Format: `<gamepad_name>_<gamepad_id>_`
///
/// # Arguments
///
/// * `gamepad` - The gamepad to get the file name prefix for.
///
/// returns: `String`
fn make_controller_name_prefix(gamepad: gilrs::Gamepad) -> String {
    let prefix = gamepad.name().replace(' ', "_");
    format!("{}_{}_", prefix, gamepad.id())
}
