#![cfg(feature = "b64")]

use base64::prelude::*;
use std::env;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <encode|decode> [input_string]", args[0]);
        eprintln!("       Or provide input via standard input.");
        eprintln!("\nExamples:");
        eprintln!("  {} encode \"Hello, World!\"", args[0]);
        eprintln!("  {} e \"Hello, World!\"", args[0]);
        eprintln!("  {} decode SGVsbG8sIFdvcmxkIQ==", args[0]);
        eprintln!("  {} d SGVsbG8sIFdvcmxkIQ==", args[0]);
        eprintln!("  echo \"Hello\" | {} encode", args[0]);
        eprintln!("\nShortcuts:");
        eprintln!("  e = encode");
        eprintln!("  d = decode");
        std::process::exit(1);
    }

    let operation = args[1].as_str();
    let input = if args.len() > 2 {
        args[2..].join(" ")
    } else {
        // Read from standard input if no input string is provided
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer.trim().to_string()
    };

    match operation {
        "encode" | "e" => {
            let encoded = BASE64_STANDARD.encode(input);
            println!("{}", encoded);
        }
        "decode" | "d" => match BASE64_STANDARD.decode(input) {
            Ok(decoded) => match String::from_utf8(decoded) {
                Ok(decoded_str) => println!("{}", decoded_str),
                Err(_) => eprintln!("Decoded data is not valid UTF-8"),
            },
            Err(_) => eprintln!("Failed to decode Base64 input"),
        },
        _ => {
            eprintln!("Invalid operation. Use 'encode' or 'decode'.");
            std::process::exit(1);
        }
    }
}
