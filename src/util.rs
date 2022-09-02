use std::{
    fs::create_dir_all,
    io,
    path::{Path, PathBuf},
};

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

/// Gets the path to the directory containing the executable, resolving symlinks if necessary.
///
/// The current working directory is not used by default because the executable may be running from a
/// different directory than the one containing said executable. This is especially important for
/// logging and/or data storage, where having a constant directory is useful.
///
/// # Errors
/// There are several errors that may occur in this process, although they are all handled.
///
/// If an error occurs...
/// - when attempting to get the executable path, the executable is taken from the command line args.
/// - when attempting to get the executable from the command line args, a default value is used.
///     - `.\{CARGO_PKG_NAME}.exe` on Windows, `./{CARGO_PKG_NAME}` on Unix
/// - when resolving a symlink, the original symlink path is returned.
/// - when attempting to get the parent of the executable path, the current working directory is
/// used.
/// - when attempting to canonicalize the final parent path, the current working directory is used as `.`.
///
/// returns: `PathBuf`
pub fn get_exe_parent_dir() -> PathBuf {
    let initial_path = std::env::current_exe().unwrap_or_else(|_| {
        warn!("failed to get current executable path");
        let exec_name = std::env::args().next().unwrap_or_else(|| {
            warn!("failed to get executable name from args");
            let dummy_name = env!("CARGO_PKG_NAME");
            if cfg!(windows) {
                format!(".\\{}.exe", dummy_name)
            } else {
                format!("./{}", dummy_name)
            }
        });
        PathBuf::from(exec_name)
    });
    // is_symlink also checks for existence and permissions, so we don't need to do that here
    let resolved_path = if initial_path.is_symlink() {
        initial_path.read_link().unwrap_or_else(|_| {
            warn!("failed to read link {}", initial_path.display());
            initial_path
        })
    } else {
        initial_path
    };
    resolved_path
        .parent()
        .unwrap_or_else(|| {
            warn!("failed to get parent of executable");
            Path::new(".")
        })
        .canonicalize()
        .unwrap_or_else(|e| {
            warn!(
                "failed to canonicalize path {}: {}",
                resolved_path.display(),
                e
            );
            resolved_path
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
