// SPDX-FileCopyrightText: 2026 Progmasoft <support@progmasoft.com>
// SPDX-License-Identifier: MIT

//! `Ariane.Report.h`: a diagnostic with its labels, notes and helps, rendered by Ariadne.

use std::ffi::{c_char, c_int, c_void};
use std::io::Write;

use crate::ffi::{Color, Status, WriteFunction, boolean, guard, text};
use crate::sources::Sources;

const KIND_ERROR: c_int = 0;
const KIND_WARNING: c_int = 1;
const KIND_ADVICE: c_int = 2;

const STREAM_STDOUT: c_int = 0;
const STREAM_STDERR: c_int = 1;

/// `Ariane_Report_tSpan`: a range of a source, from `start` up to but not including `end`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Span {
    /// The identifier `Ariane_Sources_Add` gave the source.
    pub source_id: usize,
    /// The first position of the range.
    pub start: usize,
    /// The position after the last one of the range.
    pub end: usize,
}

/// `Ariane_Report_tLabel`: a span with what it says about the code there.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Label {
    /// The code the label points at.
    pub span: Span,
    /// The message, UTF-8, or `NULL` for a label without one.
    pub message: *const c_char,
    /// The length of the message in bytes.
    pub message_length: usize,
    /// The color of the label.
    pub color: Color,
    /// Labels with a lower order are drawn first.
    pub order: i32,
    /// Where labels overlap, the one with the higher priority is shown.
    pub priority: i32,
}

/// `Ariane_Report_tConfig`: how a report is drawn.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Whether crossing label lines leave a gap instead of a crossing character.
    pub cross_gap: u8,
    /// An `Ariane_Report_tLabelAttach`: where an inline label attaches to its span.
    pub label_attach: c_int,
    /// Whether the report leaves out gaps to use less space.
    pub compact: u8,
    /// Whether spans are underlined where possible.
    pub underlines: u8,
    /// Whether arrows point to the bounds of spans over several lines.
    pub multiline_arrows: u8,
    /// Whether the output has terminal colors.
    pub color: u8,
    /// How many columns a tab takes.
    pub tab_width: usize,
    /// An `Ariane_Report_tCharSet`: Unicode or ASCII for boxes and arrows.
    pub char_set: c_int,
    /// An `Ariane_Report_tIndexType`: whether spans count bytes or characters.
    pub index_type: c_int,
}

impl Config {
    fn defaults() -> Config {
        Config {
            cross_gap: 1,
            label_attach: 1,
            compact: 0,
            underlines: 1,
            multiline_arrows: 1,
            color: 1,
            tab_width: 4,
            char_set: 0,
            index_type: 1,
        }
    }

    fn to_ariadne(self) -> Result<ariadne::Config, Status> {
        let attach = match self.label_attach {
            0 => ariadne::LabelAttach::Start,
            1 => ariadne::LabelAttach::Middle,
            2 => ariadne::LabelAttach::End,
            _ => return Err(Status::InvalidArgument),
        };
        let char_set = match self.char_set {
            0 => ariadne::CharSet::Unicode,
            1 => ariadne::CharSet::Ascii,
            _ => return Err(Status::InvalidArgument),
        };
        let index_type = match self.index_type {
            0 => ariadne::IndexType::Byte,
            1 => ariadne::IndexType::Char,
            _ => return Err(Status::InvalidArgument),
        };
        if self.tab_width == 0 {
            return Err(Status::InvalidArgument);
        }
        Ok(ariadne::Config::new()
            .with_cross_gap(boolean(self.cross_gap))
            .with_label_attach(attach)
            .with_compact(boolean(self.compact))
            .with_underlines(boolean(self.underlines))
            .with_multiline_arrows(boolean(self.multiline_arrows))
            .with_color(boolean(self.color))
            .with_tab_width(self.tab_width)
            .with_char_set(char_set)
            .with_index_type(index_type))
    }
}

enum Kind {
    Error,
    Warning,
    Advice,
    Custom(String, ariadne::Color),
}

struct OwnedLabel {
    span: Span,
    message: Option<String>,
    color: Option<ariadne::Color>,
    order: i32,
    priority: i32,
}

