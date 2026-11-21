//! The text form of a camera file, for people to read and edit: BNI (INI
//! as the game reads it, `bni-rs`), a `[w3c]` section first, then each
//! camera as a section named after it, its fields in file order.
//!
//! ```ini
//! [w3c]
//! version=0
//!
//! [Close Arthas]
//! target_x=-2728.98
//! target_y=1922.12
//! z_offset=0
//! ...
//! near_z=100
//! ```
//!
//! The local rotation (`local_pitch`, `local_yaw`, `local_roll`) is there
//! only in the layout that has it ([`Layout::Local`]): the text doesn't say
//! its layout any more than the file does, the map's `war3map.w3i` does.
//! Numbers are written as the shortest decimal that reads back as the same
//! `f32`, so the text gives back the exact bytes. A field left out takes
//! World Editor's default ([`Camera::new`]); cameras stay in their order.

use bni_rs::Document;
use bni_rs::ast::Item;
use parser_rs::{ParseError, Span};

use crate::{Camera, Cameras, Layout, VERSION};

/// The first section's name.
pub const HEADER: &str = "w3c";

/// The number of fields a camera has in `layout`.
fn field_count(layout: Layout) -> usize {
    if layout == Layout::Local { 13 } else { 10 }
}

/// The text form of `file`. Fails for a camera whose name a section header
/// can't hold: one with `]` or a line break.
pub fn to_text(file: &Cameras) -> Result<String, String> {
    let mut out = format!("[{HEADER}]\nversion={}\n", file.version);
    for c in &file.cameras {
        if c.name.contains([']', '\n', '\r']) {
            return Err(format!(
                "camera {:?}: a name with ']' or a line break can't be a section",
                c.name
            ));
        }
        out += &format!("\n[{}]\n", c.name);
        for (name, value) in &c.fields()[..field_count(file.layout)] {
            out += &format!("{name}={value}\n");
        }
    }
    Ok(out)
}

/// Is `doc` a camera file's text form: is its first section `[w3c]`?
pub fn is_camera_text(doc: &Document) -> bool {
    doc.sections.first().is_some_and(|s| s.name.text == HEADER)
}

/// What [`from_text`] read: the file when the text has no errors, and
/// every error, in order.
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    pub file: Option<Cameras>,
    pub errors: Vec<ParseError>,
}

/// Reads the text form back, in `layout`.
pub fn from_text(text: &str, layout: Layout) -> Parsed {
    from_bni(&bni_rs::parse(text), layout)
}

/// [`from_text`] of a document already parsed. Its BNI errors come first.
pub fn from_bni(doc: &Document, layout: Layout) -> Parsed {
    let mut errors = doc.errors.clone();
    for item in &doc.preamble {
        errors.push(error(
            item_span(item),
            format!("a field outside a section: the file starts with [{HEADER}]"),
        ));
    }
    let mut file = Cameras::new(layout);
    let Some((header, cameras)) = doc
        .sections
        .split_first()
        .filter(|(h, _)| h.name.text == HEADER)
    else {
        let span = doc
            .sections
            .first()
            .map_or(Span::new(0, 0, 0, 1), |s| s.name.span);
        errors.push(error(span, format!("the file starts with [{HEADER}]")));
        return Parsed { file: None, errors };
    };
    for item in &header.items {
        match item.key.as_ref().map(|k| k.text.as_str()) {
            Some("version") => match item.value_text().trim().parse::<u32>() {
                Ok(VERSION) => {}
                _ => errors.push(error(
                    item_span(item),
                    format!("the version is {VERSION}, the only one there is"),
                )),
            },
            _ => errors.push(error(
                item_span(item),
                format!("[{HEADER}] has only `version`"),
            )),
        }
    }
    let names: Vec<&str> = Camera::new("", 0.0, 0.0).fields().map(|(n, _)| n).to_vec();
    for section in cameras {
        let mut camera = Camera::new(section.name.text.clone(), 0.0, 0.0);
        let mut seen = Vec::new();
        for item in &section.items {
            let span = item_span(item);
            let Some(key) = item.key.as_ref().map(|k| k.text.trim()) else {
                errors.push(error(span, "a field without a name"));
                continue;
            };
            let Some(index) = names.iter().position(|n| *n == key) else {
                errors.push(error(
                    span,
                    format!(
                        "no field `{key}`: a camera has {}",
                        names[..field_count(layout)].join(", ")
                    ),
                ));
                continue;
            };
            if index >= field_count(layout) {
                errors.push(error(
                    span,
                    format!("`{key}` is only in maps for 1.31 and later (war3map.w3i version 28 and up); this one is for {}", layout.describe()),
                ));
                continue;
            }
            if seen.contains(&index) {
                errors.push(error(span, format!("`{key}` is set twice")));
                continue;
            }
            seen.push(index);
            match item.value_text().trim().parse::<f32>() {
                Ok(value) => *camera.field_mut(key).expect("a known field") = value,
                Err(_) => errors.push(error(span, format!("`{key}` is a number"))),
            }
        }
        file.cameras.push(camera);
    }
    let file = errors.is_empty().then_some(file);
    Parsed { file, errors }
}

fn error(span: Span, message: impl Into<String>) -> ParseError {
    let end_col = if span.end_line == span.start_line {
        span.end_col.max(span.start_col + 1)
    } else {
        span.start_col + 1
    };
    ParseError {
        message: message.into(),
        line: span.start_line,
        col: span.start_col,
        end_col,
    }
}

fn item_span(item: &Item) -> Span {
    let start = item.key.as_ref().map_or(item.eq, |k| k.span);
    let end = item.fields.last().map_or(item.eq, |f| f.span);
    Span::new(start.start_line, start.start_col, end.end_line, end.end_col)
}

#[cfg(test)]
#[path = "text.test.rs"]
mod tests;
