//! Option/Maybe Types for Safe Null Pointer Handling
//! Implements Rust-like Option<T> type for safe null pointer handling in Logos

use std::fmt::Debug;

/// The Option type represents an optional value: every Option is either Some and contains a value, or None, and does not.
/// This is commonly used to represent values that might be absent, replacing null pointers with safe alternatives.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Option<T> {
    /// No value
    None,
    /// Some value `T`
    Some(T),
}

impl<T> Option<T> {
    /// Creates a new Option with Some value
    pub fn some(value: T) -> Self {
        Option::Some(value)
    }

    /// Creates a new Option with None value
    pub fn none() -> Self {
        Option::None
    }

    /// Returns true if the option is a Some value
    pub fn is_some(&self) -> bool {
        match self {
            Option::Some(_) => true,
            Option::None => false,
        }
    }

    /// Returns true if the option is a None value
    pub fn is_none(&self) -> bool {
        match self {
            Option::Some(_) => false,
            Option::None => true,
        }
    }

    /// Returns true if the option is a Some value containing the given value
    pub fn contains<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            Option::Some(y) => x == y,
            Option::None => false,
        }
    }

    /// Returns the contained Some value, consuming the self value.
    /// Panics if the value is None with a custom panic message provided by msg.
    pub fn expect(self, msg: &str) -> T {
        match self {
            Option::Some(val) => val,
            Option::None => panic!("{}", msg),
        }
    }

    /// Returns the contained Some value, consuming the self value.
    /// Panics if the value is None with a default message.
    pub fn unwrap(self) -> T {
        match self {
            Option::Some(val) => val,
            Option::None => panic!("called `Option::unwrap()` on a `None` value"),
        }
    }

    /// Returns the contained Some value or a default.
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Option::Some(val) => val,
            Option::None => default,
        }
    }

    /// Returns the contained Some value or computes it from a closure.
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Option::Some(val) => val,
            Option::None => f(),
        }
    }

    /// Maps an Option<T> to Option<U> by applying a function to a contained value.
    pub fn map<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Option::Some(x) => Option::Some(f(x)),
            Option::None => Option::None,
        }
    }

    /// Returns the provided default result (if None), or applies a function to the contained value (if any).
    pub fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Option::Some(t) => f(t),
            Option::None => default,
        }
    }

    /// Computes a default function result (if None), or applies a different function to the contained value (if any).
    pub fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(T) -> U,
    {
        match self {
            Option::Some(t) => f(t),
            Option::None => default(),
        }
    }

    /// Returns an iterator over the possibly contained value.
    pub fn iter(&self) -> Iter<T> {
        Iter { inner: self.as_ref() }
    }

    /// Returns an iterator over the possibly contained value.
    pub fn as_ref(&self) -> Option<&T> {
        match *self {
            Option::Some(ref x) => Option::Some(x),
            Option::None => Option::None,
        }
    }

    /// Returns an iterator over the possibly contained value.
    pub fn as_mut(&mut self) -> Option<&mut T> {
        match *self {
            Option::Some(ref mut x) => Option::Some(x),
            Option::None => Option::None,
        }
    }

    /// Transforms the Option<T> into a Result<T, E>, mapping Some(v) to Ok(v) and None to Err(err).
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Option::Some(v) => Result::Ok(v),
            Option::None => Result::Err(err),
        }
    }

    /// Transforms the Option<T> into a Result<T, E>, mapping Some(v) to Ok(v) and None to Err(err).
    pub fn ok_or_else<E, F>(self, err: F) -> Result<T, E>
    where
        F: FnOnce() -> E,
    {
        match self {
            Option::Some(v) => Result::Ok(v),
            Option::None => Result::Err(err()),
        }
    }

    /// Returns None if the option is None, otherwise returns opt.
    pub fn and<U>(self, opt: Option<U>) -> Option<U> {
        match self {
            Option::Some(_) => opt,
            Option::None => Option::None,
        }
    }

    /// Returns None if the option is None, otherwise calls f with the wrapped value and returns the result.
    pub fn and_then<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> Option<U>,
    {
        match self {
            Option::Some(x) => f(x),
            Option::None => Option::None,
        }
    }

    /// Returns the option if it contains a value, otherwise returns opt.
    pub fn or(self, opt: Option<T>) -> Option<T> {
        match self {
            Option::Some(_) => self,
            Option::None => opt,
        }
    }

    /// Returns the option if it contains a value, otherwise calls f and returns the result.
    pub fn or_else<F>(self, f: F) -> Option<T>
    where
        F: FnOnce() -> Option<T>,
    {
        match self {
            Option::Some(_) => self,
            Option::None => f(),
        }
    }

    /// Returns Some if exactly one of self, opt is Some, otherwise returns None.
    pub fn xor(self, opt: Option<T>) -> Option<T> {
        match (self, opt) {
            (Option::Some(a), Option::None) => Option::Some(a),
            (Option::None, Option::Some(b)) => Option::Some(b),
            _ => Option::None,
        }
    }

    /// Inserts a value computed from f into the option if it is None, then returns a mutable reference to the contained value.
    pub fn get_or_insert_with<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        match self {
            Option::Some(ref mut v) => v,
            Option::None => {
                *self = Option::Some(f());
                match self {
                    Option::Some(ref mut v) => v,
                    Option::None => unreachable!(),
                }
            }
        }
    }

    /// Takes the value out of the option, leaving a None in its place.
    pub fn take(&mut self) -> Option<T> {
        std::mem::replace(self, Option::None)
    }
}

