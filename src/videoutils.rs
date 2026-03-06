use anyhow::{Result, anyhow};
use std::io::Error;
use std::path::Path;
use std::time::SystemTime;

#[allow(dead_code)]
pub fn get_creation_date(video_path: &Path) -> Result<SystemTime, Error> {
    video_path.metadata().and_then(|meta| meta.created())
}

#[allow(dead_code)]
pub fn get_mod_date(video_path: &Path) -> Result<SystemTime, Error> {
    video_path.metadata().and_then(|meta| meta.modified())
}

#[allow(dead_code)]
pub fn get_name(video_path: &Path) -> Result<String, anyhow::Error> {
    let stem = video_path.file_stem();
    if let Some(name) = stem {
        Ok(name.to_string_lossy().to_string())
    } else {
        Err(anyhow!(format!("{:?}", stem)))
    }
}

#[allow(dead_code)]
pub fn get_size(video_path: &Path) -> Result<u64, Error> {
    video_path.metadata().map(|meta| meta.len())
}
