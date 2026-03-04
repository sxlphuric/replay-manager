use anyhow::{Result, anyhow};
use std::io::Error;
use std::path::PathBuf;
use std::time::SystemTime;

#[allow(dead_code)]
pub fn get_creation_date(video_path: &PathBuf) -> Result<SystemTime, Error> {
    let metadata = video_path.metadata();
    if metadata.is_ok() {
        metadata.unwrap().created()
    } else {
        Err(metadata.unwrap_err())
    }
}

#[allow(dead_code)]
pub fn get_mod_date(video_path: &PathBuf) -> Result<SystemTime, Error> {
    let metadata = video_path.metadata();
    if metadata.is_ok() {
        metadata.unwrap().modified()
    } else {
        Err(metadata.unwrap_err())
    }
}

#[allow(dead_code)]
pub fn get_name(video_path: &PathBuf) -> Result<String, anyhow::Error> {
    let stem = video_path.file_stem();
    if stem.is_some() {
        Ok(stem.unwrap().to_string_lossy().to_string())
    } else {
        Err(anyhow!(format!("{:?}", stem)))
    }
}

#[allow(dead_code)]
pub fn get_size(video_path: &PathBuf) -> Result<u64, Error> {
    let metadata = video_path.metadata();
    if metadata.is_ok() {
        Ok(metadata.unwrap().len())
    } else {
        Err(metadata.unwrap_err())
    }
}
