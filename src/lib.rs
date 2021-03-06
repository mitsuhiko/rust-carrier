//! This crate implements the `Completion` type and a `try!` macro which
//! utilizes it.  It's a proof of concept for the carrier trait that will
//! power the `?` operator.
//!
//! Essentially this code:
//!
//! ```rust
//! # #[macro_use] extern crate carrier;
//! # fn foo() -> Option<i32> { Some(42) }
//! # fn main() {
//! fn foo_as_string() -> Option<String> {
//!     Some(try!(foo()).to_string())
//! }
//! # }
//! ```
//! 
//! Converts into this:
//!
//! ```rust
//! # use carrier::{Completion, IntoCompletion};
//! # fn foo() -> Option<i32> { Some(42) }
//! fn foo_as_string() -> Option<String> {
//!     Some(match IntoCompletion::into_completion(foo()) {
//!         Completion::Value(x) => x,
//!         Completion::Abrupt(x) => { return x; }
//!     }.to_string())
//! }
//! ```
//!
//! The `IntoCompletion` trait is implemented for various types to allow
//! interoperability between different result types.
//!
//! When the `try!` macro is invoked it will attempt to convert the value
//! provided into a completion appropriate for the return value of the
//! function.  It will "unwrap" the success value and return it from the
//! expression or if it encounters exceptional circumstances the abrupt
//! value will be returned from the function.  In a way this can be seen
//! as a non-panicking version of `Option::unwrap()` and `Result::unwrap()`
//! just significantly more powerful.
//!
//! # What are Completions
//!
//! We refer to a `Completion` when we talk about the outcome of a computation
//! (like the return value of a function) which can either produce a value or
//! exit abruptly because of a exceptional condition (typically a failure).
//!
//! We refer to these two cases as a `Value` return (for the successful case)
//! or an `Abrupt` return for the exceptional one.  The `Value` case holds
//! just the result of the computation as such, the `Abrupt` case typically
//! holds an object such as a `Result` or `Option` which is set to the
//! case which would fail on `unwrap()`.  For instance `Result::Err` or
//! `Option::None`.
//!
//! Because completions are typically used with the `try!` macro which
//! issues an early return, the implementations for `IntoCompletion`
//! typically heavily depend on type inference.  As an example the
//! implementation for handling `Option<U>` will return an `Option<V>`
//! where `V` is entirely unconstraint.  This causes no issue since
//! the only return value for `Option<V>` that can be produced is `None`.
//!
//! # Completion Rules
//!
//! Most of the power of the `try!` macro is provided by the
//! `IntoCompletion` trait which defines the transition rules for how
//! to create a completion from `A` to `B`.
//!
//! ## Builtin Rules
//!
//! The following basic transitions are provided automatically:
//!
//! `Result<T, E>` -> `Result<T, F>`:
//! > This is the most common completion.  It is used to propagate an
//! > error wrapped in a result upwards.  Because this conversion is
//! > using `E: Into<F>` errors can automatically be converted if
//! > necessary.
//!
//! `Result<T, E>` -> `Option<Result<T, F>>`:
//! > This is a special form of the former conversion which simplifies
//! > the handling of results in iterators.  Iterators are a common
//! > feature in Rust and when you are dealing with iterators that
//! > might fail, typically the items of the iterator are results
//! > themselves.
//!
//! `Option<U>` -> `Option<V>`:
//! > This conversion permits the propagation of `None` from one option
//! > type to another.
//!
//! `*const T` -> `Option<U>` and `*mut T` -> `Option<U>`:
//! > For the raw pointer versions the null pointer is converted into
//! > `None` whereas all other values are unwrapped unchanged.
//!
//! ## Custom Rules
//!
//! If you have a similar object you want to convert automatically
//! with the `try!` macro you can implement the `IntoCompletion`
//! yourself.  For instance you might have a library which
//! implements an HTTP response object and you want to convert it
//! into a `Result` in case the status code is not 200:
//!
//! ```rust
//! # struct Response;
//! # impl Response {
//! #     fn is_successful(&self) -> bool { true }
//! #     fn status(&self) -> i32 { 200 }
//! # }
//! # enum ErrorKind { RequestFailed(i32) }
//! # use carrier::{Completion, IntoCompletion};
//!
//! impl<E, T> IntoCompletion<Result<T, E>> for Response
//!     where E: From<ErrorKind>
//! {
//!     type Value = Response;
//! 
//!     fn into_completion(self) -> Completion<Response, Result<T, E>> {
//!         if self.is_successful() {
//!             Completion::Value(self)
//!         } else {
//!             Completion::Abrupt(Err(ErrorKind::RequestFailed(
//!                 self.status()).into()))
//!         }
//!     }
//! }
//! ```
//!
//! Note how the completion rule returns a `Result<T, E>` where
//! `E: From<ErrorKind>` instead of returning a `Result<T, ErrorKind>`.
//! This allows the rule to be utilized in a function that returns a
//! different kind of error that however is compatible to the `ErrorKind`
//! of that library.

