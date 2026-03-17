use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};

const FAVORITES_DIR_NAME: &str = ".favorites";

#[allow(dead_code)]
fn save(replay_path: &PathBuf, replay_name: &str) -> Result<PathBuf> {
    let replay_dir = replay_path.parent();
    let replay_dir =
        check_subdirectory(replay_dir).expect("Could not create favorite replays directory");
    let favorites_dir = replay_dir.join(FAVORITES_DIR_NAME);
    if let Err(e) = fs::hard_link(
        replay_path,
        favorites_dir.join(format!(
            "{}.{}",
            replay_name,
            replay_path.extension().unwrap().display()
        )),
    ) {
        return Err(anyhow!("Could not create favorite replay link: {}", e));
    }

    Ok(favorites_dir.join(replay_path.file_name().unwrap()))
}

#[allow(dead_code)]
fn remove(saved_replay_path: &PathBuf) -> Result<&PathBuf> {
    if let Err(e) = fs::remove_file(saved_replay_path) {
        return Err(anyhow!("Could not remove replay from favorites: {}", e));
    }

    Ok(saved_replay_path)
}

fn check_subdirectory(parent_dir: Option<&Path>) -> Result<PathBuf> {
    let path = parent_dir.map_or_else(|| PathBuf::new(), |p| p.to_path_buf());
    let favorites_path = path.clone().join(FAVORITES_DIR_NAME);
    if !favorites_path.exists() {
        fs::create_dir_all(favorites_path.clone())
            .expect("Could not create saved replays subdirectory");
    } else {
        return Ok(favorites_path.clone());
    }
    Ok(favorites_path.clone())
}
