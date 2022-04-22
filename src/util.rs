use std::fs::create_dir;
use std::path::{Path, PathBuf};

use log::warn;

/// Creates a directory if it does not exist, failing if some other error occurs
///
/// # Arguments
///
/// * `file_path`: the path to the directory
///
/// returns: ()
pub fn create_dir_if_not_exists(file_path: &PathBuf) {
    create_dir(&file_path).unwrap_or_else(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            return;
        }
        // initialization of the logging library uses this function, so we can't use the error!
        // macro here
        eprintln!("{:?}", e);
        std::process::exit(1);
    });
}

/// Gets the path to the directory containing the executable
///
/// returns: PathBuf
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
