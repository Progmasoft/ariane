# SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
# SPDX-License-Identifier: MIT
#
# Builds Ariane with rustc and a C compiler, without Cargo. The crates come from
# crates.io in the versions of Cargo.lock and are checked against its SHA-256.
#
#   make            the shared and the static library
#   make test       the Criterion tests of the C API
#   make install    headers, libraries and ariane.pc under PREFIX

RUSTC    ?= rustc
CC       ?= cc
CURL     ?= curl
PREFIX   ?= /usr/local
OPT      ?= 2
BUILD    := build/make
CRATES   := $(BUILD)/crates
LIB      := $(BUILD)/lib

UNAME    := $(shell uname -s)
ifeq ($(UNAME),Darwin)
SHARED   := libariane.dylib
SHA256   := shasum -a 256 -c
else
SHARED   := libariane.so
SHA256   := sha256sum -c
endif

# The version and checksum of a crate, read from Cargo.lock.
lock_version  = $(shell awk '/^name = "$(1)"$$/ { getline; gsub(/version = |"/, ""); print; exit }' Cargo.lock)
lock_checksum = $(shell awk '/^name = "$(1)"$$/ { found = 1 } found && /^checksum/ { gsub(/checksum = |"/, ""); print; exit }' Cargo.lock)

ARIADNE_VERSION       := $(call lock_version,ariadne)
YANSI_VERSION         := $(call lock_version,yansi)
UNICODE_WIDTH_VERSION := $(call lock_version,unicode-width)

RUSTFLAGS := -C opt-level=$(OPT) --cap-lints allow -L dependency=$(LIB)
HEADERS   := $(wildcard include/ariane/*.h)
SOURCES   := $(wildcard src/*.rs)
TESTS     := $(wildcard tests/c/*.c)

.PHONY: all test install clean
all: $(LIB)/$(SHARED) $(LIB)/libariane.a

# Downloads a crate and checks it: $(1) name, $(2) version.
define fetch
	mkdir -p $(CRATES)
	$(CURL) -fsSL -o $(CRATES)/$(1)-$(2).crate https://static.crates.io/crates/$(1)/$(1)-$(2).crate
	echo "$(call lock_checksum,$(1))  $(CRATES)/$(1)-$(2).crate" | $(SHA256)
	tar -xzf $(CRATES)/$(1)-$(2).crate -C $(CRATES)
endef

$(CRATES)/unicode-width-$(UNICODE_WIDTH_VERSION)/src/lib.rs:
	$(call fetch,unicode-width,$(UNICODE_WIDTH_VERSION))

$(CRATES)/yansi-$(YANSI_VERSION)/src/lib.rs:
	$(call fetch,yansi,$(YANSI_VERSION))

$(CRATES)/ariadne-$(ARIADNE_VERSION)/src/lib.rs:
	$(call fetch,ariadne,$(ARIADNE_VERSION))

$(LIB)/libunicode_width.rlib: $(CRATES)/unicode-width-$(UNICODE_WIDTH_VERSION)/src/lib.rs
	mkdir -p $(LIB)
	$(RUSTC) $(RUSTFLAGS) --edition=2021 --crate-type=rlib --crate-name=unicode_width \
		--cfg 'feature="cjk"' --out-dir $(LIB) $<

$(LIB)/libyansi.rlib: $(CRATES)/yansi-$(YANSI_VERSION)/src/lib.rs
	mkdir -p $(LIB)
	$(RUSTC) $(RUSTFLAGS) --edition=2021 --crate-type=rlib --crate-name=yansi \
		--cfg 'feature="std"' --cfg 'feature="alloc"' --out-dir $(LIB) $<

$(LIB)/libariadne.rlib: $(CRATES)/ariadne-$(ARIADNE_VERSION)/src/lib.rs $(LIB)/libunicode_width.rlib $(LIB)/libyansi.rlib
	$(RUSTC) $(RUSTFLAGS) --edition=2018 --crate-type=rlib --crate-name=ariadne \
		--extern unicode_width=$(LIB)/libunicode_width.rlib --extern yansi=$(LIB)/libyansi.rlib \
		--out-dir $(LIB) $<

$(LIB)/$(SHARED) $(LIB)/libariane.a &: $(SOURCES) $(LIB)/libariadne.rlib
	$(RUSTC) -C opt-level=$(OPT) -L dependency=$(LIB) --edition=2024 --crate-name=ariane \
		--crate-type=cdylib --crate-type=staticlib --extern ariadne=$(LIB)/libariadne.rlib \
		--out-dir $(LIB) src/lib.rs

$(BUILD)/c_tests: $(TESTS) $(HEADERS) $(LIB)/$(SHARED)
	$(CC) -std=c11 -pedantic-errors -Wall -Wextra -Werror -Iinclude $(TESTS) -L$(LIB) -lariane \
		-lcriterion -Wl,-rpath,$(abspath $(LIB)) -o $@

test: $(BUILD)/c_tests
	$(BUILD)/c_tests

$(BUILD)/ariane.pc: Cargo.toml
	mkdir -p $(BUILD)
	printf 'prefix=%s\nincludedir=$${prefix}/include\nlibdir=$${prefix}/lib\n\nName: ariane\nDescription: The diagnostics of Ariadne through a C API\nVersion: %s\nCflags: -I$${includedir}\nLibs: -L$${libdir} -lariane\n' \
		'$(PREFIX)' "$$(awk -F'"' '/^version/ { print $$2; exit }' Cargo.toml)" > $@

install: all $(BUILD)/ariane.pc
	install -d $(DESTDIR)$(PREFIX)/include/ariane $(DESTDIR)$(PREFIX)/lib/pkgconfig
	install -m 0644 $(HEADERS) $(DESTDIR)$(PREFIX)/include/ariane
	install -m 0755 $(LIB)/$(SHARED) $(DESTDIR)$(PREFIX)/lib
	install -m 0644 $(LIB)/libariane.a $(DESTDIR)$(PREFIX)/lib
	install -m 0644 $(BUILD)/ariane.pc $(DESTDIR)$(PREFIX)/lib/pkgconfig

clean:
	rm -rf $(BUILD)
