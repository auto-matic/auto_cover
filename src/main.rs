use anyhow::{anyhow, Result};
use rayon::prelude::*;
use std::io::{self, Read};
use std::{
    fs::{self, DirEntry, ReadDir},
    path::PathBuf,
};

const SIZE: u32 = 500;

fn main() -> Result<()> {
    // find all cover.png
    // convert to cover.bmp 500x500

    println!("===== Automatics cover art converter =====");
    let path = match fs::read_to_string("path.txt") {
        Ok(p) => p,
        Err(_) => {
            return Err(anyhow!(
                "File containing root path ('path.txt') is missing."
            ))
        }
    };
    println!("Root path: {}\n", path);

    let dir = fs::read_dir(path)?;

    println!("Finding covers...");
    let covers = find_covers(dir);
    println!("Found {}", covers.len());

    println!("\nResizing images...");
    process_covers(covers);

    println!("\nDone. Press Enter to exit.");
    let _ = io::stdin().read(&mut [0u8])?;
    Ok(())
}

fn find_covers(dir: ReadDir) -> Vec<PathBuf> {
    dir.into_iter()
        .par_bridge()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| process_entry(entry).ok())
        .flatten()
        .collect()
}

fn process_entry(entry: DirEntry) -> Result<Vec<PathBuf>> {
    let file_type = entry.file_type()?;
    if file_type.is_dir() {
        Ok(find_covers(fs::read_dir(entry.path())?))
    } else if file_type.is_file() {
        let is_cover = if let Ok(name) = entry.file_name().into_string() {
            //name.ends_with(".png")
            name.starts_with("cover") && !name.ends_with("bmp")
        } else {
            false
        };
        if is_cover {
            let path = entry.path();
            let bmp_path = path.with_extension("bmp");
            if fs::exists(&bmp_path)? {
                let img = image::open(bmp_path)?;
                if img.width() <= 600 && img.height() <= 600 {
                    //println!("Skipped {}", path.display());
                    return Ok(vec![]);
                }
            }
            Ok(vec![entry.path()])
        } else {
            Err(anyhow!("Entry is not cover art."))
        }
    } else {
        Err(anyhow!("Entry is neither file nor directory."))
    }
}

fn process_covers(paths: Vec<PathBuf>) {
    paths
        .into_par_iter()
        .map(|p| match process_cover(&p) {
            Ok(_) => println!("Converted {}", p.display()),
            Err(e) => println!("Error while converting {}\n\t{}", p.display(), e),
        })
        .count();
}

fn process_cover(path: &PathBuf) -> Result<()> {
    let img = image::open(path)?;
    img.resize(SIZE, SIZE, image::imageops::FilterType::Lanczos3)
        .save(path.with_extension("bmp"))?;
    Ok(())
}
