#!/bin/sh
# SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
# SPDX-License-Identifier: MIT
#
# Measures what the Rust tests and the Criterion tests of the C API reach in
# the library, and writes it as LCOV to build/coverage.lcov. Needs
# cargo-llvm-cov, llvm-tools and Criterion.
#
#   tools/coverage.sh

set -eu

cd "$(dirname "$0")/.."
eval "$(cargo llvm-cov show-env --sh)"
cargo llvm-cov clean --workspace

# The Rust tests, then the C tests against the instrumented shared library.
cargo test --locked
cargo build --locked
mkdir -p build
cc -std=c11 -pedantic-errors -Wall -Wextra -Werror -Iinclude tests/c/*.c \
        -L"$CARGO_LLVM_COV_TARGET_DIR/debug" -lariane -lcriterion \
        -Wl,-rpath,"$CARGO_LLVM_COV_TARGET_DIR/debug" -o build/c_tests_coverage
build/c_tests_coverage

cargo llvm-cov report --lcov --output-path build/coverage.lcov
cargo llvm-cov report --summary-only
