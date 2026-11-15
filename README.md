<!-- SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com> -->
<!-- SPDX-License-Identifier: MIT -->

# Ariane

Ariane gives C programs the diagnostics of [Ariadne](https://github.com/zesterer/ariadne), the Rust library that draws
errors and warnings over the source code they are about (here with ASCII boxes and without colors):

```text
[E0425] Error: cannot find value `x`
   ,-[ main.c:3:17 ]
   |
 3 |         return (x);
   |                 |
   |                 `-- not found in this scope
   |
   | Help: declare `x` first
---'
```

The library is written in Rust and built as a shared and a static library with a C API. Its headers are written by
hand and follow [Progmasoft Core C Coding Guidelines](https://github.com/Progmasoft/progmasoft-cpp). The name is
the French form of Ariadne.

Wrappers for other languages build on this C API:

| Repository | For |
| --- | --- |
| [Progmasoft/arianex](https://github.com/Progmasoft/arianex), Arianex | C++17 |
| [Progmasoft/jariane](https://github.com/Progmasoft/jariane), JAriane | Java 21+ and the JVM 21+ languages with Java interop, through JNI |
| [Progmasoft/arianekt](https://github.com/Progmasoft/arianekt), ArianeKt | Kotlin/JVM 25+, through FFM |
| [Progmasoft/ariane.net](https://github.com/Progmasoft/ariane.net), Ariane.NET | the .NET 8+ languages |

Ariane is in development; no version is released yet.

## Using it from C

```c
#include <stdio.h>
#include <string.h>

#include "ariane/Ariane.Foundation.h"

/**
 * @brief Writes the text of a report to a stream.
 *
 * @param[in,out] aInOut_pContext The stream, a FILE*.
 * @param[in]     aIn_pBytes      The text.
 * @param[in]     aLength         The length of the text in bytes.
 * @return true when the whole text was written.
 */
static bool
Print(void* aInOut_pContext, const char* aIn_pBytes, size_t aLength)
{
        return (fwrite(aIn_pBytes, 1U, aLength, aInOut_pContext) == aLength);
}

int
main(void)
{
        static const char sText[] = "int main(void)\n{\n        return (x);\n}\n";

        Ariane_Sources_tSources* pSources = NULL;
        Ariane_Report_tReport*   pReport  = NULL;
        Ariane_Report_tSpan      span     = { 0U, 33U, 34U };
        Ariane_Report_tLabel     label;
        size_t                   id = 0U;

        Ariane_Sources_Create(&pSources);
        Ariane_Sources_Add(pSources, "main.c", 6U, sText, sizeof(sText) - 1U, &id);

        Ariane_Report_Create(Ariane_Report_kError, span, &pReport);
        Ariane_Report_SetCode(pReport, "E0425", 5U);
        Ariane_Report_SetMessage(pReport, "cannot find value `x`", 21U);
        Ariane_Report_InitLabel(&label, span);
        label.pMessage      = "not found in this scope";
        label.messageLength = strlen(label.pMessage);
        Ariane_Report_AddLabel(pReport, &label);
        Ariane_Report_AddHelp(pReport, "declare `x` first", 17U);

        Ariane_Report_Write(pReport, pSources, Print, stderr);

        Ariane_Report_Destroy(pReport);
        Ariane_Sources_Destroy(pSources);
        return (0);
}
```

Every function returns an `Ariane_Core_tStatus` and writes nothing when it fails. Texts are UTF-8 with a length and
are copied, so the caller keeps its memory. Spans count characters by default; `Ariane_Report_tConfig` switches to
bytes, ASCII boxes or no colors. The headers document each function.

| Header | Holds |
| --- | --- |
| `ariane/Ariane.Core.h` | the status, colors, the write function and the version |
| `ariane/Ariane.Sources.h` | the source texts that reports point into |
| `ariane/Ariane.Report.h` | reports, their labels, notes, helps and configuration |
| `ariane/Ariane.Foundation.h` | all of the above |

## Building

Ariadne is a dependency in `Cargo.toml`, pinned with its own dependencies in `Cargo.lock`. A C program needs a C
compiler and `rustc`, not Cargo: GNU Make, Meson and Bazel download the crates of `Cargo.lock` from crates.io and check
each against its SHA-256.

| Build | Command | Tests |
| --- | --- | --- |
| Bazel, the main build | `bazelisk build //:ariane //:ariane_c` | `bazelisk test //...` |
| Cargo | `cargo build --release` | `cargo test` |
| Meson 1.5+ | `meson setup build && meson compile -C build` | `meson test -C build` |
| GNU Make | `make` | `make test` |

- **Bazel** builds `//:ariane` (the shared library) and `//:ariane_c` (the headers with the static library, for
  `cc_*` targets); rules_rust downloads rustc 1.99.1. `//:crates_match_cargo_lock` fails when `MODULE.bazel` and
  `Cargo.lock` disagree.
- **Meson** reads `Cargo.lock` itself. Its support for crates is marked experimental by Meson.
- **GNU Make** installs with `make install PREFIX=/usr/local`: the headers, both libraries and `ariane.pc`, so a
  program can build with `pkg-config --cflags --libs ariane`.

The C tests use [Criterion](https://github.com/Snaipe/Criterion), from the system (`libcriterion-dev` on Debian and
Ubuntu, `criterion` on Arch and Homebrew).

## License

[MIT](LICENSE.txt). The libraries contain Ariadne (MIT), yansi and unicode-width (MIT or Apache-2.0); a binary
distribution carries their notices.

Participation follows the [Code of Conduct](CODE_OF_CONDUCT.md).
