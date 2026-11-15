/*
 * SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
 * SPDX-License-Identifier: MIT
 */

#include <stddef.h>

#include <criterion/criterion.h>

#include "ariane/Ariane.Sources.h"

Test(Sources, IdentifiersCountUpFromZero)
{
        Ariane_Sources_tSources* pSources = NULL;
        size_t                   id       = 99U;

        cr_assert_eq(Ariane_Sources_Create(&pSources), Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Sources_Add(pSources, "a.c", 3U, "int a;", 6U, &id),
                     Ariane_Core_kSuccess);
        cr_assert_eq(id, 0U);
        cr_assert_eq(Ariane_Sources_Add(pSources, "b.c", 3U, "int b;", 6U, &id),
                     Ariane_Core_kSuccess);
        cr_assert_eq(id, 1U);
        cr_assert_eq(Ariane_Sources_Count(pSources), 2U);

        Ariane_Sources_Destroy(pSources);
}

Test(Sources, NullIsRejected)
{
        size_t id = 99U;

        cr_assert_eq(Ariane_Sources_Create(NULL), Ariane_Core_kInvalidArgument);
        cr_assert_eq(Ariane_Sources_Add(NULL, NULL, 0U, NULL, 0U, &id),
                     Ariane_Core_kInvalidArgument);
        cr_assert_eq(id, 99U);
        cr_assert_eq(Ariane_Sources_Count(NULL), 0U);
        Ariane_Sources_Destroy(NULL);
}

Test(Sources, InvalidUtf8IsRejected)
{
        Ariane_Sources_tSources* pSources = NULL;
        size_t                   id       = 99U;

        cr_assert_eq(Ariane_Sources_Create(&pSources), Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Sources_Add(pSources, "a.c", 3U, "\xFF\xFE", 2U, &id),
                     Ariane_Core_kInvalidUtf8);
        cr_assert_eq(id, 99U);
        cr_assert_eq(Ariane_Sources_Count(pSources), 0U);

        Ariane_Sources_Destroy(pSources);
}

Test(Sources, EmptyTextMayBeNull)
{
        Ariane_Sources_tSources* pSources = NULL;
        size_t                   id       = 99U;

        cr_assert_eq(Ariane_Sources_Create(&pSources), Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Sources_Add(pSources, NULL, 0U, NULL, 0U, &id), Ariane_Core_kSuccess);
        cr_assert_eq(id, 0U);
        cr_assert_eq(Ariane_Sources_Add(pSources, NULL, 1U, NULL, 0U, &id),
                     Ariane_Core_kInvalidArgument);

        Ariane_Sources_Destroy(pSources);
}
