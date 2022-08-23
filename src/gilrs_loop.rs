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
use log::{debug, error, warn};

use crate::{
    util::{create_dir_if_not_exists, get_exe_parent_dir},
    ControllerButtonEvent, ControllerConnectionEvent, ControllerStickEvent, ControllerStickState,
    CrossbeamChannelPair,
};

// TODO: use to replace starting/stopping recording, possibly the the event loop altogether
#[derive(Debug)]
pub enum GilrsEventLoopEvent {
    GetAllControllers,
}

#[derive(Default)]
pub struct GilrsEventLoop {
    pub channels: CrossbeamChannelPair<ControllerConnectionEvent>,
    pub event_channels: CrossbeamChannelPair<GilrsEventLoopEvent>,
    should_record: Arc<AtomicBool>,
    should_run: Arc<AtomicBool>,
    loop_handle: Option<JoinHandle<()>>,
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
        let should_record = self.should_record.clone();
        let channels = self.channels.clone();
        let event_channels = self.event_channels.clone();
        self.loop_handle = Some(thread::spawn(move || {
            if let Err(e) = inner_listen(&should_run, &should_record, &channels, &event_channels) {
                error!("{:?}", e);
            }
        }));
        Ok(())
    }

    /// Sets whether the event loop should record button/stick events.
    pub fn set_recording(&self, should_record: bool) {
        self.should_record.store(should_record, Ordering::Relaxed);
    }

    /// Returns whether the event loop is currently running.
    pub fn is_running(&self) -> bool {
        self.loop_handle.is_some() && self.should_run.load(Ordering::Relaxed)
    }

    /// Returns whether the event loop is currently recording button/stick events.
    pub fn is_recording(&self) -> bool {
        self.is_running() && self.should_record.load(Ordering::Relaxed)
    }

    /// Stops the event loop. This will block until the event loop has stopped.
    pub fn stop_listening(&mut self) {
        if self.loop_handle.is_none() || !self.is_running() {
            return;
        }
        self.should_record.store(false, Ordering::Relaxed);
        self.should_run.store(false, Ordering::Relaxed);
        if let Err(e) = self.loop_handle.take().unwrap().join() {
            error!("{:?}", e);
        }
    }
}

fn make_csv_writers() -> io::Result<(csv::Writer<File>, csv::Writer<File>)> {
    let data_folder = get_exe_parent_dir().join("data");
    create_dir_if_not_exists(&data_folder)?;
    let timestamp_string = chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d_%H-%M-%S.csv")
        .to_string();

    // csv file paths
    let button_csv_path = data_folder.join("buttons_".to_owned() + &timestamp_string);
    let stick_csv_path = data_folder.join("sticks_".to_owned() + &timestamp_string);

    // csv writers
    let button_csv_writer = csv::Writer::from_path(button_csv_path)?;
    let stick_csv_writer = csv::Writer::from_path(stick_csv_path)?;
    Ok((button_csv_writer, stick_csv_writer))
}

fn inner_listen(
    should_run: &Arc<AtomicBool>,
    should_record: &Arc<AtomicBool>,
    channels: &CrossbeamChannelPair<ControllerConnectionEvent>,
    event_channels: &CrossbeamChannelPair<GilrsEventLoopEvent>,
) -> io::Result<()> {
    // if this fails, the event loop can never run
    let mut gilrs = Gilrs::new().expect("failed to initialize controller processor");
    // loads any currently connected controllers into the UI

    // TODO: make dedicated writer thread using a crossbeam queue
    // csv writers
    let mut writers: Option<(csv::Writer<File>, csv::Writer<File>)> = None;

    // time map
    let mut time_map: HashMap<gilrs::GamepadId, SystemTime> = HashMap::new();

    // stick state
    let mut left_stick_state = ControllerStickState::default();
    let mut right_stick_state = ControllerStickState::default();

    let mut start_time = SystemTime::now();
    let mut last_record = should_record.load(Ordering::Relaxed);

    while should_run.load(Ordering::Relaxed) {
        // get events
        for next_event in event_channels.rx.try_iter() {
            debug!("got event: {:?}", next_event);
            match next_event {
                GilrsEventLoopEvent::GetAllControllers => {
                    gilrs.gamepads().for_each(|(id, gamepad)| {
                        if let Err(e) = channels.tx.send(ControllerConnectionEvent {
                            connected: true,
                            controller_id: id,
                            gamepad_name: gamepad.name().to_string(),
                        }) {
                            warn!(
                                "Error sending controller connection to main thread: {:?}",
                                e
                            );
                        }
                    });
                }
            }
        }
        let should_record = should_record.load(Ordering::Relaxed);
        if !last_record && should_record {
            if writers.is_some() {
                let (button_csv_writer, stick_csv_writer) = &mut writers.as_mut().unwrap();
                button_csv_writer.flush()?;
                stick_csv_writer.flush()?;
            }
            writers = Some(make_csv_writers()?);
            time_map.clear();
            left_stick_state = ControllerStickState::default();
            right_stick_state = ControllerStickState::default();
            start_time = SystemTime::now();
        } else if last_record && !should_record {
            writers = None;
        }
        last_record = should_record;
        while let Some(gilrs::Event {
            event,
            time: event_time,
            id: gamepad_id,
        }) = gilrs.next_event()
        {
            match event {
                EventType::AxisChanged(axis, value, ..) if should_record => {
                    match axis {
                        Axis::LeftStickX => left_stick_state.x = value,
                        Axis::LeftStickY => left_stick_state.y = value,
                        Axis::RightStickX => right_stick_state.x = value,
                        Axis::RightStickY => right_stick_state.y = value,
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
                    let stick_csv_writer = &mut writers.as_mut().unwrap().1;
                    if let Err(e) = stick_csv_writer.serialize(&stick_event) {
                        error!(
                            "failed to write stick event <{:?}> to csv with following error: {:?}",
                            stick_event, e
                        );
                    }
                    stick_csv_writer.flush()?;
                }
                EventType::ButtonChanged(button, value, ..) if should_record => {
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
                        let button_csv_writer = &mut writers.as_mut().unwrap().0;
                        if let Err(e) = button_csv_writer.serialize(&button_event) {
                            error!(
                            "failed to write button event <{:?}> to csv with following error: {:?}",
                            button_event, e
                        );
                        }
                        button_csv_writer.flush()?;
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
                EventType::Connected | EventType::Disconnected => {
                    let connected = matches!(event, EventType::Connected);
                    let gamepad_name = gilrs.gamepad(gamepad_id).name().to_string();
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
    Ok(())
}
