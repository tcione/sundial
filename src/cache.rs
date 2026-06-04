use std::path::PathBuf;

pub fn get_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dirs = directories::ProjectDirs::from("", "", "sundial")
        .ok_or("Could not find config directory")?;

    let data_dir = dirs.data_dir();
    std::fs::create_dir_all(&data_dir)?;

    Ok(data_dir.to_path_buf())
}