/// `Ariane_Report_tReport`: a diagnostic being put together, drawn by `Ariane_Report_Write` or `Ariane_Report_Print`.
pub struct Report {
    kind: Kind,
    span: Span,
    code: Option<String>,
    message: Option<String>,
    notes: Vec<String>,
    helps: Vec<String>,
    labels: Vec<OwnedLabel>,
    config: Config,
}

impl Report {
    fn check_span(span: Span, sources: &Sources, in_chars: bool) -> Result<(), Status> {
        let length = sources
            .length(span.source_id, in_chars)
            .ok_or(Status::UnknownSource)?;
        if span.end > length {
            return Err(Status::OutOfRange);
        }
        Ok(())
    }

    fn render(&self, sources: &Sources, for_stdout: bool) -> Result<Vec<u8>, Status> {
        let config = self.config.to_ariadne()?;
        let in_chars = self.config.index_type == 1;
        Self::check_span(self.span, sources, in_chars)?;
        for label in &self.labels {
            Self::check_span(label.span, sources, in_chars)?;
        }
        let kind = match &self.kind {
            Kind::Error => ariadne::ReportKind::Error,
            Kind::Warning => ariadne::ReportKind::Warning,
            Kind::Advice => ariadne::ReportKind::Advice,
            Kind::Custom(name, color) => ariadne::ReportKind::Custom(name.as_str(), *color),
        };
        let span = (self.span.source_id, self.span.start..self.span.end);
        let mut builder = ariadne::Report::build(kind, span).with_config(config);
        if let Some(code) = &self.code {
            builder = builder.with_code(code);
        }
        if let Some(message) = &self.message {
            builder.set_message(message);
        }
        for note in &self.notes {
            builder.add_note(note);
        }
        for help in &self.helps {
            builder.add_help(help);
        }
        for label in &self.labels {
            let mut drawn =
                ariadne::Label::new((label.span.source_id, label.span.start..label.span.end))
                    .with_order(label.order)
                    .with_priority(label.priority);
            if let Some(message) = &label.message {
                drawn = drawn.with_message(message);
            }
            if let Some(color) = label.color {
                drawn = drawn.with_color(color);
            }
            builder.add_label(drawn);
        }
        let report = builder.finish();
        let mut output = Vec::new();
        let written = if for_stdout {
            report.write_for_stdout(sources.cache(), &mut output)
        } else {
            report.write(sources.cache(), &mut output)
        };
        written.map_err(|_| Status::InternalError)?;
        if let Kind::Custom(name, _) = &self.kind {
            if !boolean(self.config.color) {
                remove_kind_color(&mut output, self.code.as_deref(), name);
            }
        }
        Ok(output)
    }
}

/// Ariadne 0.6.0 draws the color of a custom kind even when colors are off: the report then starts with
/// `ESC [ <color> m`, the code and the name, and `ESC [ 0 m`. This takes exactly those two sequences out.
fn remove_kind_color(output: &mut Vec<u8>, code: Option<&str>, name: &str) {
    let header = match code {
        Some(code) => format!("[{code}] {name}:"),
        None => format!("{name}:"),
    };
    let Some(end_of_start) = output.iter().position(|&byte| byte == b'm') else {
        return;
    };
    let start = &output[..=end_of_start];
    let is_color = start.starts_with(b"\x1b[")
        && start[2..end_of_start]
            .iter()
            .all(|&byte| byte.is_ascii_digit() || byte == b';');
    let rest = &output[end_of_start + 1..];
    let reset = b"\x1b[0m";
    if is_color && rest.starts_with(header.as_bytes()) && rest[header.len()..].starts_with(reset) {
        let reset_start = end_of_start + 1 + header.len();
        output.drain(reset_start..reset_start + reset.len());
        output.drain(..=end_of_start);
    }
}

fn check_range(span: Span) -> Result<Span, Status> {
    if span.start > span.end {
        return Err(Status::OutOfRange);
    }
    Ok(span)
}

/// Runs `change` on the report behind `report`.
///
/// # Safety
///
/// `report` is `NULL` or came from `Ariane_Report_Create`, and no other reference to it is alive.
unsafe fn with_report(
    report: *mut Report,
    change: impl FnOnce(&mut Report) -> Result<(), Status>,
) -> Status {
    guard(|| {
        // SAFETY: the caller guarantees that `report` is NULL or a live report used by no one else.
        let report = unsafe { report.as_mut() }.ok_or(Status::InvalidArgument)?;
        change(report)
    })
}

