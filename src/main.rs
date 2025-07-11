mod dir;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mosaic_dir = dir::mosaic_dir()?;
    eprintln!("Mosaic dir = {}", mosaic_dir.display());
    Ok(())
}

