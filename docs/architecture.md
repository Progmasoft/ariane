<!-- SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com> -->
<!-- SPDX-License-Identifier: MIT -->

# Architecture

Ariane gives the diagnostics of the Rust crate Ariadne to C through a C API.

| Part | What it holds |
| --- | --- |
| `src/lib.rs`, `src/report.rs`, `src/sources.rs` | The Rust side: reports, their labels and notes, and the sources they point into |
| `src/ffi.rs` | The C functions, `extern "C"`, which check every pointer and length they are given |
| `include/ariane/` | The C headers, written by hand: `Ariane.Foundation.h`, `Ariane.Core.h`, `Ariane.Sources.h`, `Ariane.Report.h` |
| `tests/c/` | C tests of the API, with Criterion |
| `bazel/crates/` | Bazel's build files for the crates Ariane uses |

The library is built as a `cdylib`, a `staticlib` and an `rlib`. Bazel is the main build; Cargo, Meson and GNU Make
build the same sources for projects that use them.
