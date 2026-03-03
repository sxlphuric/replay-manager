use anyhow::Result;
use std::path::PathBuf;
use std::time::SystemTime;

#[allow(dead_code)]
pub fn get_creation_date(video_path: &PathBuf) -> Result<SystemTime, std::io::Error> {
    video_path
        .metadata()
        .expect("Could not get video metadata")
        .created()
}

#[allow(dead_code)]
pub fn get_mod_date(video_path: &PathBuf) -> Result<SystemTime, std::io::Error> {
    video_path
        .metadata()
        .expect("Could not get video metadata")
        .modified()
}

#[allow(dead_code)]
pub fn get_name(video_path: &PathBuf) -> String {
    video_path
        .file_stem()
        .expect("Could not get file name")
        .to_string_lossy()
        .to_string()
}

#[allow(dead_code)]
pub fn get_size(video_path: &PathBuf) -> u64 {
    video_path
        .metadata()
        .expect("Could not get video metadata")
        .len()
}
