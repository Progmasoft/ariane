<!-- SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com> -->
<!-- SPDX-License-Identifier: MIT -->

# Development

```sh
bazelisk test //...            # the main build and its tests
cargo test --locked            # the Rust tests
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
make -j"$(nproc)" test         # the C tests with GNU Make
doxygen Doxyfile               # the documentation of the C API, in doxygen/
tools/coverage.sh
```

CI runs the Bazel, Cargo, Meson and Make builds, Clippy and rustfmt, the C tests, coverage, and Doxygen;
`tools/check-crates.sh` checks that Bazel's crates match `Cargo.lock`.
