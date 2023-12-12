# a_happy_life

Challenge based on rust-lang/rust#25860.

## Idea

This challenge exploits a unsoundness hole discovered in Rust lifetime deduction system. When a type like `&'a &'b ()` is encountered, the compiler implicitly relies upon `'b: 'a`, which could be broken by contravariance.

See rust-lang/rust#25860 for the original issue. A more theoretical yet neater explanation can be found on https://counterexamples.org/nearly-universal.html.

## Solution

Part 1:

```rust
/*
 * Create "any-to-any" lifetime cast function.
 */

fn aux<'a, 'b, T>(_: Option<&'a &'b ()>, v: &'b T) -> &'a T { v }

fn exp<'c, 'd, T>(x: &'c T) -> &'d T {
    let f: fn(Option<&'d &'d ()>, &'c T) -> &'d T = aux;
    f(None, x)
}
```

Part 2:

```rust
/*
 * Create a local variable whose memory location
 * may coincide with `_secret`.
 * 
 * The length of the `&'static str` literal should be
 * the same as `_secret`.
 */
let local = String::from("aaaabaaacaaadaaa");
/*
 * Scramble its lifetime, and make it dangle after
 * it leaves its scope.
 */
exp(&local)
```

### Explanation

The value `local` is dropped once it leaves the scope. But as the compiler thinks that `x` has a lifetime longer than `local`, the reference `x` to `local` dangles, which can then possibly (in fact almost always on `-C opt_level=0`) refer to the memory space allocated for `_secret`, creating a use-after-free scenario.

## Requirements

* `rustc` on PATH, tested with `rustc 1.76.0-nightly (d86d65bbc 2023-12-10)`
* Flag in environment variable `FLAG`
