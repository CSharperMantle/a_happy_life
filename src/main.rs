#![forbid(unsafe_code)]
#![deny(clippy::all)]

mod err;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path;
use std::process;
use std::str;

const MAX_LINES: usize = 30;
const MAX_CHARS_PER_LINE: usize = 120;

const ALLOC_FILE_CONTENT: &str = r#"
pub use std::alloc::System;

#[global_allocator]
static GLOBAL: System = System;

pub fn print(s: &str) {
    println!("my_proxy::print(s: &str) says: {}", s);
}
"#;

const MAIN_FILE_CONTENT: &str = r#"
#![no_std]
#![allow(clippy::all)]
#![forbid(unsafe_code)]
#![forbid(clippy::disallowed_macros)]

extern crate my_proxy;
extern crate alloc;
use alloc::string::String;

include!("part_1.in");

fn main() {
    let x = include!("part_2.in");
    let _secret = String::from("@@FLAG@@");
    my_proxy::print(&x);
}"#;

const CLIPPY_CONF_FILE_CONTENT: &str = r#"
disallowed-macros = [
    "std::include_str",
    "core::include_str",
    "std::include_bytes",
    "core::include_bytes",
]
"#;

fn read_into_file(base: &path::Path, filename: &str) -> Result<(), err::Err> {
    let filepath = base.join(filename);
    let mut f = File::create(filepath)
        .map_err(|e| err::Err::from_error(Box::new(e), format!("failed to create {}", filename)))?;
    println!(
        "Now supply the content of `{}` in no more than {} lines and {} chars per line.",
        filename, MAX_LINES, MAX_CHARS_PER_LINE
    );
    println!("End your input with a single line containing \"[END]\".");
    for _ in 0..MAX_LINES {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).map_err(|e| {
            err::Err::from_error(Box::new(e), "failed to read from stdin".to_string())
        })?;
        if line.trim() == "[END]" || line.len() > MAX_CHARS_PER_LINE {
            break;
        }
        writeln!(f, "{}", &line).map_err(|e| {
            err::Err::from_error(Box::new(e), format!("failed to write to {}", filename))
        })?;
    }
    Ok(())
}

