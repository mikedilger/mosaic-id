use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

use mosaic_core::{EncryptedSecretKey, UserBootstrap};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserProfile; // define in mosaic-core

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyCertificate; // define in mosaic-core

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Data {
    pub encrypted_master_key: Option<EncryptedSecretKey>,
    pub bootstrap: Option<UserBootstrap>,
    pub profile: Option<UserProfile>,
    pub key_schedule: Option<Vec<KeyCertificate>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config_file = data_path()?;

    let data: Data = if config_file.exists() {
        let contents = fs::read(config_file)?;
        serde_json::from_slice(&contents)?
    } else {
        Data::default()
    };

    eprintln!("DATA: {}", serde_json::to_string(&data)?);

    Ok(())
}

fn data_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
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

    let mut data_path = data_dir;

    data_path.push("mosaic.json");

    Ok(data_path)
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
