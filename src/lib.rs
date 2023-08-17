//! The static_str_ops crate solves a longstanding issue about how to
//! perform non-const string operations, e.g., `format!()`, `concat!()`, etc.
//! and return static string, i.e., `&'static str`.
//!
//! Internally, the crate uses a global static HashSet to store all the
//! static strings, and return the reference to the string in the HashSet
//! if the string has been staticized before.
//!
//! This create provides the following macros and functions:
//!
//! - `staticize(s: &str) -> &'static str`
//!
//!   Convert a string to a static string. If the string has been staticized
//!   before, return the reference to the string in the HashSet.
//!
//!   This function is the most basic usage of this crate, e.g.,
//!
//!   ```rust
//!   use static_str_ops::staticize;
//!
//!   let s: &'static str = staticize(&String::from("hello world!"));
//!   ```
//!
//! - `is_staticized(s: &str) -> bool`
//!
//!   Check if a string has been staticized before.
//!
//! - `destaticize(s: &str) -> bool`
//!
//!   Remove a static string from the internal HashSet. Return `true` if was present.
//!
//! - `static_concat!(s1: expr, s2: expr, ...) -> &'static str`
//!
//!   Concatenate multiple strings into a static string. The arguments can
//!   be either a string literal.
//!
//!   Like `concat!()`, but returns a static string.
//!
//! - `static_format!(s: expr, ...) -> &'static str`
//!
//!   Format a string into a static string. The arguments can be whatever
//!   the builtin macro `format!()` can accept.
//!
//!   Like `format!()`, but returns a static string.
//!
//! - `staticize_once!(expr: expr) -> &'static str`
//!
//!   Similar to staticize(), but the expr will be evaluated only once. Under
//!   the hood, `std::sync::Once` is used.
//!
//!   The function will be useful if you have a function that want to return
//!   a static string, while the generate logic is non-trivial, and you want
//!   this process only happen once, e.g.,
//!
//!   ```rust
//!   use static_str_ops::*;
//!
//!   let make_string = || {
//!       staticize_once!({
//!           let s = "";  // can be some expensive computation
//!           s
//!       })
//!   };
//!
//!   let s1: &'static str = make_string();
//!   let s2: &'static str = make_string();
//!   ```
//!
//!   When you call `make_string()` for multiple times, the body will be
//!   guaranteed to be evaluated only once.

#![allow(non_upper_case_globals)]

use std::collections::HashSet;
use std::sync::Mutex;

use lazy_static::lazy_static;

/// Re-export the gensym symbol to avoid introducing a new dependency (gensym)
/// in callers.
pub use gensym;

lazy_static! {
    static ref STATIC_STRINGS: Mutex<HashSet<&'static str>> = Mutex::new(HashSet::new());
}

/// Converts a string slice to a static string slice.
///
/// This function takes a string slice and returns a static string slice with the same contents.
/// If the same string slice has been previously converted, the function returns the previously
/// converted static string slice. Otherwise, it creates a new static string slice and returns it.
///
/// # Arguments
///
/// * `s` - A string slice to be converted to a static string slice.
///
/// # Examples
///
/// ```
/// use static_str_ops::staticize;
///
/// let s = "hello";
/// let static_s = staticize(s);
///
/// assert_eq!(static_s, "hello");
/// ```
pub fn staticize<T: Into<String>>(s: T) -> &'static str {
    let s: Box<String> = Box::new(s.into());
    let mut strings = STATIC_STRINGS.lock().unwrap();
    match strings.get(s.as_str()) {
        Some(s) => s,
        None => {
            let s = Box::leak(s);
            strings.insert(s);
            s
        }
    }
}

/// Checks if a given string is a static string.
///
/// # Arguments
///
/// * `s` - A string slice to check.
///
/// # Returns
///
/// Returns `true` if the given string is a static string, `false` otherwise.
pub fn is_staticized(s: &str) -> bool {
    STATIC_STRINGS.lock().unwrap().contains(s)
}

/// Removes a static string from the internal set of static strings.
///
/// # Arguments
///
/// * `s` - A string slice that represents the static string to be removed.
///
/// # Returns
///
/// A boolean value indicating whether the static string was present.
///
pub fn destaticize(s: &str) -> bool {
    STATIC_STRINGS.lock().unwrap().remove(s)
}

/// Concatenates the given string literals into a single static string slice.
///
/// # Examples
///
/// ```
/// use static_str_ops::static_concat;
///
/// let hello_world: &'static str = static_concat!("Hello", ", ", "world!");
/// assert_eq!(hello_world, "Hello, world!");
/// ```
///
/// # Panics
///
/// This macro will panic if any of the input expressions is not a string literal.
#[macro_export]
macro_rules! static_concat {
    ()=>{""};
    ($($arg: expr),* $(,)?)=>(
        $crate::staticize(concat!($($arg),*))
    );
}

