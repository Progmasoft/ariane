// SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
// SPDX-License-Identifier: MIT

//! `Ariane.Sources.h`: the texts that reports point into.

use std::ffi::c_char;
use std::fmt;

use crate::ffi::{Status, guard, text};

/// `Ariane_Sources_tSources`: the source texts of a set of reports, each with a name and an identifier.
pub struct Sources {
    entries: Vec<Entry>,
}

struct Entry {
    name: String,
    source: ariadne::Source<String>,
    byte_length: usize,
    char_length: usize,
}

impl Sources {
    /// The length of a source in bytes or in characters, or `None` when there is no such source.
    pub(crate) fn length(&self, id: usize, in_chars: bool) -> Option<usize> {
        self.entries.get(id).map(|entry| {
            if in_chars {
                entry.char_length
            } else {
                entry.byte_length
            }
        })
    }

    /// The sources as an Ariadne cache.
    pub(crate) fn cache(&self) -> Cache<'_> {
        Cache(self)
    }
}

/// The view of `Sources` that Ariadne reads.
pub(crate) struct Cache<'a>(&'a Sources);

impl ariadne::Cache<usize> for Cache<'_> {
    type Storage = String;

    fn fetch(&mut self, id: &usize) -> Result<&ariadne::Source<String>, impl fmt::Debug> {
        self.0
            .entries
            .get(*id)
            .map(|entry| &entry.source)
            .ok_or("unknown source")
    }

    fn display<'b>(&self, id: &'b usize) -> Option<impl fmt::Display + 'b> {
        self.0.entries.get(*id).map(|entry| entry.name.clone())
    }
}

/// Creates an empty set of sources, which the caller releases with `Ariane_Sources_Destroy`.
///
/// # Safety
///
/// `out_sources` is `NULL` or points to writable storage for one pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Sources_Create(out_sources: *mut *mut Sources) -> Status {
    guard(|| {
        if out_sources.is_null() {
            return Err(Status::InvalidArgument);
        }
        let sources = Box::new(Sources {
            entries: Vec::new(),
        });
        // SAFETY: `out_sources` is not NULL and points to storage for one pointer.
        unsafe { out_sources.write(Box::into_raw(sources)) };
        Ok(())
    })
}

/// Releases sources created by `Ariane_Sources_Create`. `NULL` is accepted and does nothing.
///
/// # Safety
///
/// `sources` is `NULL` or came from `Ariane_Sources_Create` and was not released before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Sources_Destroy(sources: *mut Sources) {
    if sources.is_null() {
        return;
    }
    let _ = guard(|| {
        // SAFETY: `sources` came from `Box::into_raw` in `Ariane_Sources_Create` and is released once.
        drop(unsafe { Box::from_raw(sources) });
        Ok(())
    });
}

/// Adds a source with its name and text, both UTF-8, and writes its identifier: 0 for the first, then 1, 2 and on.
///
/// The name and the text are copied; the caller's memory may be released when the function returns.
///
/// # Safety
///
/// `sources` came from `Ariane_Sources_Create`; `name` and `source_text` point to their lengths of readable bytes, or are
/// `NULL` with a length of zero; `out_id` points to writable storage for one `size_t`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Sources_Add(
    sources: *mut Sources,
    name: *const c_char,
    name_length: usize,
    source_text: *const c_char,
    source_text_length: usize,
    out_id: *mut usize,
) -> Status {
    guard(|| {
        if sources.is_null() || out_id.is_null() {
            return Err(Status::InvalidArgument);
        }
        // SAFETY: the caller guarantees the bytes of the name and the text.
        let (name, source_text) = unsafe {
            (
                text(name, name_length)?,
                text(source_text, source_text_length)?,
            )
        };
        // SAFETY: `sources` came from `Ariane_Sources_Create` and no other reference to it is alive.
        let sources = unsafe { &mut *sources };
        let id = sources.entries.len();
        sources.entries.push(Entry {
            name: name.to_owned(),
            source: ariadne::Source::from(source_text.to_owned()),
            byte_length: source_text.len(),
            char_length: source_text.chars().count(),
        });
        // SAFETY: `out_id` is not NULL and points to storage for one `size_t`.
        unsafe { out_id.write(id) };
        Ok(())
    })
}

/// The number of sources added so far; 0 for `NULL`.
///
/// # Safety
///
/// `sources` is `NULL` or came from `Ariane_Sources_Create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Sources_Count(sources: *const Sources) -> usize {
    // SAFETY: `sources` is NULL or a live set of sources.
    unsafe { sources.as_ref() }.map_or(0, |sources| sources.entries.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_count_up_from_zero() {
        let mut sources = std::ptr::null_mut();
        // SAFETY: the pointers are valid for the duration of each call.
        unsafe {
            assert_eq!(Ariane_Sources_Create(&mut sources), Status::Success);
            let mut id = usize::MAX;
            for expected in 0..3 {
                let status =
                    Ariane_Sources_Add(sources, c"a".as_ptr(), 1, c"x".as_ptr(), 1, &mut id);
                assert_eq!(status, Status::Success);
                assert_eq!(id, expected);
            }
            assert_eq!(Ariane_Sources_Count(sources), 3);
            Ariane_Sources_Destroy(sources);
        }
    }

    #[test]
    fn null_arguments_are_rejected() {
        let mut id = 0;
        // SAFETY: NULL is never dereferenced.
        unsafe {
            assert_eq!(
                Ariane_Sources_Create(std::ptr::null_mut()),
                Status::InvalidArgument
            );
            let status = Ariane_Sources_Add(
                std::ptr::null_mut(),
                std::ptr::null(),
                0,
                std::ptr::null(),
                0,
                &mut id,
            );
            assert_eq!(status, Status::InvalidArgument);
            assert_eq!(Ariane_Sources_Count(std::ptr::null()), 0);
            Ariane_Sources_Destroy(std::ptr::null_mut());
        }
    }
}
