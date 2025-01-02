#![cfg(feature = "bcrypt")]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use bcrypt::{hash, verify, DEFAULT_COST};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!(
            "Usage: {} <passwd>\n\nError: Accept ONE and ONLY ONE argument",
            &args[0]
        );
        std::process::exit(-1);
    }

    // 要加密的密码
    let password = &args[1];

    // 使用默认成本因子生成 bcrypt 哈希
    let hashed_password = hash(password, DEFAULT_COST).unwrap();

    // 打印哈希密码
    println!("哈希密码：{}", hashed_password);

    // 验证密码是否与哈希匹配
    let is_valid = verify(password, &hashed_password).unwrap();

    // 打印验证结果
    if is_valid {
        println!("密码有效！");
    } else {
        println!("密码无效！");
    }
}