/// Writes the default configuration: Unicode boxes, colors, character spans, tabs of four columns.
///
/// # Safety
///
/// `out_config` is `NULL` or points to writable storage for one `Ariane_Report_tConfig`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Report_InitConfig(out_config: *mut Config) -> Status {
    guard(|| {
        if out_config.is_null() {
            return Err(Status::InvalidArgument);
        }
        // SAFETY: `out_config` is not NULL and points to storage for one configuration.
        unsafe { out_config.write(Config::defaults()) };
        Ok(())
    })
}

/// Writes a label for `span` without a message or a color, with order and priority 0.
///
/// # Safety
///
/// `out_label` is `NULL` or points to writable storage for one `Ariane_Report_tLabel`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Report_InitLabel(out_label: *mut Label, span: Span) -> Status {
    guard(|| {
        if out_label.is_null() {
            return Err(Status::InvalidArgument);
        }
        let label = Label {
            span,
            message: std::ptr::null(),
            message_length: 0,
            color: Color {
                kind: 0,
                fixed: 0,
                red: 0,
                green: 0,
                blue: 0,
            },
            order: 0,
            priority: 0,
        };
        // SAFETY: `out_label` is not NULL and points to storage for one label.
        unsafe { out_label.write(label) };
        Ok(())
    })
}

/// Creates a report of `kind` (error, warning or advice) at `span`, released with `Ariane_Report_Destroy`.
///
/// # Safety
///
/// `out_report` is `NULL` or points to writable storage for one pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Report_Create(
    kind: c_int,
    span: Span,
    out_report: *mut *mut Report,
) -> Status {
    guard(|| {
        if out_report.is_null() {
            return Err(Status::InvalidArgument);
        }
        let kind = match kind {
            KIND_ERROR => Kind::Error,
            KIND_WARNING => Kind::Warning,
            KIND_ADVICE => Kind::Advice,
            _ => return Err(Status::InvalidArgument),
        };
        let report = Box::new(Report {
            kind,
            span: check_range(span)?,
            code: None,
            message: None,
            notes: Vec::new(),
            helps: Vec::new(),
            labels: Vec::new(),
            config: Config::defaults(),
        });
        // SAFETY: `out_report` is not NULL and points to storage for one pointer.
        unsafe { out_report.write(Box::into_raw(report)) };
        Ok(())
    })
}

/// Releases a report created by `Ariane_Report_Create`. `NULL` is accepted and does nothing.
///
/// # Safety
///
/// `report` is `NULL` or came from `Ariane_Report_Create` and was not released before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Report_Destroy(report: *mut Report) {
    if report.is_null() {
        return;
    }
    let _ = guard(|| {
        // SAFETY: `report` came from `Box::into_raw` in `Ariane_Report_Create` and is released once.
        drop(unsafe { Box::from_raw(report) });
        Ok(())
    });
}

/// Gives the report a kind of its own, with its name and color, in place of error, warning or advice.
///
/// # Safety
///
/// `report` came from `Ariane_Report_Create`; `name` points to `name_length` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Report_SetCustomKind(
    report: *mut Report,
    name: *const c_char,
    name_length: usize,
    color: Color,
) -> Status {
    // SAFETY: the caller guarantees the report and the bytes of the name.
    unsafe {
        with_report(report, |report| {
            let name = text(name, name_length)?;
            if name.is_empty() {
                return Err(Status::InvalidArgument);
            }
            let color = color.to_ariadne()?.unwrap_or(ariadne::Color::Primary);
            report.kind = Kind::Custom(name.to_owned(), color);
            Ok(())
        })
    }
}

