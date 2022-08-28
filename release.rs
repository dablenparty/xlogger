#[cfg(target_os = "macos")]
use std::{fs, io, process::Command};

#[cfg(target_os = "macos")]
fn main() -> Result<(), io::Error> {
    use std::path::PathBuf;

    const XLOGGER_TARGET: &str = "target/release/macos/xlogger/xlogger.app";
    const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    if PathBuf::from(XLOGGER_TARGET).exists() {
        println!("removing old xlogger.app");
        fs::remove_dir_all(XLOGGER_TARGET)?;
    }
    // compile in release
    Command::new("cargo")
        .args(["build", "--release", "--bin", "xlogger"])
        .status()?;
    println!("building xlogger.app");
    // copy binary
    let contents_folder = format!("{}/Contents", XLOGGER_TARGET);
    fs::create_dir_all(format!("{}/MacOS", contents_folder))?;
    fs::copy(
        "target/release/xlogger",
        format!("{}/MacOS/xlogger", contents_folder),
    )?;
    // copy icon
    let resources_folder = format!("{}/Resources", contents_folder);
    fs::create_dir_all(&resources_folder)?;
    let icon_path = format!("{}/icon.icns", resources_folder);
    fs::copy("assets/icon.icns", icon_path)?;
    // format and copy Info.plist
    //* MacOS parses whitespace in Info.plist as significant, so don't format with extra newlines and spaces
    let plist_path = format!("{}/Info.plist", contents_folder);
    let plist_text = fs::read_to_string("assets/macos/Info.plist")?
        .replace("{XLOGGER_BUNDLE_VERSION}", CARGO_PKG_VERSION)
        .replace("{XLOGGER_BUNDLE_VERSION_SHORT}", CARGO_PKG_VERSION);
    fs::write(plist_path, plist_text)?;
    println!("packaging xlogger.app");
    // package into dmg
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
