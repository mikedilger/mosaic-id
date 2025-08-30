use std::error::Error;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crossterm::event::{Event, KeyCode};
use serde::{Deserialize, Serialize};

use mosaic_core::{EncryptedSecretKey, PublicKey, SecretKey, UserBootstrap};

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

    run_main_menu(params)?;

    Ok(())
}

fn data_path() -> Result<PathBuf, Box<dyn Error>> {
    let mut data_dir = normalize(
        dirs::data_dir().ok_or(Box::<dyn Error>::from("Cannot determine data directory"))?,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuOption {
    NewMaster,
    DecryptMaster,
    DestroyMaster,
    NewBootstrap,
    EditBootstrap,
    NewProfile,
    EditProfile,
    NewKeySchedule,
    EditKeySchedule,
    SaveAndExit,
    ExitWithoutSaving,

    // Bootstrap Submenu Options
    AddServer,
    RemoveServer,
    ChangeServerPriority,
    ChangeServerUsage,
    // Profile Submenu Options

    // Key Schedule Submenu Options
}

impl MenuOption {
    pub fn prompt(&self) -> &'static str {
        match self {
            Self::NewMaster => "Generate a new Master Keypair",
            Self::DecryptMaster => "Decrypt your Master Keypair (so we can operate with it)",
            Self::DestroyMaster => "DESTROY your Master Keypair (DANGER!)",
            Self::NewBootstrap => "Generate a new empty Bootstrap",
            Self::EditBootstrap => "Edit Bootstrap",
            Self::NewProfile => "Generate a new empty Profile",
            Self::EditProfile => "Edit Profile",
            Self::NewKeySchedule => "Generate a new empty Key Schedule",
            Self::EditKeySchedule => "Edit Key Schedule",
            Self::SaveAndExit => "Save and Quit",
            Self::ExitWithoutSaving => "Quit Without Saving",

            Self::AddServer => "Add a server",
            Self::RemoveServer => "Remove a server",
            Self::ChangeServerPriority => "Change priority of a server",
            Self::ChangeServerUsage => "Change usage of a server",
        }
    }
}

pub fn main_options_from_params(params: &Params) -> Vec<MenuOption> {
    let mut options: Vec<MenuOption> = Vec::new();

    if params.data.encrypted_master_key.is_some() {
        if params.secret_key.is_some() {
            if params.data.bootstrap.is_some() {
                options.push(MenuOption::EditBootstrap);
            } else {
                options.push(MenuOption::NewBootstrap);
            }

            if params.data.profile.is_some() {
                options.push(MenuOption::EditProfile);
            } else {
                options.push(MenuOption::NewProfile);
            }

            if params.data.key_schedule.is_some() {
                options.push(MenuOption::EditKeySchedule);
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

pub fn run_main_menu(mut params: Params) -> Result<(), Box<dyn Error>> {
    loop {
        let options = main_options_from_params(&params);
        let exit = run_menu_once(options, &mut params, 0)?;
        if exit {
            return Ok(());
        }
    }
}

pub fn run_menu_once(
    options: Vec<MenuOption>,
    params: &mut Params,
    indent: usize,
) -> Result<bool, Box<dyn Error>> {
    let indent = "  ".repeat(indent);

    // Print the menu
    let mut stdout = io::stdout();
    println!("{}\n-----------------------------------", indent);
    for (i, option) in options.iter().enumerate() {
        println!("{}{}) {}", indent, i, option.prompt());
    }
    print!("{}> ", indent);
    stdout.flush()?;

    // Handle one command from the menu
    loop {
        if let Event::Key(key_event) = crossterm::event::read()?
            && let KeyCode::Char(c) = key_event.code
            && let Some(digit) = c.to_digit(10)
        {
            let index = digit as usize;
            if index < options.len() {
                println!();
                let exit = execute(options[index], params)?;
                return Ok(exit);
            }
        }
    }
}

// Execute the chosen menu option.
//
// Return value 'true' means exit
pub fn execute(option: MenuOption, params: &mut Params) -> Result<bool, Box<dyn Error>> {
    match option {
        MenuOption::NewMaster => {
            let secret_key = SecretKey::generate();
            let public_key = secret_key.public();
            let password =
                rpassword::prompt_password("Enter new password to encrypt your master key: ")?;
            println!("Encrypting...");
            let encrypted_secret_key =
                EncryptedSecretKey::from_secret_key(&secret_key, &password, 18);
            params.data.encrypted_master_key = Some(encrypted_secret_key);
            params.secret_key = Some(secret_key);
            println!("Master Key generated.");
            println!("Your Mosaic Identity is: {}", public_key);
        }
        MenuOption::DecryptMaster => match &params.data.encrypted_master_key {
            Some(e) => {
                let password =
                    rpassword::prompt_password("Enter password to decrypt your master key: ")?;
                println!("Decrypting...");
                params.secret_key = Some(e.to_secret_key(&password)?);
            }
            None => panic!("Menu option should not have been there!"),
        },
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
            params.data.bootstrap = Some(UserBootstrap::new());
        }
        MenuOption::EditBootstrap => {
            println!("Not implemented");
        }
        MenuOption::AddServer => {
            let _max_index = params.data.bootstrap.as_ref().unwrap().len();

            print!("Enter server public key: ");
            io::stdout().flush()?;
            let stdin = io::stdin();
            let line = stdin.lock().lines().next().unwrap()?;

            if let Ok(_pk) = PublicKey::from_printable(&line) {
                println!("NOT YET IMPLEMENTED");
            } else {
                println!("Not understood.");
            }

            // params.data.bootstrap.add_server(pk, usage, priority)
            println!("Add Server - not yet implemented.");
        }
        MenuOption::RemoveServer => {
            println!("Remove Server - not yet implemented.");
        }
        MenuOption::ChangeServerPriority => {
            println!("Change Server Priority - not yet implemented.");
        }
        MenuOption::ChangeServerUsage => {
            println!("Change Server Usage - not yet implemented.");
        }
        MenuOption::NewProfile => {
            println!("New Profile - not yet implemented.");
        }
        MenuOption::EditProfile => {
            println!("Not implemented");
        }
        MenuOption::NewKeySchedule => {
            println!("New Key Schedule - not yet implemented.");
        }
        MenuOption::EditKeySchedule => {
            println!("Not implemented");
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