impl<T: Clone> Option<&T> {
    /// Maps an Option<&T> to an Option<T> by cloning the contents.
    pub fn cloned(self) -> Option<T> {
        match self {
            Option::Some(x) => Option::Some(x.clone()),
            Option::None => Option::None,
        }
    }
}

impl<T: Default> Option<T> {
    /// Transforms the Option<T> into a T value. If the value is None, returns the default value for that type.
    pub fn unwrap_or_default(self) -> T {
        match self {
            Option::Some(x) => x,
            Option::None => T::default(),
        }
    }
}

impl<T: Debug> Debug for Option<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Option::Some(x) => f.debug_tuple("Some").field(x).finish(),
            Option::None => f.write_str("None"),
        }
    }
}

/// An iterator over a reference to the contents of an Option.
#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    inner: Option<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> std::option::Option<Self::Item> {
        // Take the value from our custom Option type and convert to std::option::Option
        match self.inner.take() {
            crate::memory_management::option::Option::Some(x) => std::option::Option::Some(x),
            crate::memory_management::option::Option::None => std::option::Option::None,
        }
    }

    fn size_hint(&self) -> (usize, std::option::Option<usize>) {
        match self.inner {
            crate::memory_management::option::Option::Some(_) => (1, std::option::Option::Some(1)),
            crate::memory_management::option::Option::None => (0, std::option::Option::None),
        }
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> std::option::Option<Self::Item> {
        self.next()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T> IntoIterator for &'a Option<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Result type for explicit error handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Result<T, E> {
    /// Contains the success value
    Ok(T),
    /// Contains the error value
    Err(E),
}

impl<T, E> Result<T, E> {
    /// Returns true if the result is Ok
    pub fn is_ok(&self) -> bool {
        match self {
            Result::Ok(_) => true,
            Result::Err(_) => false,
        }
    }

    /// Returns true if the result is Err
    pub fn is_err(&self) -> bool {
        match self {
            Result::Ok(_) => false,
            Result::Err(_) => true,
        }
    }

    /// Returns the contained Ok value, consuming the self value.
    /// Panics if the value is an Err with a default message.
    pub fn unwrap(self) -> T {
        match self {
            Result::Ok(val) => val,
            Result::Err(_) => panic!("called `Result::unwrap()` on an `Err` value"),
        }
    }

    /// Returns the contained Ok value, consuming the self value.
    /// Panics if the value is an Err with a custom message provided by msg.
    pub fn expect(self, msg: &str) -> T {
        match self {
            Result::Ok(val) => val,
            Result::Err(_) => panic!("{}", msg),
        }
    }

    /// Returns the contained Err value, consuming the self value.
    /// Panics if the value is an Ok with a default message.
    pub fn unwrap_err(self) -> E {
        match self {
            Result::Ok(_) => panic!("called `Result::unwrap_err()` on an `Ok` value"),
            Result::Err(val) => val,
        }
    }

    /// Returns the contained Ok value or a default.
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Result::Ok(val) => val,
            Result::Err(_) => default,
        }
    }

    /// Returns the contained Ok value or computes it from a closure.
    pub fn unwrap_or_else<F>(self, op: F) -> T
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Result::Ok(val) => val,
            Result::Err(err) => op(err),
        }
    }

    /// Maps a Result<T, E> to Result<U, E> by applying a function to a contained Ok value, leaving an Err value untouched.
    pub fn map<U, F>(self, op: F) -> Result<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Result::Ok(val) => Result::Ok(op(val)),
            Result::Err(err) => Result::Err(err),
        }
    }

    /// Maps a Result<T, E> to Result<T, F> by applying a function to a contained Err value, leaving an Ok value untouched.
    pub fn map_err<F, O>(self, op: O) -> Result<T, F>
    where
        O: FnOnce(E) -> F,
    {
        match self {
            Result::Ok(val) => Result::Ok(val),
            Result::Err(err) => Result::Err(op(err)),
        }
    }

    /// Returns the contained Ok value, but never panics.
    /// Unlike unwrap(), this method returns a copy of the value.
    /// If the value is an Err, it returns the "default" value for the type.
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Result::Ok(val) => val,
            Result::Err(_) => T::default(),
        }
    }

    /// Returns res if the result is Ok, otherwise returns the Err value of self.
    pub fn and<U>(self, res: Result<U, E>) -> Result<U, E> {
        match self {
            Result::Ok(_) => res,
            Result::Err(e) => Result::Err(e),
        }
    }

    /// Calls op if the result is Ok, otherwise returns the Err value of self.
    pub fn and_then<U, F>(self, op: F) -> Result<U, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        match self {
            Result::Ok(val) => op(val),
            Result::Err(err) => Result::Err(err),
        }
    }

    /// Returns res if the result is Err, otherwise returns the Ok value of self.
    pub fn or<F>(self, res: Result<T, F>) -> Result<T, F> {
        match self {
            Result::Ok(val) => Result::Ok(val),
            Result::Err(_) => res,
        }
    }

    /// Calls op if the result is Err, otherwise returns the Ok value of self.
    pub fn or_else<F, O>(self, op: O) -> Result<T, F>
    where
        O: FnOnce(E) -> Result<T, F>,
    {
        match self {
            Result::Ok(val) => Result::Ok(val),
            Result::Err(err) => op(err),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_some() {
        let x = Option::some(2);
        assert_eq!(x.is_some(), true);
        assert_eq!(x.is_none(), false);
        assert_eq!(x.unwrap(), 2);
    }

    #[test]
    fn test_option_none() {
        let x: Option<i32> = Option::none();
        assert_eq!(x.is_some(), false);
        assert_eq!(x.is_none(), true);
    }

    #[test]
    fn test_option_map() {
        let x = Option::some(2);
        let y = x.map(|v| v * 2);
        assert_eq!(y, Option::some(4));

        let x: Option<i32> = Option::none();
        let y = x.map(|v| v * 2);
        assert_eq!(y, Option::none());
    }

    #[test]
    fn test_result_ok() {
        let x: Result<i32, &str> = Result::Ok(2);
        assert_eq!(x.is_ok(), true);
        assert_eq!(x.is_err(), false);
        assert_eq!(x.unwrap(), 2);
    }

    #[test]
    fn test_result_err() {
        let x: Result<i32, &str> = Result::Err("Error occurred");
        assert_eq!(x.is_ok(), false);
        assert_eq!(x.is_err(), true);
        assert_eq!(x.unwrap_err(), "Error occurred");
    }

    #[test]
    fn test_result_map() {
        let x: Result<i32, &str> = Result::Ok(2);
        let y = x.map(|v| v * 2);
        assert_eq!(y, Result::Ok(4));

        let x: Result<i32, &str> = Result::Err("Error occurred");
        let y = x.map(|v| v * 2);
        assert_eq!(y, Result::Err("Error occurred"));
    }
}