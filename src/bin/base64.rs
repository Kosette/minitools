#![cfg(feature = "base64")]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use base64::prelude::*;
use std::env;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <encode|decode> [input_string]", args[0]);
        eprintln!("       Or provide input via standard input.");
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
        "encode" => {
            let encoded = BASE64_STANDARD.encode(input);
            println!("{}", encoded);
        }
        "decode" => match BASE64_STANDARD.decode(input) {
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
