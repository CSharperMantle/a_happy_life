#![forbid(unsafe_code)]
#![deny(clippy::all)]

use std::env;
use std::fs::File;
use std::io::Write;
use std::path;
use std::process;
use std::str;

use tempfile::tempdir;

const MAX_LINES: usize = 30;
const MAX_CHARS_PER_LINE: usize = 3000;

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
#![forbid(unsafe_code)]

extern crate my_proxy;
extern crate alloc;
use alloc::string::String;

include!("part_1.in");

fn main() {
    let x = include!("part_2.in");
    let _secret = String::from("@@FLAG@@");
    my_proxy::print(&x);
}"#;

fn read_into_file(base: &path::Path, filename: &str) {
    let filepath = base.join(filename);
    let mut f = File::create(filepath).unwrap_or_else(|_| panic!("failed to create {}", filename));
    println!(
        "Now supply the content of `{}` in no more than {} lines and {} chars per line.",
        filename, MAX_LINES, MAX_CHARS_PER_LINE
    );
    println!("End your input with a single line containing \"[END]\".");
    for _ in 0..MAX_LINES {
        let mut line = String::new();
        std::io::stdin()
            .read_line(&mut line)
            .expect("failed to read line from stdin");
        if line.trim() == "[END]" || line.len() > MAX_CHARS_PER_LINE {
            break;
        }
        writeln!(f, "{}", &line).unwrap_or_else(|_| panic!("failed to write to {}", filename));
    }
}

fn main() {
    let out = process::Command::new("rustc")
        .arg("-V")
        .output()
        .expect("failed to check rustc version");
    if !out.status.success() {
        panic!("no rustc found on PATH");
    }
    let rustc_ver = str::from_utf8(&out.stdout).expect("failed to decode stdout as utf-8");

    let flag = env::var("FLAG").expect("no FLAG env var found");
    let tmp_dir = tempdir().expect("failed to create temp dir");
    let tmp_dir_path = tmp_dir.path();

    let proxy_path = tmp_dir_path.join("my_proxy.rs");
    let mut f_proxy = File::create(proxy_path).expect("failed to create my_proxy.rs");
    writeln!(f_proxy, "{}", ALLOC_FILE_CONTENT).expect("failed to write to my_proxy.rs");
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
        .expect("failed to run rustc");
    if !out.status.success() {
        panic!("failed to compile proxy lib");
    }

    println!("Rust can promise you a happy life... but does this proposition always hold?");
    println!("rustc version: {}", rustc_ver);
    println!(
        "Fill in this code:\n\n=====BEGIN====={}\n=====END=====\n",
        MAIN_FILE_CONTENT
    );
    println!("where there are {} chars in \"@@FLAG@@\".", flag.len());
    println!();

    read_into_file(tmp_dir_path, "part_1.in");
    read_into_file(tmp_dir_path, "part_2.in");

    let mut f_src = File::create(tmp_dir_path.join("main.rs")).expect("failed to create main.rs");
    writeln!(f_src, "{}", MAIN_FILE_CONTENT.replace("@@FLAG@@", &flag))
        .expect("failed to write to main.rs");
    drop(f_src);

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
            "main.rs",
        ])
        .current_dir(tmp_dir.path())
        .env_remove("FLAG")
        .output()
        .expect("failed to run rustc");
    if !out.status.success() {
        println!("Oops...");
        process::exit(1);
    }

    let exe_path = tmp_dir_path.join("main.exe");
    let out = process::Command::new(exe_path)
        .current_dir(tmp_dir.path())
        .env_clear()
        .output()
        .expect("failed to run built exe");
    if !out.status.success() {
        println!("Oh no, please try again...");
        process::exit(1);
    }
    println!(
        "{}",
        str::from_utf8(&out.stdout).expect("failed to decode stdout as utf-8")
    );
}
