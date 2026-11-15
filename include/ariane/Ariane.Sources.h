/*
 * SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stddef.h>

#include "ariane/Ariane.Core.h"

#ifdef __cplusplus
extern "C" {
#endif

/** @brief The source texts that reports point into, each with a name and an identifier. */
typedef struct Ariane_Sources_Sources Ariane_Sources_tSources;

/**
 * @brief Creates an empty set of sources.
 *
 * @param[out] aOut_ppSources Receives the sources, which the caller releases with
 * Ariane_Sources_Destroy().
 * @return Ariane_Core_kSuccess, or Ariane_Core_kInvalidArgument when the pointer is `NULL`.
 *
 * @see Ariane_Sources_Destroy()
 */
Ariane_Core_tStatus
Ariane_Sources_Create(Ariane_Sources_tSources** aOut_ppSources);

/**
 * @brief Releases sources and every text they hold.
 *
 * @param[in,out] aInOut_pSources The sources to release, or `NULL`.
 *
 * @see Ariane_Sources_Create()
 */
void
Ariane_Sources_Destroy(Ariane_Sources_tSources* aInOut_pSources);

/**
 * @brief Adds a source with its name and text.
 *
 * The name and the text are copied: the caller may release them when the function returns.
 * The first source gets the identifier 0, the next 1, and so on.
 *
 * @param[in,out] aInOut_pSources The sources.
 * @param[in]     aIn_pName       The name shown in reports, UTF-8, such as a file path.
 * @param[in]     aNameLength     The length of the name in bytes.
 * @param[in]     aIn_pText       The text of the source, UTF-8.
 * @param[in]     aTextLength     The length of the text in bytes.
 * @param[out]    aOut_pId        Receives the identifier of the source.
 * @return Ariane_Core_kSuccess, Ariane_Core_kInvalidArgument or Ariane_Core_kInvalidUtf8.
 *
 * @note A pointer may be `NULL` when its length is 0.
 */
Ariane_Core_tStatus
Ariane_Sources_Add(Ariane_Sources_tSources* aInOut_pSources,
                   const char*              aIn_pName,
                   size_t                   aNameLength,
                   const char*              aIn_pText,
                   size_t                   aTextLength,
                   size_t*                  aOut_pId);

/**
 * @brief Returns the number of sources.
 *
 * @param[in] aIn_pSources The sources, or `NULL`.
 * @return The number of sources added, or 0 for `NULL`.
 */
size_t
Ariane_Sources_Count(const Ariane_Sources_tSources* aIn_pSources);

#ifdef __cplusplus
}
#endif
