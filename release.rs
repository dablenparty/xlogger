#[cfg(target_os = "macos")]
use std::{fs, io, process::Command};

#[cfg(target_os = "macos")]
fn main() -> Result<(), io::Error> {
    // TODO: package icon
    println!("building xlogger.app");
    const XLOGGER_TARGET: &str = "target/release/macos/xlogger";
    Command::new("cargo")
        .args(["build", "--release", "--bin", "xlogger"])
        .status()?;
    fs::create_dir_all(format!("{}/xlogger.app/Contents/MacOS", XLOGGER_TARGET))?;
    fs::copy(
        "target/release/xlogger",
        format!("{}/xlogger.app/Contents/MacOS/xlogger", XLOGGER_TARGET),
    )?;
    println!("packaging xlogger.app");
    Command::new("hdiutil")
        .args([
            "create",
            "target/release/macos/xlogger.dmg",
            "-volname",
            "xlogger",
            "-srcfolder",
            XLOGGER_TARGET,
            "-ov",
        ])
        .status()?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    // TODO: implement packaging for Windows and Linux
}
