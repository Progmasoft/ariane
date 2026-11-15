# SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
# SPDX-License-Identifier: MIT
#
# Builds Ariane with rustc and a C compiler, without Cargo. Meson takes the
# crates from crates.io in the versions and with the SHA-256 of Cargo.lock.

project(
  'ariane',
  'rust',
  'c',
  version: '0.1.0',
  license: 'MIT',
  meson_version: '>= 1.5.0',
  default_options: ['rust_std=2024', 'c_std=c11', 'warning_level=3', 'werror=true'],
)

# Meson reads the crates and their SHA-256 from Cargo.lock; warnings of their code are theirs.
ariadne = dependency('ariadne-0.6-rs', default_options: ['werror=false', 'warning_level=0'])

sources = files('src/lib.rs')

ariane_shared = shared_library(
  'ariane',
  sources,
  rust_abi: 'c',
  dependencies: ariadne,
  install: true,
)

ariane_static = static_library(
  'ariane',
  sources,
  rust_abi: 'c',
  dependencies: ariadne,
  install: true,
)

includes = include_directories('include')
install_headers(
  'include/ariane/Ariane.Core.h',
  'include/ariane/Ariane.Foundation.h',
  'include/ariane/Ariane.Report.h',
  'include/ariane/Ariane.Sources.h',
  subdir: 'ariane',
)

ariane_dep = declare_dependency(include_directories: includes, link_with: ariane_shared)
meson.override_dependency('ariane', ariane_dep)

pkgconfig = import('pkgconfig')
pkgconfig.generate(
  ariane_shared,
  name: 'ariane',
  description: 'The diagnostics of Ariadne through a C API',
)

criterion = dependency('criterion', required: get_option('tests'))
if criterion.found()
  c_tests = executable(
    'c_tests',
    'tests/c/ReportTests.c',
    'tests/c/SourcesTests.c',
    dependencies: [ariane_dep, criterion],
  )
  test('c_tests', c_tests)
endif
