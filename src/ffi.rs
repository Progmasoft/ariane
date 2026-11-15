// SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
// SPDX-License-Identifier: MIT

//! What every module of the C API shares: the status, colors, the write callback and the reading of C text.

use std::ffi::{c_char, c_int, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// The result of a function of the C API: `Ariane_Core_tStatus`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// `Ariane_Core_kSuccess`: the function did what it was asked.
    Success,
    /// `Ariane_Core_kInvalidArgument`: a pointer was `NULL`, or a value was outside its enumeration.
    InvalidArgument,
    /// `Ariane_Core_kInvalidUtf8`: a text was not UTF-8.
    InvalidUtf8,
    /// `Ariane_Core_kUnknownSource`: a span names a source that the sources do not hold.
    UnknownSource,
    /// `Ariane_Core_kOutOfRange`: a span ends before it starts, or after the end of its source.
    OutOfRange,
    /// `Ariane_Core_kWriteFailed`: the write function or the stream refused the output.
    WriteFailed,
    /// `Ariane_Core_kInternalError`: the library failed in a way it did not expect; it is a bug of the library.
    InternalError,
}

/// `Ariane_Core_tColor`: no color, one of the 256 terminal colors, or a 24-bit color.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Color {
    /// An `Ariane_Core_tColorKind`: 0 none, 1 fixed, 2 RGB.
    pub kind: c_int,
    /// The terminal color, from 0 to 255, for a fixed color.
    pub fixed: u8,
    /// The red part of an RGB color.
    pub red: u8,
    /// The green part of an RGB color.
    pub green: u8,
    /// The blue part of an RGB color.
    pub blue: u8,
}

impl Color {
    /// The color as Ariadne takes it, or `None` for no color.
    pub(crate) fn to_ariadne(self) -> Result<Option<ariadne::Color>, Status> {
        match self.kind {
            0 => Ok(None),
            1 => Ok(Some(ariadne::Color::Fixed(self.fixed))),
            2 => Ok(Some(ariadne::Color::Rgb(self.red, self.green, self.blue))),
            _ => Err(Status::InvalidArgument),
        }
    }
}

/// `Ariane_Core_tFn_Write`: receives the rendered bytes of a report; returns `false` when it could not take them.
pub type WriteFunction =
    Option<unsafe extern "C" fn(context: *mut c_void, bytes: *const c_char, length: usize) -> bool>;

/// Runs the body of a C API function, turning a panic into `InternalError`.
pub(crate) fn guard(body: impl FnOnce() -> Result<(), Status>) -> Status {
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(Ok(())) => Status::Success,
        Ok(Err(status)) => status,
        Err(_) => Status::InternalError,
    }
}

/// Reads `length` bytes of UTF-8 at `pointer`. `NULL` with a length of zero is the empty text.
///
/// # Safety
///
/// When `pointer` is not `NULL`, it points to `length` readable bytes that stay valid for `'a`.
pub(crate) unsafe fn text<'a>(pointer: *const c_char, length: usize) -> Result<&'a str, Status> {
    if pointer.is_null() {
        return if length == 0 {
            Ok("")
        } else {
            Err(Status::InvalidArgument)
        };
    }
    // SAFETY: the caller guarantees `length` readable bytes at `pointer`.
    let bytes = unsafe { std::slice::from_raw_parts(pointer.cast::<u8>(), length) };
    std::str::from_utf8(bytes).map_err(|_| Status::InvalidUtf8)
}

/// Reads a C `bool` passed through a `u8`, so that a value other than 0 and 1 is not undefined behavior.
pub(crate) fn boolean(value: u8) -> bool {
    value != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_accepts_null_only_when_empty() {
        // SAFETY: NULL is never read.
        assert_eq!(unsafe { text(std::ptr::null(), 0) }, Ok(""));
        // SAFETY: NULL is never read.
        let result = unsafe { text(std::ptr::null(), 3) };
        assert_eq!(result, Err(Status::InvalidArgument));
    }

    #[test]
    fn text_rejects_invalid_utf8() {
        let bytes = [0xFFu8, 0xFE];
        // SAFETY: the array has two bytes.
        let result = unsafe { text(bytes.as_ptr().cast(), bytes.len()) };
        assert_eq!(result, Err(Status::InvalidUtf8));
    }

    #[test]
    fn guard_turns_a_panic_into_internal_error() {
        let status = guard(|| panic!("a bug"));
        assert_eq!(status, Status::InternalError);
    }

    #[test]
    fn colors_outside_the_enumeration_are_rejected() {
        let color = Color {
            kind: 7,
            fixed: 0,
            red: 0,
            green: 0,
            blue: 0,
        };
        assert_eq!(color.to_ariadne().unwrap_err(), Status::InvalidArgument);
    }
}
