#![cfg(feature = "dupfinder")]

use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

use glob::glob;
use sha2::{Digest, Sha256};

const BUFFER_SIZE: usize = 1024 * 1024; // 1MB buffer for streaming

fn main() {
    let mut duplicates = HashMap::new();

    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <pattern> [pattern2] [pattern3] ...", args[0]);
        eprintln!("\nExamples:");
        eprintln!("  {} *", args[0]);
        eprintln!("  {} *.jpg *.png", args[0]);
        eprintln!("  {} /path/to/directory/*", args[0]);
        eprintln!("\nFinds duplicate files by calculating SHA256 hashes.");
        std::process::exit(1);
    }

    for pattern in &args[1..] {
        match glob(pattern) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(path) => {
                            if path.is_dir() {
                                process_dir(&path, &mut duplicates);
                            } else if path.is_file() {
                                match get_sha256(&path) {
                                    Ok(hash) => update_duplicates(hash, &path, &mut duplicates),
                                    Err(e) => eprintln!("Error hashing {}: {}", path.display(), e),
                                }
                            }
                        }
                        Err(e) => eprintln!("Error reading path: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("Invalid pattern '{}': {}", pattern, e),
        }
    }

    print_duplicates(&duplicates);
}

fn process_dir(dir: &Path, duplicates: &mut HashMap<String, Vec<String>>) {
    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_dir() {
                            process_dir(&path, duplicates);
                        } else {
                            match get_sha256(&path) {
                                Ok(hash) => update_duplicates(hash, &path, duplicates),
                                Err(e) => eprintln!("Error hashing {}: {}", path.display(), e),
                            }
                        }
                    }
                    Err(e) => eprintln!("Error reading directory entry: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Error reading directory {}: {}", dir.display(), e),
    }
}

fn update_duplicates(hash: String, path: &Path, duplicates: &mut HashMap<String, Vec<String>>) {
    if let Some(names) = duplicates.get_mut(&hash) {
        names.push(path.display().to_string());
    } else {
        duplicates.insert(hash, vec![path.display().to_string()]);
    }
}

fn print_duplicates(duplicates: &HashMap<String, Vec<String>>) {
    let mut total_files = 0;
    let mut duplicate_groups = 0;
    let mut duplicate_files = 0;
    
    for (hash, names) in duplicates {
        total_files += names.len();
        if names.len() > 1 {
            duplicate_groups += 1;
            duplicate_files += names.len();
            println!("\nDuplicate files (Hash: {}):", hash);
            for name in names {
                println!("  - {}", name);
            }
        }
    }
    
    println!("\n===== Summary =====");
    println!("Total files scanned: {}", total_files);
    println!("Duplicate groups found: {}", duplicate_groups);
    println!("Total duplicate files: {}", duplicate_files);
    if duplicate_files > 0 {
        println!("Space that could be freed by removing duplicates: estimate {} files", duplicate_files - duplicate_groups);
    }
}

fn get_sha256(path: &Path) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; BUFFER_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();

    Ok(format!("{:x}", hash))
}
