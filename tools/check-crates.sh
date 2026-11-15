#!/bin/sh
# SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
# SPDX-License-Identifier: MIT
#
# Checks that MODULE.bazel downloads every crate of Cargo.lock in its version
# and with its SHA-256, so that Bazel builds what Cargo builds.
#
#   tools/check-crates.sh Cargo.lock MODULE.bazel

set -eu

lock=$1
module=$2
status=0

crates=$(awk '
        /^\[\[package\]\]/ { name = ""; version = "" }
        /^name = /     { gsub(/name = |"/, ""); name = $0 }
        /^version = /  { gsub(/version = |"/, ""); version = $0 }
        /^checksum = / { gsub(/checksum = |"/, ""); print name, version, $0 }
' "$lock")

if [ -z "$crates" ]; then
        echo "no crates with a checksum in $lock" >&2
        exit 1
fi

echo "$crates" | while read -r name version checksum; do
        url="https://static.crates.io/crates/$name/$name-$version.crate"
        if ! grep -q "\"$url\"" "$module" || ! grep -q "\"$checksum\"" "$module"; then
                echo "$name $version: MODULE.bazel does not download $url with sha256 $checksum" >&2
                exit 1
        fi
        echo "$name $version: matches"
done || status=1

count=$(grep -c '^http_archive($' "$module")
expected=$(echo "$crates" | wc -l | tr -d ' ')
if [ "$count" -ne "$expected" ]; then
        echo "MODULE.bazel has $count crates, Cargo.lock $expected" >&2
        status=1
fi

exit $status
