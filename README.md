# static_str_ops

The static_str_ops crate solves a longstanding issue about how to
perform non-const string operations, e.g., `format!()`, `concat!()`, etc.
and return static string, i.e., `&'static str`.

Internally, the crate uses a global static HashSet to store all the
static strings, and return the reference to the string in the HashSet
if the string has been staticized before.

[![crates.io](https://img.shields.io/crates/v/static_str_ops.svg)](https://crates.io/crates/static_str_ops)
[![Downloads](https://img.shields.io/crates/d/static_str_ops)](https://crates.io/crates/static_str_ops)
[![Docs.rs](https://img.shields.io/docsrs/static_str_ops/latest)](https://docs.rs/static_str_ops/latest/static_str_ops/)
[![Github Actions](https://github.com/unsafecoerce/static_str_ops/actions/workflows/ci.yml/badge.svg)](https://github.com/unsafecoerce/static_str_ops/actions/workflows/ci.yml)

APIs
----

This create provides the following macros and functions:

- `staticize(s: &str) -> &'static str`

  Convert a string to a static string. If the string has been staticized
  before, return the reference to the string in the HashSet.
  This function is the most basic usage of this crate, e.g.,

  ```rust
  use static_str_ops::staticize;
  let s: &'static str = staticize(&String::from("hello world!"));
  ```

- `is_staticized(s: &str) -> bool`

  Check if a string has been staticized before.

- `static_concat!(s1: expr, s2: expr, ...) -> &'static str`

  Concatenate multiple strings into a static string. The arguments can
  be either a string literal.
  Like `concat!()`, but returns a static string.

- `static_format!(s: expr, ...) -> &'static str`

  Format a string into a static string. The arguments can be whatever
  the builtin macro `format!()` can accept.
  Like `format!()`, but returns a static string.

- `staticize_once!(expr: expr) -> &'static str`

  Similar to staticize(), but the expr will be evaluated only once. Under
  the hood, `std::sync::Once` is used.
  The function will be useful if you have a function that want to return
  a static string, while the generate logic is non-trivial, and you want
  this process only happen once, e.g.,

  ```rust
  use static_str_ops::*;
  let make_string = || {
      staticize_once!({
          let s = "";  // can be some expensive computation
          s
      })
  };

  let s1: &'static str = make_string();
  let s2: &'static str = make_string();
  ```

  When you call `make_string()` for multiple times, the body will be
  guaranteed to be evaluated only once.

License
-------

This project is licensed under the BSD-3 Clause license ([LICENSE](LICENSE) or
http://opensource.org/licenses/BSD-3-Clause).