fn do_interaction(tmp_dir: &tempfile::TempDir) -> Result<String, err::Err> {
    let out = process::Command::new("rustc")
        .arg("-V")
        .output()
        .map_err(|e| {
            err::Err::from_error(Box::new(e), "failed to check rustc version".to_string())
        })?;
    if !out.status.success() {
        return Err(err::Err::from_msg_internal("no rustc found".to_string()));
    }
    let rustc_ver = str::from_utf8(&out.stdout).map_err(|e| {
        err::Err::from_error(Box::new(e), "failed to decode stdout as utf-8".to_string())
    })?;

    let out = process::Command::new("clippy-driver")
        .arg("-V")
        .output()
        .map_err(|e| {
            err::Err::from_error(Box::new(e), "failed to check clippy version".to_string())
        })?;
    if !out.status.success() {
        return Err(err::Err::from_msg_internal("no clippy found".to_string()));
    }
    let clippy_ver = str::from_utf8(&out.stdout).map_err(|e| {
        err::Err::from_error(Box::new(e), "failed to decode stdout as utf-8".to_string())
    })?;

    let flag = env::var("FLAG")
        .map_err(|e| err::Err::from_error(Box::new(e), "no FLAG env var found".to_string()))?;
    let tmp_dir_path = tmp_dir.path();

    let proxy_path = tmp_dir_path.join("my_proxy.rs");
    let mut f_proxy = File::create(proxy_path).map_err(|e| {
        err::Err::from_error(Box::new(e), "failed to create my_proxy.rs".to_string())
    })?;
    writeln!(f_proxy, "{}", ALLOC_FILE_CONTENT).map_err(|e| {
        err::Err::from_error(Box::new(e), "failed to write to my_proxy.rs".to_string())
    })?;
    drop(f_proxy);
    let out = process::Command::new("rustc")
        .args([
            "--edition",
            "2021",
            "-O",
            "--crate-type=lib",
            "-o",
            "libmy_proxy.rlib",
            "my_proxy.rs",
        ])
        .current_dir(tmp_dir.path())
        .env_remove("FLAG")
        .output()
        .map_err(|e| {
            err::Err::from_error(
                Box::new(e),
                "failed to run rustc for my_proxy.rs".to_string(),
            )
        })?;
    if !out.status.success() {
        return Err(err::Err::from_msg_internal(
            "failed to compile proxy lib".to_string(),
        ));
    }

    let clippy_conf_path = tmp_dir_path.join("clippy.conf");
    let mut f_clippy_conf = File::create(clippy_conf_path).map_err(|e| {
        err::Err::from_error(Box::new(e), "failed to create clippy.conf".to_string())
    })?;
    writeln!(f_clippy_conf, "{}", CLIPPY_CONF_FILE_CONTENT).map_err(|e| {
        err::Err::from_error(Box::new(e), "failed to write to clippy.conf".to_string())
    })?;
    drop(f_clippy_conf);

    println!("Rust can promise you a happy life... but does this proposition always hold?");
    println!("rustc version: {}", rustc_ver);
    println!("clippy version: {}", clippy_ver);
    println!(
        "Fill in this code:\n\n=====BEGIN====={}\n=====END=====\n",
        MAIN_FILE_CONTENT
    );
    println!("where there are {} chars in \"@@FLAG@@\".", flag.len());
    println!();

    read_into_file(tmp_dir_path, "part_1.in")?;
    read_into_file(tmp_dir_path, "part_2.in")?;

    let main_uuid = uuid::Uuid::new_v4();
    let main_name = format!("main_{}.rs", main_uuid);

    let mut f_src = File::create(tmp_dir_path.join(&main_name))
        .map_err(|e| err::Err::from_error(Box::new(e), "failed to create main.rs".to_string()))?;
    writeln!(f_src, "{}", MAIN_FILE_CONTENT.replace("@@FLAG@@", &flag))
        .map_err(|e| err::Err::from_error(Box::new(e), "failed to write to main.rs".to_string()))?;
    drop(f_src);


    let out = process::Command::new("clippy-driver")
        .args([
            "--edition",
            "2021",
            "-C",
            "opt-level=0",
            "--extern",
            "my_proxy=libmy_proxy.rlib",
            "-o",
            "main.exe",
            &main_name,
        ])
        .current_dir(tmp_dir.path())
        .env_remove("FLAG")
        .output()
        .map_err(|e| {
            err::Err::from_error(Box::new(e), "failed to run clippy for main.rs".to_string())
        })?;
    if !out.status.success() {
        return Err(err::Err::from_user("Sorry...".to_string()));
    }

    let out = process::Command::new("rustc")
        .args([
            "--edition",
            "2021",
            "-C",
            "opt-level=0",
            "--extern",
            "my_proxy=libmy_proxy.rlib",
            "-o",
            "main.exe",
            &main_name,
        ])
        .current_dir(tmp_dir.path())
        .env_remove("FLAG")
        .output()
        .map_err(|e| {
            err::Err::from_error(Box::new(e), "failed to run rustc for main.rs".to_string())
        })?;
    if !out.status.success() {
        return Err(err::Err::from_user("Oops...".to_string()));
    }

    let exe_path = tmp_dir_path.join("main.exe");
    let out = process::Command::new(exe_path)
        .current_dir(tmp_dir.path())
        .env_clear()
        .output()
        .map_err(|e| err::Err::from_error(Box::new(e), "failed to run target exe".to_string()))?;
    if !out.status.success() {
        return Err(err::Err::from_user("Oh no, try again...".to_string()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn main() -> Result<(), err::Err> {
    let tmp_dir = tempfile::tempdir()
        .map_err(|e| err::Err::from_error(Box::new(e), "failed to create temp dir".to_string()))?;
    let result = do_interaction(&tmp_dir)
        .map(|s| println!("[+] {}", s))
        .map_err(|e| {
            match e.err_type {
                err::ErrType::User => println!("[-] {}", e.msg),
                err::ErrType::Internal => eprintln!("{}", e),
            };
            e
        });
    tmp_dir
        .close()
        .map_err(|e| err::Err::from_error(Box::new(e), "failed to remove temp dir".to_string()))?;
    result
}
