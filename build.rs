use std::{env, fs::File, io::BufWriter, path::PathBuf};

use image::{ImageOutputFormat, ImageResult};
use winres;

fn main() -> ImageResult<()> {
    #[cfg(windows)]
    {
        // write icon to output directory
        let out_dir = env::var_os("OUT_DIR").unwrap();
        let dest_path = PathBuf::from(out_dir).join("icon.ico");
        let img = image::open("./assets/icon.ico")?;
        img.write_to(
            &mut BufWriter::new(File::create(dest_path)?),
            ImageOutputFormat::Ico,
        )?;

        // write Windows resource file
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().unwrap();
    }
    Ok(())
}