/// This macro performs error handling through the completion system.
///
/// In the future this will be implemented with the `?` operator instead.
#[macro_export]
macro_rules! try {
    ($expr:expr) => {
        match $crate::IntoCompletion::into_completion($expr) {
            $crate::Completion::Value(x) => x,
            $crate::Completion::Abrupt(x) => { return x; }
        }
    }
}

/// A `Completion` is an internal construct which can hold the result of a
/// computation that in the context of failure handling.
///
/// It can be compared to a special form of a result object.  It comes in
/// two variants: a `Value` which holds the resulting value of a
/// computation or an `Abrupt` which holds an object which indicates
/// an abrupt failure.
///
/// This object is also called a "completion carrier" or "result carrier".
pub enum Completion<V, F> {
    /// The completion resulted in a value
    Value(V),
    /// The completion encountered an abrupt failure
    Abrupt(F),
}

/// A conversion trait to convert an object into a `Completion`.
pub trait IntoCompletion<R> {
    /// The value of a completion
    type Value;

    /// Converts the given object into a completion record.
    fn into_completion(self) -> Completion<Self::Value, R>;
}

impl<T, U, E, F> IntoCompletion<Result<U, F>> for Result<T, E>
    where E: Into<F>
{
    type Value = T;

    fn into_completion(self) -> Completion<T, Result<U, F>> {
        match self {
            Ok(value) => Completion::Value(value),
            Err(err) => Completion::Abrupt(Err(err.into())),
        }
    }
}

impl<T, U, E, F> IntoCompletion<Option<Result<U, F>>> for Result<T, E>
    where E: Into<F>
{
    type Value = T;

    fn into_completion(self) -> Completion<T, Option<Result<U, F>>> {
        match self {
            Ok(value) => Completion::Value(value),
            Err(err) => Completion::Abrupt(Some(Err(err.into()))),
        }
    }
}

impl<U, V> IntoCompletion<Option<V>> for Option<U> {
    type Value = U;

    fn into_completion(self) -> Completion<U, Option<V>> {
        match self {
            Some(value) => Completion::Value(value),
            None => Completion::Abrupt(None),
        }
    }
}

impl<U, V> IntoCompletion<Option<V>> for *const U {
    type Value = *const U;

    fn into_completion(self) -> Completion<*const U, Option<V>> {
        if self.is_null() {
            Completion::Abrupt(None)
        } else {
            Completion::Value(self)
        }
    }
}

impl<U, V> IntoCompletion<Option<V>> for *mut U {
    type Value = *mut U;

    fn into_completion(self) -> Completion<*mut U, Option<V>> {
        if self.is_null() {
            Completion::Abrupt(None)
        } else {
            Completion::Value(self)
        }
    }
}
