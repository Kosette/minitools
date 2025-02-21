use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use glob::glob;
use sha2::{Digest, Sha256};

fn main() {
    let mut duplicates = HashMap::new();

    let args: Vec<String> = env::args().collect();

    for pattern in &args[1..] {
        for entry in glob(pattern).unwrap() {
            let path = entry.unwrap();

            if path.is_dir() {
                process_dir(&path, &mut duplicates);
            } else if path.is_file() {
                let hash = get_sha256(&path);
                update_duplicates(hash, &path, &mut duplicates);
            }
        }
    }

    print_duplicates(&duplicates);
}

fn process_dir(dir: &Path, duplicates: &mut HashMap<String, Vec<String>>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            process_dir(&path, duplicates);
        } else {
            let hash = get_sha256(&path);
            update_duplicates(hash, &path, duplicates);
        }
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
    for (hash, names) in duplicates {
        if names.len() > 1 {
            println!("重复文件({}):", hash);
            for name in names {
                println!(" - {}", name);
            }
        }
    }
}

fn get_sha256(path: &Path) -> String {
    let data = fs::read(path).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&data);

    let hash = hasher.finalize();

    format!("{:x}", hash)
}
