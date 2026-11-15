/*
 * SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "ariane/Ariane.Core.h"
#include "ariane/Ariane.Sources.h"

#ifdef __cplusplus
extern "C" {
#endif

/** @brief The kind of a report, shown before its message. */
typedef enum Ariane_Report_Kind
{
        Ariane_Report_kError,
        Ariane_Report_kWarning,
        Ariane_Report_kAdvice,
} Ariane_Report_tKind;

/** @brief Where an inline label attaches to its span. */
typedef enum Ariane_Report_LabelAttach
{
        Ariane_Report_kAttachStart,
        Ariane_Report_kAttachMiddle,
        Ariane_Report_kAttachEnd,
} Ariane_Report_tLabelAttach;

/** @brief The characters that draw boxes and arrows. */
typedef enum Ariane_Report_CharSet
{
        Ariane_Report_kUnicode,
        Ariane_Report_kAscii,
} Ariane_Report_tCharSet;

/** @brief What the positions of a span count. */
typedef enum Ariane_Report_IndexType
{
        /** Bytes of UTF-8. */
        Ariane_Report_kByteIndex,
        /** Characters, that is Unicode scalar values. */
        Ariane_Report_kCharIndex,
} Ariane_Report_tIndexType;

/** @brief The stream Ariane_Report_Print() writes to. */
typedef enum Ariane_Report_Stream
{
        Ariane_Report_kStdout,
        Ariane_Report_kStderr,
} Ariane_Report_tStream;

/** @brief A range of a source, from `start` up to but not including `end`. */
typedef struct Ariane_Report_Span
{
        size_t sourceId;
        size_t start;
        size_t end;
} Ariane_Report_tSpan;

/** @brief A span with what it says about the code there. */
typedef struct Ariane_Report_Label
{
        Ariane_Report_tSpan span;
        const char*         pMessage;
        size_t              messageLength;
        Ariane_Core_tColor  color;
        int32_t             order;
        int32_t             priority;
} Ariane_Report_tLabel;

/** @brief How a report is drawn. */
typedef struct Ariane_Report_Config
{
        bool                       crossGap;
        Ariane_Report_tLabelAttach labelAttach;
        bool                       compact;
        bool                       underlines;
        bool                       multilineArrows;
        bool                       color;
        size_t                     tabWidth;
        Ariane_Report_tCharSet     charSet;
        Ariane_Report_tIndexType   indexType;
} Ariane_Report_tConfig;

/** @brief A diagnostic: a kind, a message, labels, notes and helps. */
typedef struct Ariane_Report_Report Ariane_Report_tReport;

/**
 * @brief Writes the default configuration.
 *
 * The default draws Unicode boxes in color, counts characters, gives a tab four columns, attaches
 * labels to the middle of their spans and leaves gaps where label lines cross.
 *
 * @param[out] aOut_pConfig The configuration to fill.
 * @return Ariane_Core_kSuccess, or Ariane_Core_kInvalidArgument when the pointer is `NULL`.
 */
Ariane_Core_tStatus
Ariane_Report_InitConfig(Ariane_Report_tConfig* aOut_pConfig);

/**
 * @brief Writes a label for a span without a message or a color, with order and priority 0.
 *
 * @param[out] aOut_pLabel The label to fill.
 * @param[in]  aSpan       The span of the label.
 * @return Ariane_Core_kSuccess, or Ariane_Core_kInvalidArgument when the pointer is `NULL`.
 */
Ariane_Core_tStatus
Ariane_Report_InitLabel(Ariane_Report_tLabel* aOut_pLabel, Ariane_Report_tSpan aSpan);

/**
 * @brief Creates a report.
 *
 * @param[in]  aKind         The kind of the report.
 * @param[in]  aSpan         The place the report is about.
 * @param[out] aOut_ppReport Receives the report, which the caller releases with
 * Ariane_Report_Destroy().
 * @return Ariane_Core_kSuccess, Ariane_Core_kInvalidArgument, or Ariane_Core_kOutOfRange when the
 * span ends before it starts.
 *
 * @see Ariane_Report_Destroy()
 */
Ariane_Core_tStatus
Ariane_Report_Create(Ariane_Report_tKind     aKind,
                     Ariane_Report_tSpan     aSpan,
                     Ariane_Report_tReport** aOut_ppReport);

/**
 * @brief Releases a report.
 *
 * @param[in,out] aInOut_pReport The report to release, or `NULL`.
 *
 * @see Ariane_Report_Create()
 */
void
Ariane_Report_Destroy(Ariane_Report_tReport* aInOut_pReport);

/**
 * @brief Gives the report a kind of its own in place of error, warning or advice.
 *
 * @param[in,out] aInOut_pReport The report.
 * @param[in]     aIn_pName      The name of the kind, UTF-8, not empty.
 * @param[in]     aNameLength    The length of the name in bytes.
 * @param[in]     aColor         The color of the name; no color is the default of the terminal.
 * @return Ariane_Core_kSuccess, Ariane_Core_kInvalidArgument or Ariane_Core_kInvalidUtf8.
 */
Ariane_Core_tStatus
Ariane_Report_SetCustomKind(Ariane_Report_tReport* aInOut_pReport,
                            const char*            aIn_pName,
                            size_t                 aNameLength,
                            Ariane_Core_tColor     aColor);