/// Defines one of the setters and adders of text: the text is checked, copied and handed to `apply`.
macro_rules! text_function {
    ($(#[$doc:meta])* $name:ident, $apply:expr) => {
        $(#[$doc])*
        ///
        /// # Safety
        ///
        /// `report` came from `Ariane_Report_Create`; `value` points to `length` readable bytes of UTF-8.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(report: *mut Report, value: *const c_char, length: usize) -> Status {
            // SAFETY: the caller guarantees the report and the bytes of the text.
            unsafe {
                with_report(report, |report| {
                    let value = text(value, length)?.to_owned();
                    let apply: fn(&mut Report, String) = $apply;
                    apply(report, value);
                    Ok(())
                })
            }
        }
    };
}

text_function!(
    /// Sets the code of the report, such as `E0308`, shown in brackets before its message.
    Ariane_Report_SetCode,
    |report, value| report.code = Some(value)
);
text_function!(
    /// Sets the message of the report.
    Ariane_Report_SetMessage,
    |report, value| report.message = Some(value)
);
text_function!(
    /// Adds a note below the code of the report.
    Ariane_Report_AddNote,
    |report, value| report.notes.push(value)
);
text_function!(
    /// Adds a help below the code of the report.
    Ariane_Report_AddHelp,
    |report, value| report.helps.push(value)
);

/// Adds a label; its message is copied.
///
/// # Safety
///
/// `report` came from `Ariane_Report_Create`; `label` points to a label whose message points to its length of bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Report_AddLabel(
    report: *mut Report,
    label: *const Label,
) -> Status {
    // SAFETY: the caller guarantees the report, the label and its message.
    unsafe {
        with_report(report, |report| {
            let label = label.as_ref().ok_or(Status::InvalidArgument)?;
            let message = if label.message.is_null() && label.message_length == 0 {
                None
            } else {
                Some(text(label.message, label.message_length)?.to_owned())
            };
            report.labels.push(OwnedLabel {
                span: check_range(label.span)?,
                message,
                color: label.color.to_ariadne()?,
                order: label.order,
                priority: label.priority,
            });
            Ok(())
        })
    }
}

/// Sets how the report is drawn; the configuration is copied.
///
/// # Safety
///
/// `report` came from `Ariane_Report_Create`; `config` points to a configuration.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Report_SetConfig(
    report: *mut Report,
    config: *const Config,
) -> Status {
    // SAFETY: the caller guarantees the report and the configuration.
    unsafe {
        with_report(report, |report| {
            let config = *config.as_ref().ok_or(Status::InvalidArgument)?;
            config.to_ariadne()?;
            report.config = config;
            Ok(())
        })
    }
}

/// Draws the report over `sources` and hands the whole text to `write` in one call.
///
/// # Safety
///
/// `report` came from `Ariane_Report_Create` and `sources` from `Ariane_Sources_Create`; `write` may be called with
/// `context`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Report_Write(
    report: *const Report,
    sources: *const Sources,
    write: WriteFunction,
    context: *mut c_void,
) -> Status {
    guard(|| {
        // SAFETY: the caller guarantees that both are NULL or live.
        let (report, sources) = unsafe { (report.as_ref(), sources.as_ref()) };
        let (Some(report), Some(sources), Some(write)) = (report, sources, write) else {
            return Err(Status::InvalidArgument);
        };
        let output = report.render(sources, false)?;
        // SAFETY: the caller allows `write` to be called with `context`; the bytes live until it returns.
        let accepted = unsafe { write(context, output.as_ptr().cast(), output.len()) };
        if accepted {
            Ok(())
        } else {
            Err(Status::WriteFailed)
        }
    })
}

/// Draws the report over `sources` to standard output or standard error.
///
/// # Safety
///
/// `report` came from `Ariane_Report_Create` and `sources` from `Ariane_Sources_Create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Ariane_Report_Print(
    report: *const Report,
    sources: *const Sources,
    stream: c_int,
) -> Status {
    guard(|| {
        // SAFETY: the caller guarantees that both are NULL or live.
        let (report, sources) = unsafe { (report.as_ref(), sources.as_ref()) };
        let (Some(report), Some(sources)) = (report, sources) else {
            return Err(Status::InvalidArgument);
        };
        let result = match stream {
            STREAM_STDOUT => {
                let output = report.render(sources, true)?;
                let mut stdout = std::io::stdout().lock();
                stdout.write_all(&output).and_then(|()| stdout.flush())
            }
            STREAM_STDERR => {
                let output = report.render(sources, false)?;
                let mut stderr = std::io::stderr().lock();
                stderr.write_all(&output).and_then(|()| stderr.flush())
            }
            _ => return Err(Status::InvalidArgument),
        };
        result.map_err(|_| Status::WriteFailed)
    })
}
