use anyhow::{Result, anyhow};
use std::path::PathBuf;
use std::process::Command;

#[allow(dead_code)]
pub fn create(
    videopath: &PathBuf,
    folder: &str,
    check_exists: bool,
    thumbnail_time: f64,
) -> Result<PathBuf> {
    let video_name = &videopath.file_stem();
    let thumbnail_path = if video_name.is_some() {
        PathBuf::from(&format!(
            "{}/.thumbnails/Thumbnail_{}.png",
            folder,
            video_name.unwrap().to_string_lossy()
        ))
    } else {
        return Err(anyhow!(format!("Video name is empty")));
    };

    let thumbnail_dir = thumbnail_path.parent().unwrap();
    if !thumbnail_dir.exists() {
        std::fs::create_dir_all(thumbnail_dir)?;
    }
    if check_exists && thumbnail_path.exists() {
        return Ok(thumbnail_path);
    }

    let ffmpeg_cmd = find_ffmpeg()?;

    let output = Command::new(ffmpeg_cmd)
        .arg("-hwaccel")
        .arg("auto")
        .arg("-strict")
        .arg("experimental")
        .arg("-ss")
        .arg(thumbnail_time.to_string())
        .arg("-i")
        .arg(videopath)
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

fn find_ffmpeg() -> Result<String> {
    which::which("ffmpeg")
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|_| anyhow!("ffmpeg not found in path, please install it or add it to path"))
}
