mod paths;

use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let paths = paths::paths()?;
    eprintln!("Mosaic dir = {}", paths.base.display());

    if !fs::exists(&paths.master_key)? {
        eprintln!("NO master key file");
    }

    if !fs::exists(&paths.bootstrap)? {
        eprintln!("NO bootstrap file");
    }

    if !fs::exists(&paths.profile)? {
        eprintln!("NO profile file");
    }

    if !fs::exists(&paths.key_schedule)? {
        eprintln!("NO key schedule file");

    }

    Ok(())
}
