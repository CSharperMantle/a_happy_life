#![forbid(unsafe_code)]

use std::env;
use std::fs::File;
use std::io::Write;
use std::process;
use std::str;

use tempfile::tempdir;

/*
 const EXP_1: &'static str = r#"

 fn aux<'a, 'b, T>(_: Option<&'a &'b ()>, v: &'b T) -> &'a T { v }

 fn exp<'c, 'd, T>(x: &'c T) -> &'d T {
     let f: fn(Option<&'d &'d ()>, &'c T) -> &'d T = aux;
     f(None, x)
 }

 "#;

 const EXP_2: &'static str = r#"

 let local = String::from("aaaabaaacaaadaaa"); // <-- Same length as flag
 exp(&local)

 "#;
*/

const ALLOC_FILE_CONTENT: &'static str = r#"
#![feature(allocator_api)]
#![feature(alloc)]

pub use std::alloc::System;

#[global_allocator]
static GLOBAL: System = System;

pub fn print(s: &str) {
    println!("s = {}", s);
}
"#;

fn main() {
    let out = process::Command::new("rustc")
        .arg("-V")
        .env_clear()
        .output()
        .expect("failed to check rustc version");
    if !out.status.success() {
        eprintln!("no rustc found");
        process::exit(1);
    }
    let rustc_version = str::from_utf8(&out.stdout).expect("failed to decode stdout as utf-8");

    let flag = env::var("FLAG").expect("no FLAG env var found");
    let tmp_dir = tempdir().expect("failed to create temp dir");

    let alloc_path = tmp_dir.path().join("my_alloc.rs");
    let mut alloc_file = File::create(alloc_path).expect("failed to create alloc file");
    writeln!(alloc_file, "{}", ALLOC_FILE_CONTENT).expect("failed to write to alloc file");
    let out = process::Command::new("rustc")
        .args([
            "--edition",
            "2021",
            "-C",
            "opt-level=0",
            "--crate-type=lib",
            "-o",
            "libmy_alloc.rlib",
            "my_alloc.rs",
        ])
        .current_dir(tmp_dir.path())
        .output()
        .expect("failed to run rustc");
    if !out.status.success() {
        println!("Oops...");
        process::exit(1);
    }

    let src_path = tmp_dir.path().join("main.rs");
    let mut src_file = File::create(src_path).expect("failed to src file");

    println!("Welcome to this challenge!");
    println!("rustc version: {}", rustc_version);
    println!(
        "Fill in this code:\n\n=====BEGIN====={}\n=====END=====\n",
        r#"
#![no_std]
#![feature(alloc)]
#![forbid(unsafe_code)]

extern crate my_alloc;
extern crate alloc;
use alloc::string::String;

[YOUR CODE PART 1]

fn main() {
    let x = {
        [YOUR CODE PART 2]
    };
    let _secret = String::from("[???]");
    my_alloc::print(&x);
}"#
    );
    println!("where there are {} chars in [???].", flag.len());
    println!("Now fill in PART 1, end with a single line containing \"[END]\".");
    let mut part_1 = String::new();
    loop {
        let mut this_line = String::new();
        std::io::stdin()
            .read_line(&mut this_line)
            .expect("failed to read line");
        if this_line.trim() == "[END]" {
            break;
        }
        part_1.push_str(&this_line);
    }
    println!("Now fill in PART 2, end with a single line containing \"[END]\".");
    let mut part_2 = String::new();
    loop {
        let mut this_line = String::new();
        std::io::stdin()
            .read_line(&mut this_line)
            .expect("failed to read line");
        if this_line.trim() == "[END]" {
            break;
        }
        part_2.push_str(&this_line);
    }

    let src_content = format!(
        r#"
#![no_std]
#![feature(alloc)]
#![forbid(unsafe_code)]

extern crate my_alloc;
extern crate alloc;
use alloc::string::String;

{}

fn main() {{
    let x = {{
        {}
    }};
    let y = String::from("{}");
    my_alloc::print(&x);
}}"#,
        part_1, part_2, flag
    );

    writeln!(src_file, "{}", src_content).expect("failed to write to src file");

    let out = process::Command::new("rustc")
        .args([
            "--edition",
            "2021",
            "-C",
            "opt-level=0",
            "--extern",
            "my_alloc=libmy_alloc.rlib",
            "-o",
            "main.exe",
            "main.rs",
        ])
        .current_dir(tmp_dir.path())
        .output()
        .expect("failed to run rustc");
    if !out.status.success() {
        println!("Oops...");
        process::exit(1);
    }

    let exe_path = tmp_dir.path().join("main.exe");
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
