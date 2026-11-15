/*
 * SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
 * SPDX-License-Identifier: MIT
 */

#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

#include <criterion/criterion.h>

#include "ariane/Ariane.Foundation.h"

typedef struct Buffer
{
        char*  pBytes;
        size_t length;
        size_t calls;
} tBuffer;

/**
 * @brief Collects the text of a report into a tBuffer.
 *
 * @param[in,out] aInOut_pContext The tBuffer.
 * @param[in]     aIn_pBytes      The text.
 * @param[in]     aLength         The length of the text in bytes.
 * @return true when the text was copied, false when memory ran out.
 */
static bool
Collect(void* aInOut_pContext, const char* aIn_pBytes, size_t aLength)
{
        tBuffer* pBuffer = aInOut_pContext;
        char*    pCopy   = malloc(aLength + 1U);

        if(pCopy == NULL)
                return (false);

        memcpy(pCopy, aIn_pBytes, aLength);
        pCopy[ aLength ] = '\x00';
        free(pBuffer->pBytes);
        pBuffer->pBytes = pCopy;
        pBuffer->length = aLength;
        pBuffer->calls++;

        return (true);
}

/**
 * @brief Refuses every text.
 *
 * @param[in,out] aInOut_pContext Unused.
 * @param[in]     aIn_pBytes      Unused.
 * @param[in]     aLength         Unused.
 * @return false.
 */
static bool
Refuse(void* aInOut_pContext, const char* aIn_pBytes, size_t aLength)
{
        (void) aInOut_pContext;
        (void) aIn_pBytes;
        (void) aLength;

        return (false);
}

/**
 * @brief Creates the sources of the tests: one file, `main.c`.
 *
 * @return The sources.
 */
static Ariane_Sources_tSources*
CreateSources(void)
{
        static const char sText[] = "int main(void)\n{\n        return (x);\n}\n";

        Ariane_Sources_tSources* pSources = NULL;
        size_t                   id       = 0U;

        cr_assert_eq(Ariane_Sources_Create(&pSources), Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Sources_Add(pSources, "main.c", 6U, sText, sizeof(sText) - 1U, &id),
                     Ariane_Core_kSuccess);

        return (pSources);
}

/**
 * @brief Writes a configuration without colors and with ASCII boxes, so the text can be compared.
 *
 * @param[out] aOut_pConfig The configuration to fill.
 */
static void
PlainConfig(Ariane_Report_tConfig* aOut_pConfig)
{
        cr_assert_eq(Ariane_Report_InitConfig(aOut_pConfig), Ariane_Core_kSuccess);
        aOut_pConfig->color   = false;
        aOut_pConfig->charSet = Ariane_Report_kAscii;
}

Test(Report, DefaultConfigIsReadFromC)
{
        Ariane_Report_tConfig config;

        cr_assert_eq(Ariane_Report_InitConfig(&config), Ariane_Core_kSuccess);
        cr_assert(config.crossGap);
        cr_assert_eq(config.labelAttach, Ariane_Report_kAttachMiddle);
        cr_assert_not(config.compact);
        cr_assert(config.underlines);
        cr_assert(config.multilineArrows);
        cr_assert(config.color);
        cr_assert_eq(config.tabWidth, 4U);
        cr_assert_eq(config.charSet, Ariane_Report_kUnicode);
        cr_assert_eq(config.indexType, Ariane_Report_kCharIndex);
}

Test(Report, InitLabelKeepsTheSpan)
{
        Ariane_Report_tLabel label;
        Ariane_Report_tSpan  span = { 0U, 3U, 7U };

        cr_assert_eq(Ariane_Report_InitLabel(&label, span), Ariane_Core_kSuccess);
        cr_assert_eq(label.span.sourceId, 0U);
        cr_assert_eq(label.span.start, 3U);
        cr_assert_eq(label.span.end, 7U);
        cr_assert_null(label.pMessage);
        cr_assert_eq(label.messageLength, 0U);
        cr_assert_eq(label.color.kind, Ariane_Core_kNoColor);
        cr_assert_eq(label.order, 0);
        cr_assert_eq(label.priority, 0);
}

Test(Report, WritesTheWholeReportInOneCall)
{
        static const char sLabel[]    = "not found in this scope";
        static const char sExpected[] = "[E0425] Error: cannot find value `x`\n"
                                        "   ,-[ main.c:3:17 ]\n"
                                        "   |\n"
                                        " 3 |         return (x);\n"
                                        "   |                 |  \n"
                                        "   |                 `-- not found in this scope\n"
                                        "   | \n"
                                        "   | Help: declare `x` first\n"
                                        "---'\n";

        Ariane_Sources_tSources* pSources = CreateSources();
        Ariane_Report_tReport*   pReport  = NULL;
        Ariane_Report_tSpan      span     = { 0U, 33U, 34U };
        Ariane_Report_tConfig    config;
        Ariane_Report_tLabel     label;
        tBuffer                  buffer = { 0 };

        PlainConfig(&config);
        cr_assert_eq(Ariane_Report_Create(Ariane_Report_kError, span, &pReport),
                     Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Report_SetConfig(pReport, &config), Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Report_SetCode(pReport, "E0425", 5U), Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Report_SetMessage(pReport, "cannot find value `x`", 21U),
                     Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Report_InitLabel(&label, span), Ariane_Core_kSuccess);
        label.pMessage      = sLabel;
        label.messageLength = sizeof(sLabel) - 1U;
        cr_assert_eq(Ariane_Report_AddLabel(pReport, &label), Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Report_AddHelp(pReport, "declare `x` first", 17U),
                     Ariane_Core_kSuccess);

        cr_assert_eq(Ariane_Report_Write(pReport, pSources, Collect, &buffer),
                     Ariane_Core_kSuccess);
        cr_assert_eq(buffer.calls, 1U);
        cr_assert_str_eq(buffer.pBytes, sExpected);

        free(buffer.pBytes);
        Ariane_Report_Destroy(pReport);
        Ariane_Sources_Destroy(pSources);
}

