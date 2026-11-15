/*
 * SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define ARIANE_CORE_VERSION_MAJOR 0U
#define ARIANE_CORE_VERSION_MINOR 1U
#define ARIANE_CORE_VERSION_PATCH 0U

/** The version of these headers, as `major << 16 | minor << 8 | patch`. */
#define ARIANE_CORE_VERSION                                                   \
        ((ARIANE_CORE_VERSION_MAJOR << 16) | (ARIANE_CORE_VERSION_MINOR << 8) \
         | ARIANE_CORE_VERSION_PATCH)

/**
 * @brief The result of a function of Ariane.
 *
 * A function that fails changes nothing and writes none of its out parameters.
 */
typedef enum Ariane_Core_Status
{
        /** The function did what it was asked. */
        Ariane_Core_kSuccess,
        /** A pointer was `NULL`, or a value was outside its enumeration. */
        Ariane_Core_kInvalidArgument,
        /** A text was not UTF-8. */
        Ariane_Core_kInvalidUtf8,
        /** A span names a source that the sources do not hold. */
        Ariane_Core_kUnknownSource,
        /** A span ends before it starts, or after the end of its source. */
        Ariane_Core_kOutOfRange,
        /** The write function or the stream refused the output. */
        Ariane_Core_kWriteFailed,
        /** The library failed in a way it did not expect; it is a bug of Ariane. */
        Ariane_Core_kInternalError,
} Ariane_Core_tStatus;

/** @brief What an `Ariane_Core_tColor` holds. */
typedef enum Ariane_Core_ColorKind
{
        /** No color: the default of the terminal. */
        Ariane_Core_kNoColor,
        /** One of the 256 terminal colors, in `fixed`. */
        Ariane_Core_kFixedColor,
        /** A 24-bit color, in `red`, `green` and `blue`. */
        Ariane_Core_kRgbColor,
} Ariane_Core_tColorKind;

/** @brief A color of a label or of a custom kind of report. */
typedef struct Ariane_Core_Color
{
        Ariane_Core_tColorKind kind;
        uint8_t                fixed;
        uint8_t                red;
        uint8_t                green;
        uint8_t                blue;
} Ariane_Core_tColor;

/**
 * @brief Receives the text of a report.
 *
 * @param[in,out] aInOut_pContext The context the caller passed with the function.
 * @param[in]     aIn_pBytes      The text, UTF-8, not terminated by a null character.
 * @param[in]     aLength         The length of the text in bytes.
 * @return true when the text was taken, false otherwise.
 */
typedef bool (*Ariane_Core_tFn_Write)(void*       aInOut_pContext,
                                      const char* aIn_pBytes,
                                      size_t      aLength);

/**
 * @brief Returns the version of the library.
 *
 * @return The version as `major << 16 | minor << 8 | patch`, to compare with `ARIANE_CORE_VERSION`.
 */
uint32_t
Ariane_Core_Version(void);

#ifdef __cplusplus
}
#endif
