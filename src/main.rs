use std::error::Error;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crossterm::event::{Event, KeyCode};
use serde::{Serialize, Deserialize};

use mosaic_core::{EncryptedSecretKey, SecretKey, UserBootstrap};

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

pub struct Params {
    data: Data,
    config_file: PathBuf,
    secret_key: Option<SecretKey>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config_file = data_path()?;

    let data: Data = if config_file.exists() {
        let contents = fs::read(&config_file)?;
        serde_json::from_slice(&contents)?
    } else {
        Data::default()
    };

    eprintln!("Current Data: {}", serde_json::to_string_pretty(&data)?);

    let params = Params {
        data,
        config_file,
        secret_key: None,
    };

    update(params)?;

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
    DecryptMaster,
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
            Self::DecryptMaster => KeyCode::Char('d'),
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
            KeyCode::Char('d') => Some(Self::DecryptMaster),
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
            Self::DecryptMaster => "Decrypt your Master Keypair (so we can operate with it)",
            Self::DestroyMaster => "DESTROY your Master Keypair (DANGER!)",
            Self::NewBootstrap => "Generate a new empty Bootstrap",
            Self::NewProfile => "Generate a new empty Profile",
            Self::NewKeySchedule => "Generate a new empty Key Schedule",
            Self::SaveAndExit => "Save and Quit",
            Self::ExitWithoutSaving => "Quit Without Saving",
        }
    }
}

pub fn options_from_params(params: &Params) -> Vec<MenuOption> {
    let mut options: Vec<MenuOption> = Vec::new();

    if let Some(_) = params.data.encrypted_master_key {
        if params.secret_key.is_some() {
            if let Some(ref _bootstrap) = params.data.bootstrap {
            } else {
                options.push(MenuOption::NewBootstrap);
            }

            if let Some(ref _profile) = params.data.profile {
            } else {
                options.push(MenuOption::NewProfile);
            }

            if let Some(ref _key_schedule) = params.data.profile {
            } else {
                options.push(MenuOption::NewKeySchedule);
            }

        } else {
            options.push(MenuOption::DecryptMaster);
        }
        options.push(MenuOption::DestroyMaster);
    } else {
        options.push(MenuOption::NewMaster);
    }

    options.push(MenuOption::SaveAndExit);
    options.push(MenuOption::ExitWithoutSaving);

    options
}

pub fn update(mut params: Params) -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout();

    'next_menu:
    loop {
        let options = options_from_params(&params);

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
                        if execute(option, &mut params)? {
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
    params: &mut Params,
) -> Result<bool,  Box<dyn Error>> {
    match option {
        MenuOption::NewMaster => {
            let secret_key = SecretKey::generate();
            let public_key = secret_key.public();
            let password = rpassword::prompt_password("Enter new password to encrypt your master key: ")?;
            println!("Encrypting...");
            let encrypted_secret_key = EncryptedSecretKey::from_secret_key(&secret_key, &password, 18);
            params.data.encrypted_master_key = Some(encrypted_secret_key);
            params.secret_key = Some(secret_key);
            println!("Master Key generated.");
            println!("Your Mosaic Identity is: {}", public_key);
        }
        MenuOption::DecryptMaster => {
            match &params.data.encrypted_master_key {
                Some(e) => {
                    let password = rpassword::prompt_password("Enter password to decrypt your master key: ")?;
                    println!("Decrypting...");
                    params.secret_key = Some(e.to_secret_key(&password)?);
                },
                None => panic!("Menu option should not have been there!"),
            }
        }
        MenuOption::DestroyMaster => {
            print!("Are you sure (type YES): ");
            io::stdout().flush()?;
            let stdin = io::stdin();
            let line = stdin.lock().lines().next().unwrap()?;
            if line.starts_with("YES") {
                params.data.encrypted_master_key = None;
                params.secret_key = None;
                println!("WARNING:  Master Key has been cleared. Takes effect when you save.");
                println!("WARNING:  If this was a mistake, break out now with ^C");
            } else {
                println!("Failed to confirm the operation. Taking no action.");
            }
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
            let contents: String = serde_json::to_string(&params.data)?;
            fs::write(&params.config_file, contents)?;
            println!("Saved.");
            return Ok(true);
        }
        MenuOption::ExitWithoutSaving => {
            return Ok(true);
        }
    }

    Ok(false)
}
