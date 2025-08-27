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


    let mut data: Data = if config_file.exists() {
        let contents = fs::read(&config_file)?;
        serde_json::from_slice(&contents)?
    } else {
        Data::default()
    };

    eprintln!("Current Data: {}", serde_json::to_string_pretty(&data)?);

    update(&mut data, config_file)?;

    Ok(())
}

fn data_path() -> Result<PathBuf, Box<dyn Error>> {
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

pub enum MenuOption {
    NewMaster,
    DestroyMaster,
    NewBootstrap,
    NewProfile,
    NewKeySchedule,
    SaveAndExit,
    ExitWithoutSaving,
}

impl MenuOption {
    pub fn prompt(&self) -> &'static str {
        match self {
            Self::NewMaster => "Generate a new Master Keypair",
            Self::DestroyMaster => "DESTROY your Master Keypair (DANGER!)",
            Self::NewBootstrap => "Generate a new empty Bootstrap",
            Self::NewProfile => "Generate a new empty Profile",
            Self::NewKeySchedule => "Generate a new empty Key Schedule",
            Self::SaveAndExit => "Save and Exit",
            Self::ExitWithoutSaving => "Exit Without Saving",
        }
    }
}


pub fn update(data: &mut Data, config_file: PathBuf) -> Result<(), Box<dyn Error>> {

    loop {
        let mut options: Vec<MenuOption> = Vec::new();

        if let Some(_) = data.encrypted_master_key {
            options.push(MenuOption::DestroyMaster);
        } else {
            options.push(MenuOption::NewMaster);
        }

        if let Some(ref mut _bootstrap) = data.bootstrap {
        } else {
            options.push(MenuOption::NewBootstrap);
        }

        options.push(MenuOption::SaveAndExit);
        options.push(MenuOption::ExitWithoutSaving);


        match menu(data, options)? {
            None => continue,
            Some(true) => {
                let contents: String = serde_json::to_string(&data)?;
                fs::write(&config_file, contents)?;
                break;
            },
            Some(false) => {
                break;
            },
        }
    }

    Ok(())
}


// return Some(true) to save and exit, Some(false) to exit without saving
pub fn menu(_data: &mut Data, options: Vec<MenuOption>) -> Result<Option<bool>, Box<dyn Error>> {
    for (i, option) in options.iter().enumerate() {
        println!("{i}) {}", option.prompt());
    }

    // Cheat while testing
    return Ok(Some(true));

}
