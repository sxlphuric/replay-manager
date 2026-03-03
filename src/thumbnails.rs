use anyhow::{Result, anyhow};
use std::path::PathBuf;
use std::process::Command;

#[allow(dead_code)]
pub fn create<'a>(videopath: &PathBuf, folder: &'a str, check_exists: bool) -> Result<PathBuf> {
    let thumbnail_path = PathBuf::from(&format!(
        "{}/.thumbnails/Thumbnail_{}.png",
        folder,
        &videopath
            .file_stem()
            .expect("Could not find file name")
            .to_string_lossy()
    ));
    if check_exists {
        if thumbnail_path.exists() {
            return Ok(thumbnail_path);
        }
    }

    let output = Command::new("ffmpeg")
        .arg("-hwaccel")
        .arg("auto")
        .arg("-strict")
        .arg("experimental")
        .arg("-ss")
        .arg("5.0")
        .arg("-i")
        .arg(format!("{}", &videopath.display()))
        .arg("-frames:v")
        .arg("1")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-vf")
        .arg("scale=640:360")
        .arg("-f")
        .arg("image2")
        .arg("-q:v")
        .arg("2")
        .arg("-an")
        .arg("-y")
        .arg(&thumbnail_path)
        .output()?;
    if output.status.success() {
        println!("| Done: {}", thumbnail_path.display());
    } else {
        return Err(anyhow!(String::from_utf8(output.stderr)?));
    }

    Ok(thumbnail_path)
}
