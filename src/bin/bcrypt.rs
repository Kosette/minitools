#![cfg(feature = "bcrypt")]

use bcrypt::{DEFAULT_COST, hash, verify};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <command> [args...]", &args[0]);
        eprintln!("\nCommands:");
        eprintln!("  hash <password>          - Generate bcrypt hash");
        eprintln!("  verify <password> <hash> - Verify password against hash");
        eprintln!("\nExamples:");
        eprintln!("  {} hash mypassword", &args[0]);
        eprintln!("  {} verify mypassword $2b$12$...", &args[0]);
        eprintln!("\nNote: Default cost factor is {}", DEFAULT_COST);
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "hash" | "h" => {
            if args.len() != 3 {
                eprintln!("Error: hash command requires exactly one password argument");
                eprintln!("Usage: {} hash <password>", &args[0]);
                std::process::exit(1);
            }
            
            let password = &args[2];
            
            match hash(password, DEFAULT_COST) {
                Ok(hashed_password) => {
                    println!("Hash: {}", hashed_password);
                    
                    // Verify the hash was created correctly
                    match verify(password, &hashed_password) {
                        Ok(true) => println!("✓ Verification successful"),
                        Ok(false) => eprintln!("✗ Verification failed (should not happen)"),
                        Err(e) => eprintln!("✗ Verification error: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("Error generating hash: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "verify" | "v" => {
            if args.len() != 4 {
                eprintln!("Error: verify command requires password and hash arguments");
                eprintln!("Usage: {} verify <password> <hash>", &args[0]);
                std::process::exit(1);
            }
            
            let password = &args[2];
            let hash = &args[3];
            
            match verify(password, hash) {
                Ok(true) => {
                    println!("✓ Password is valid!");
                    std::process::exit(0);
                }
                Ok(false) => {
                    println!("✗ Password is invalid!");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Error verifying password: {}", e);
                    std::process::exit(1);
                }
            }
        }
        // Backward compatibility: if first arg is not a command, treat it as password
        _ => {
            if args.len() != 2 {
                eprintln!("Error: Expected password as single argument");
                eprintln!("Usage: {} <password>", &args[0]);
                eprintln!("Or use: {} hash <password>", &args[0]);
                std::process::exit(1);
            }
            
            let password = &args[1];
            
            match hash(password, DEFAULT_COST) {
                Ok(hashed_password) => {
                    println!("哈希密码：{}", hashed_password);
                    
                    match verify(password, &hashed_password) {
                        Ok(true) => println!("密码有效！"),
                        Ok(false) => println!("密码无效！"),
                        Err(e) => eprintln!("验证错误: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("生成哈希错误: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
