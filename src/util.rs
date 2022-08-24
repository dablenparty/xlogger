use std::fs::create_dir_all;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDateTime, Utc};
use log::warn;

/// Creates a directory if it does not exist, failing if some other error occurs
///
/// # Arguments
///
/// * `file_path`: the path to the directory
///
/// returns: ()
pub fn create_dir_if_not_exists(file_path: &PathBuf) -> io::Result<()> {
    if let Err(e) = create_dir_all(&file_path) {
        if e.kind() == io::ErrorKind::AlreadyExists {
            warn!("{} already exists", file_path.display());
            Ok(())
        } else {
            Err(e)
        }
    } else {
        Ok(())
    }
}

/// Gets the path to the directory containing the executable
///
/// returns: `PathBuf`
pub fn get_exe_parent_dir() -> PathBuf {
    std::env::current_exe()
        .unwrap_or_else(|_| {
            warn!("failed to get current executable path");
            PathBuf::from("./xlogger.exe")
        })
        .parent()
        .unwrap_or_else(|| {
            warn!("failed to get parent of executable");
            Path::new(".")
        })
        .to_path_buf()
}

/// Formats an f64 to a string with the format "%H:%M:%S.%2f"
///
/// # Arguments
///
/// * `secs`: the time to format in seconds
///
/// returns: `String`
pub fn f64_to_formatted_time(secs: f64) -> String {
    const TIME_FORMAT: &str = "%H:%M:%S";
    let datetime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(secs as i64, 0), Utc)
        .format(TIME_FORMAT)
        .to_string();
    format!(
        "{}.{:.2}",
        datetime,
        secs.fract()
            .to_string()
            .split_once('.')
            .unwrap_or(("0", "0"))
            .1
    )
}
