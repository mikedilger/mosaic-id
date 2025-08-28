use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crossterm::event::{Event, KeyCode};
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
    pub fn key(&self) -> KeyCode {
        match self {
            Self::NewMaster => KeyCode::Char('m'),
            Self::DestroyMaster => KeyCode::Char('x'),
            Self::NewBootstrap => KeyCode::Char('b'),
            Self::NewProfile => KeyCode::Char('p'),
            Self::NewKeySchedule => KeyCode::Char('k'),
            Self::SaveAndExit => KeyCode::Char('s'),
            Self::ExitWithoutSaving => KeyCode::Char('q'),
        }
    }

    pub fn from_key(k: KeyCode) -> Option<MenuOption> {
        match k {
            KeyCode::Char('m') => Some(Self::NewMaster),
            KeyCode::Char('x') => Some(Self::DestroyMaster),
            KeyCode::Char('b') => Some(Self::NewBootstrap),
            KeyCode::Char('p') => Some(Self::NewProfile),
            KeyCode::Char('k') => Some(Self::NewKeySchedule),
            KeyCode::Char('s') => Some(Self::SaveAndExit),
            KeyCode::Char('q') => Some(Self::ExitWithoutSaving),
            _ => None,
        }
    }

    pub fn prompt(&self) -> &'static str {
        match self {
            Self::NewMaster => "Generate a new Master Keypair",
            Self::DestroyMaster => "DESTROY your Master Keypair (DANGER!)",
            Self::NewBootstrap => "Generate a new empty Bootstrap",
            Self::NewProfile => "Generate a new empty Profile",
            Self::NewKeySchedule => "Generate a new empty Key Schedule",
            Self::SaveAndExit => "Save and Quit",
            Self::ExitWithoutSaving => "Quit Without Saving",
        }
    }
}

pub fn options_from_data(data: &Data) -> Vec<MenuOption> {
    let mut options: Vec<MenuOption> = Vec::new();

    if let Some(_) = data.encrypted_master_key {
        options.push(MenuOption::DestroyMaster);

        if let Some(ref _bootstrap) = data.bootstrap {
        } else {
            options.push(MenuOption::NewBootstrap);
        }

        if let Some(ref _profile) = data.profile {
        } else {
            options.push(MenuOption::NewProfile);
        }

        if let Some(ref _key_schedule) = data.profile {
        } else {
            options.push(MenuOption::NewKeySchedule);
        }
    } else {
        options.push(MenuOption::NewMaster);
    }

    options.push(MenuOption::SaveAndExit);
    options.push(MenuOption::ExitWithoutSaving);

    options
}

pub fn update(data: &mut Data, config_file: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut stdout = std::io::stdout();

    'next_menu:
    loop {
        let options = options_from_data(&*data);

        println!("\n-----------------------------------");
        for option in options.iter() {
            println!("{}) {}", option.key(), option.prompt());
        }
        print!("> ");
        stdout.flush()?;

        'next_keypress:
        loop {
            match crossterm::event::read()? {
                Event::Key(key_event) => {
                    if let Some(option) = MenuOption::from_key(key_event.code) {
                        println!("");
                        if execute(option, data, config_file.as_path())? {
                            return Ok(());
                        }
                        continue 'next_menu;
                    } else {
                        continue 'next_keypress;
                    }
                },
                _ => continue 'next_keypress,
            }
        }
    }
}

// Return value 'true' means exit
pub fn execute(
    option: MenuOption,
    data: &mut Data,
    config_file: &Path
) -> Result<bool,  Box<dyn Error>> {
    match option {
        MenuOption::NewMaster => {
            println!("New Master - not yet implemented.");
        }
        MenuOption::DestroyMaster => {
            println!("Destroy Master - not yet implemented.");
        }
        MenuOption::NewBootstrap => {
            println!("New Bootstrap - not yet implemented.");
        }
        MenuOption::NewProfile => {
            println!("New Profile - not yet implemented.");
        }
        MenuOption::NewKeySchedule => {
            println!("New Key Schedule - not yet implemented.");
        }
        MenuOption::SaveAndExit => {
            let contents: String = serde_json::to_string(&data)?;
            fs::write(&config_file, contents)?;
            println!("Saved.");
            return Ok(true);
        }
        MenuOption::ExitWithoutSaving => {
            return Ok(true);
        }
    }

    Ok(false)
}
