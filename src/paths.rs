use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Paths {
    pub base: PathBuf,
    pub master_key: PathBuf,
    pub bootstrap: PathBuf,
    pub profile: PathBuf,
    pub key_schedule: PathBuf,
}

pub fn paths() -> Result<Paths, Box<dyn Error>> {
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

    let mut paths = Paths {
        base: data_dir.clone(),
        master_key: data_dir.clone(),
        bootstrap: data_dir.clone(),
        profile: data_dir.clone(),
        key_schedule: data_dir.clone(),
    };

    paths.master_key.push("master_key.mocryptsec0");
    paths.bootstrap.push("bootstrap.mub25");
    paths.profile.push("profile.morec");
    paths.key_schedule.push("key_schedule.morec");

    Ok(paths)
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
