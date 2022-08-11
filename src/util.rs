use std::fs::create_dir_all;
use std::io;
use std::path::{Path, PathBuf};

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

/// Gets the name for a GILRS button
///
/// # Arguments
///
/// * `button`: The GILRS button to get the name for
///
/// returns: `String`
pub fn get_button_name(button: gilrs::Button) -> String {
    // TODO: add platform maps (e.g., Xbox, PS, etc.)
    format!("{:?}", button)
}
