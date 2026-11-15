// SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
// SPDX-License-Identifier: MIT

//! Ariane: the diagnostics of [Ariadne](https://crates.io/crates/ariadne) through a C API.
//!
//! The C API is declared in `include/ariane/`. Every function of it is defined here with the name it has in C. No
//! panic leaves the library: each function catches it and returns `Ariane_Core_kInternalError`. Values that come from C
//! as enumerations or `bool` are read as integers and checked, so an invalid value is reported, never undefined.

#![allow(non_snake_case)]

mod ffi;
pub mod report;
pub mod sources;

pub use ffi::{Color, Status, WriteFunction};

/// The major version of the library.
pub const VERSION_MAJOR: u32 = 0;
/// The minor version of the library.
pub const VERSION_MINOR: u32 = 1;
/// The patch version of the library.
pub const VERSION_PATCH: u32 = 0;

/// The version of the library as `major << 16 | minor << 8 | patch`.
///
/// A caller compares it with `ARIANE_CORE_VERSION` from the headers it was compiled against.
#[unsafe(no_mangle)]
pub extern "C" fn Ariane_Core_Version() -> u32 {
    (VERSION_MAJOR << 16) | (VERSION_MINOR << 8) | VERSION_PATCH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_the_package() {
        let expected = env!("CARGO_PKG_VERSION");
        let actual = format!(
            "{}.{}.{}",
            Ariane_Core_Version() >> 16,
            (Ariane_Core_Version() >> 8) & 0xFF,
            Ariane_Core_Version() & 0xFF
        );
        assert_eq!(actual, expected);
    }
}