/// A macro that takes a format string and arguments, and returns a static string slice.
///
/// # Examples
///
/// ```
/// use static_str_ops::static_format;
///
/// let name = "John";
/// let age = 30;
/// let message = static_format!("My name is {} and I'm {} years old.", name, age);
///
/// assert_eq!(message, "My name is John and I'm 30 years old.");
/// ```
///
/// This macro is similar to the `format!` macro, but it returns a static string slice
/// instead of a `String`, and can take whatever the `format!` macro recognizes.
#[macro_export]
macro_rules! static_format {
    ()=>{""};
    ($($arg: expr),* $(,)?)=>(
        $crate::staticize(format!($($arg),*))
    );
}

/// Internally used by `staticize_once!()`.
#[doc(hidden)]
#[macro_export]
macro_rules! _staticize_once {
    ($gensym: ident, $expr: expr) => {{
        use std::sync::Once;

        static mut $gensym: &str = "";
        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe {
            $gensym = staticize($expr);
        });
        unsafe { $gensym }
    }};
}

/// Macro to generate a unique identifier for a given expression, which can be used to create a static variable
/// that is initialized once with the result of the expression. This macro ensures that the expression is only
/// evaluated once at runtime, and the result is stored in a static variable for future use.
///
/// # Arguments
///
/// * `expr` - An expression of type `&str` or `String`.
///
/// # Examples
///
/// ```
/// #![feature(stmt_expr_attributes)]
///
/// use static_str_ops::*;
///
/// let s = "hello world";
/// let static_str = staticize_once!(s.to_uppercase());
/// assert_eq!(static_str, "HELLO WORLD");
/// ```
///
/// # Notes
///
/// This macro uses the `gensym` crate to generate a unique identifier for the expression. The `gensym` crate
/// is a compile-time code generation library that generates unique identifiers based on the source code
/// location where the macro is invoked. This ensures that the identifier is unique across the entire crate,
/// and is not accidentally used elsewhere.
#[macro_export]
macro_rules! staticize_once {
    ($expr: expr) => {
        gensym::gensym! { _staticize_once!{ $expr } }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic;

    #[test]
    fn test_staticize() {
        let s = staticize("hello");
        assert_eq!(s, "hello");
        let s = staticize("world");
        assert_eq!(s, "world");

        let s = staticize(String::from("hello world!"));
        assert_eq!(s, "hello world!");
    }

    #[test]
    fn test_is_staticized() {
        let s = "new hello world!";
        assert!(!is_staticized(s));
        let _ = staticize(s);
        assert!(is_staticized(s));
    }

    #[test]
    fn test_destaticize() {
        let s = "new hello world to be destaticized!";
        assert!(!is_staticized(s));
        let _ = staticize(s);
        assert!(is_staticized(s));
        assert!(destaticize(s));
        println!("{:?}", STATIC_STRINGS.lock().unwrap());
        assert!(!is_staticized(s));
    }

    #[test]
    fn test_static_concat() {
        let result: &'static str = static_concat!("hello", " ", "world", "!");
        assert_eq!(result, "hello world!");
    }

    #[test]
    fn test_static_format() {
        let result: &'static str = static_format!("{} {}!", "hello", "world");
        assert_eq!(result, "hello world!");
    }

    trait Typename {
        fn typename() -> &'static str {
            std::any::type_name::<Self>()
        }
    }

    impl Typename for i32 {}
    impl Typename for i64 {}

    struct Data<T> {
        phantom: std::marker::PhantomData<T>,
    }

    impl<T: Typename> Typename for Data<T> {
        fn typename() -> &'static str {
            static_format!("Data<{}>", T::typename())
        }
    }

    #[test]
    fn test_typename() {
        assert_eq!(Data::<i32>::typename(), "Data<i32>");
        assert_eq!(Data::<Data::<i64>>::typename(), "Data<Data<i64>>");
    }

    #[test]
    fn tets_staticize_once() {
        let called = atomic::AtomicI32::new(0);

        let make_typename = || {
            staticize_once!({
                called.fetch_add(1, atomic::Ordering::SeqCst);
                Data::<i32>::typename()
            })
        };

        let s1: &'static str = make_typename();
        let s2: &'static str = make_typename();
        let s3: &'static str = make_typename();
        assert!(s1 == s2 && s2 == s3);

        // ensure the body been called only once
        assert!(called.load(atomic::Ordering::SeqCst) == 1);
    }
}