/**
 * @brief Sets the code of the report, such as `E0308`, shown in brackets before its message.
 *
 * @param[in,out] aInOut_pReport The report.
 * @param[in]     aIn_pCode      The code, UTF-8.
 * @param[in]     aCodeLength    The length of the code in bytes.
 * @return Ariane_Core_kSuccess, Ariane_Core_kInvalidArgument or Ariane_Core_kInvalidUtf8.
 */
Ariane_Core_tStatus
Ariane_Report_SetCode(Ariane_Report_tReport* aInOut_pReport,
                      const char*            aIn_pCode,
                      size_t                 aCodeLength);

/**
 * @brief Sets the message of the report.
 *
 * @param[in,out] aInOut_pReport  The report.
 * @param[in]     aIn_pMessage    The message, UTF-8.
 * @param[in]     aMessageLength  The length of the message in bytes.
 * @return Ariane_Core_kSuccess, Ariane_Core_kInvalidArgument or Ariane_Core_kInvalidUtf8.
 */
Ariane_Core_tStatus
Ariane_Report_SetMessage(Ariane_Report_tReport* aInOut_pReport,
                         const char*            aIn_pMessage,
                         size_t                 aMessageLength);

/**
 * @brief Adds a note, shown below the code of the report.
 *
 * @param[in,out] aInOut_pReport The report.
 * @param[in]     aIn_pNote      The note, UTF-8.
 * @param[in]     aNoteLength    The length of the note in bytes.
 * @return Ariane_Core_kSuccess, Ariane_Core_kInvalidArgument or Ariane_Core_kInvalidUtf8.
 */
Ariane_Core_tStatus
Ariane_Report_AddNote(Ariane_Report_tReport* aInOut_pReport,
                      const char*            aIn_pNote,
                      size_t                 aNoteLength);

/**
 * @brief Adds a help, shown below the code of the report.
 *
 * @param[in,out] aInOut_pReport The report.
 * @param[in]     aIn_pHelp      The help, UTF-8.
 * @param[in]     aHelpLength    The length of the help in bytes.
 * @return Ariane_Core_kSuccess, Ariane_Core_kInvalidArgument or Ariane_Core_kInvalidUtf8.
 */
Ariane_Core_tStatus
Ariane_Report_AddHelp(Ariane_Report_tReport* aInOut_pReport,
                      const char*            aIn_pHelp,
                      size_t                 aHelpLength);

/**
 * @brief Adds a label; its message is copied.
 *
 * @param[in,out] aInOut_pReport The report.
 * @param[in]     aIn_pLabel     The label; its message may be `NULL` with a length of 0.
 * @return Ariane_Core_kSuccess, Ariane_Core_kInvalidArgument, Ariane_Core_kInvalidUtf8, or
 * Ariane_Core_kOutOfRange when the span ends
 * before it starts.
 */
Ariane_Core_tStatus
Ariane_Report_AddLabel(Ariane_Report_tReport*      aInOut_pReport,
                       const Ariane_Report_tLabel* aIn_pLabel);

/**
 * @brief Sets how the report is drawn; the configuration is copied.
 *
 * @param[in,out] aInOut_pReport The report.
 * @param[in]     aIn_pConfig    The configuration.
 * @return Ariane_Core_kSuccess, or Ariane_Core_kInvalidArgument for a `NULL` pointer, a value
 * outside its enumeration or a tab
 * width of 0.
 */
Ariane_Core_tStatus
Ariane_Report_SetConfig(Ariane_Report_tReport*       aInOut_pReport,
                        const Ariane_Report_tConfig* aIn_pConfig);

/**
 * @brief Draws the report and hands the whole text to a write function in one call.
 *
 * @param[in]     aIn_pReport     The report.
 * @param[in]     aIn_pSources    The sources the spans of the report point into.
 * @param[in]     aWrite          The function that receives the text.
 * @param[in,out] aInOut_pContext Passed to the write function as it is.
 * @return Ariane_Core_kSuccess, Ariane_Core_kInvalidArgument, Ariane_Core_kUnknownSource,
 * Ariane_Core_kOutOfRange when a span ends after
 * its source, or Ariane_Core_kWriteFailed when the write function returns false.
 */
Ariane_Core_tStatus
Ariane_Report_Write(const Ariane_Report_tReport*   aIn_pReport,
                    const Ariane_Sources_tSources* aIn_pSources,
                    Ariane_Core_tFn_Write          aWrite,
                    void*                          aInOut_pContext);

/**
 * @brief Draws the report to standard output or standard error.
 *
 * @param[in] aIn_pReport  The report.
 * @param[in] aIn_pSources The sources the spans of the report point into.
 * @param[in] aStream      The stream to write to.
 * @return Ariane_Core_kSuccess, Ariane_Core_kInvalidArgument, Ariane_Core_kUnknownSource,
 * Ariane_Core_kOutOfRange when a span ends after
 * its source, or Ariane_Core_kWriteFailed when the stream refuses the text.
 */
Ariane_Core_tStatus
Ariane_Report_Print(const Ariane_Report_tReport*   aIn_pReport,
                    const Ariane_Sources_tSources* aIn_pSources,
                    Ariane_Report_tStream          aStream);

#ifdef __cplusplus
}
#endif