Test(Report, SpansAreChecked)
{
        Ariane_Sources_tSources* pSources  = CreateSources();
        Ariane_Report_tReport*   pReport   = NULL;
        Ariane_Report_tSpan      backwards = { 0U, 5U, 2U };
        Ariane_Report_tSpan      tooFar    = { 0U, 0U, 1000U };
        Ariane_Report_tSpan      otherFile = { 7U, 0U, 1U };
        tBuffer                  buffer    = { 0 };

        cr_assert_eq(Ariane_Report_Create(Ariane_Report_kError, backwards, &pReport),
                     Ariane_Core_kOutOfRange);
        cr_assert_null(pReport);

        cr_assert_eq(Ariane_Report_Create(Ariane_Report_kWarning, tooFar, &pReport),
                     Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Report_Write(pReport, pSources, Collect, &buffer),
                     Ariane_Core_kOutOfRange);
        Ariane_Report_Destroy(pReport);

        cr_assert_eq(Ariane_Report_Create(Ariane_Report_kAdvice, otherFile, &pReport),
                     Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Report_Write(pReport, pSources, Collect, &buffer),
                     Ariane_Core_kUnknownSource);
        cr_assert_eq(buffer.calls, 0U);

        Ariane_Report_Destroy(pReport);
        Ariane_Sources_Destroy(pSources);
}

Test(Report, ValuesOutsideTheirEnumerationsAreRejected)
{
        Ariane_Report_tReport* pReport = NULL;
        Ariane_Report_tSpan    span    = { 0U, 0U, 1U };
        Ariane_Report_tConfig  config;
        Ariane_Core_tColor     color = { 0 };

        cr_assert_eq(Ariane_Report_Create((Ariane_Report_tKind) 9, span, &pReport),
                     Ariane_Core_kInvalidArgument);
        cr_assert_eq(Ariane_Report_Create(Ariane_Report_kError, span, &pReport),
                     Ariane_Core_kSuccess);

        PlainConfig(&config);
        config.charSet = (Ariane_Report_tCharSet) 5;
        cr_assert_eq(Ariane_Report_SetConfig(pReport, &config), Ariane_Core_kInvalidArgument);
        PlainConfig(&config);
        config.tabWidth = 0U;
        cr_assert_eq(Ariane_Report_SetConfig(pReport, &config), Ariane_Core_kInvalidArgument);

        color.kind = (Ariane_Core_tColorKind) 3;
        cr_assert_eq(Ariane_Report_SetCustomKind(pReport, "Lint", 4U, color),
                     Ariane_Core_kInvalidArgument);
        cr_assert_eq(Ariane_Report_Print(pReport, NULL, Ariane_Report_kStderr),
                     Ariane_Core_kInvalidArgument);

        Ariane_Report_Destroy(pReport);
}

Test(Report, CustomKindNamesTheReport)
{
        Ariane_Sources_tSources* pSources = CreateSources();
        Ariane_Report_tReport*   pReport  = NULL;
        Ariane_Report_tSpan      span     = { 0U, 0U, 3U };
        Ariane_Report_tConfig    config;
        Ariane_Core_tColor       color  = { Ariane_Core_kFixedColor, 81U, 0U, 0U, 0U };
        tBuffer                  buffer = { 0 };

        PlainConfig(&config);
        cr_assert_eq(Ariane_Report_Create(Ariane_Report_kAdvice, span, &pReport),
                     Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Report_SetConfig(pReport, &config), Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Report_SetCustomKind(pReport, "Lint", 4U, color), Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Report_SetMessage(pReport, "prefer int32_t", 14U),
                     Ariane_Core_kSuccess);
        cr_assert_eq(Ariane_Report_Write(pReport, pSources, Collect, &buffer),
                     Ariane_Core_kSuccess);
        cr_assert(strncmp(buffer.pBytes, "Lint: prefer int32_t\n", 21U) == 0, "%s", buffer.pBytes);

        cr_assert_eq(Ariane_Report_Write(pReport, pSources, Refuse, NULL),
                     Ariane_Core_kWriteFailed);

        free(buffer.pBytes);
        Ariane_Report_Destroy(pReport);
        Ariane_Sources_Destroy(pSources);
}

Test(Report, VersionMatchesTheHeaders)
{
        cr_assert_eq(Ariane_Core_Version(), ARIANE_CORE_VERSION);
}
