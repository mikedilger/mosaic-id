use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub fn mosaic_dir() -> Result<PathBuf, Box<dyn Error>> {
    // Normalize their data dir
    let mut data_dir = normalize(
        dirs::data_dir()
            .ok_or(Box::<dyn Error>::from("Cannot determine data directory"))?
    );

    // Add "mosaic" to the end
    data_dir.push("mosaic");

    // normalize again in case mosaic existed and was a link
    let data_dir = normalize(data_dir.as_path());

    // Create the directory if it does not already exist
    fs::create_dir_all(&data_dir)?;

    Ok(data_dir)
}

#[cfg(not(windows))]
fn normalize<P: AsRef<Path>>(path: P) -> PathBuf {
    fs::canonicalize(&path).unwrap_or(path.as_ref().to_path_buf())
}

#[cfg(windows)]
fn normalize<P: AsRef<Path>>(path: P) -> PathBuf {
    match path.as_ref().normalize() {
        Ok(p) => p.into_path_buf(),
        Err(_) => path.as_ref().to_path_buf(),
    }
}
