//! Parse the `.cyb` three-rule frontmatter: TOML up to the first `~~~` marker,
//! and index the `~~~name` sections that follow. Shared by every `.cyb`
//! container reader (`.graph`, `.vocab`, `.model`).

use std::collections::HashMap;

use crate::error::{McError, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Frontmatter {
    pub cyb: CybMeta,
    #[serde(default, rename = "files")]
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CybMeta {
    pub types: Vec<String>,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub format: String,
    #[serde(default)]
    pub size: Option<u64>,
}

/// Split a `.cyb` byte stream at the first `\n~~~` line.
///
/// Returns the frontmatter as utf-8 and the byte offset of the first `~~~`
/// (so the caller starts section parsing exactly there).
pub fn split(bytes: &[u8]) -> Result<(&str, usize)> {
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if &bytes[i..i + 4] == b"\n~~~" {
            let fm = std::str::from_utf8(&bytes[..i + 1])
                .map_err(|e| McError::InvalidGraph(format!("non-utf8 frontmatter: {e}")))?;
            return Ok((fm, i + 1));
        }
        i += 1;
    }
    Err(McError::InvalidGraph("no `~~~` delimiter found".into()))
}

pub fn parse(s: &str) -> Result<Frontmatter> {
    Ok(toml::from_str(s)?)
}

/// Locate every declared `~~~name` section, returning `name → (start, end)`
/// byte ranges over `bytes`. A section with a `size` is that many bytes of
/// binary; a section without one runs to the next `~~~` marker or EOF.
pub fn index_sections(
    bytes: &[u8],
    body_start: usize,
    entries: &[FileEntry],
) -> Result<HashMap<String, (usize, usize)>> {
    let mut sections = HashMap::new();
    let mut cursor = body_start;

    for entry in entries {
        let header = format!("~~~{}\n", entry.name);
        if cursor + header.len() > bytes.len() || !bytes[cursor..].starts_with(header.as_bytes()) {
            return Err(McError::InvalidGraph(format!(
                "expected `{}` at byte {}",
                header.trim_end(),
                cursor
            )));
        }
        let start = cursor + header.len();
        let end = match entry.size {
            Some(sz) => start.checked_add(sz as usize).ok_or_else(|| {
                McError::InvalidGraph(format!(
                    "section `{}` size overflows (start={start}, size={sz})",
                    entry.name
                ))
            })?,
            None => find_text_end(bytes, start),
        };
        if end > bytes.len() {
            return Err(McError::InvalidGraph(format!(
                "section `{}` extends past EOF (end={}, file={})",
                entry.name,
                end,
                bytes.len()
            )));
        }
        sections.insert(entry.name.clone(), (start, end));
        cursor = end;
        if cursor < bytes.len() && bytes[cursor] == b'\n' {
            cursor += 1;
        }
    }
    Ok(sections)
}

/// Scan forward from `start` for `\n~~~` (next section) or EOF, returning the
/// first byte offset not belonging to this text section.
fn find_text_end(bytes: &[u8], start: usize) -> usize {
    let mut j = start;
    while j + 4 <= bytes.len() {
        if &bytes[j..j + 4] == b"\n~~~" {
            return j;
        }
        j += 1;
    }
    bytes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, size: Option<u64>) -> FileEntry {
        FileEntry {
            name: name.into(),
            format: "records".into(),
            size,
        }
    }

    #[test]
    fn split_finds_the_first_delimiter_and_returns_the_body_offset() {
        let bytes = b"[cyb]\nname=\"x\"\n~~~a\ndata".to_vec();
        let (fm, body_start) = split(&bytes).unwrap();
        assert_eq!(fm, "[cyb]\nname=\"x\"\n");
        assert_eq!(&bytes[body_start..], b"~~~a\ndata");
    }

    #[test]
    fn split_rejects_a_stream_with_no_delimiter() {
        assert!(split(b"no delimiter here at all").is_err());
    }

    #[test]
    fn split_rejects_non_utf8_frontmatter() {
        let mut bytes = vec![0xFF, 0xFE];
        bytes.extend_from_slice(b"\n~~~a\n");
        assert!(split(&bytes).is_err());
    }

    #[test]
    fn index_sections_locates_a_sized_and_an_unsized_section_in_order() {
        let bytes = b"~~~a\nXYZ~~~b\ntail to eof".to_vec();
        let entries = vec![entry("a", Some(3)), entry("b", None)];
        let body_start = 0;
        let sections = index_sections(&bytes, body_start, &entries).unwrap();
        let (a_start, a_end) = sections["a"];
        assert_eq!(&bytes[a_start..a_end], b"XYZ");
        let (b_start, b_end) = sections["b"];
        assert_eq!(&bytes[b_start..b_end], b"tail to eof");
    }

    #[test]
    fn index_sections_rejects_a_header_that_does_not_match() {
        let bytes = b"~~~a\ndata".to_vec();
        let entries = vec![entry("wrong-name", None)];
        assert!(index_sections(&bytes, 0, &entries).is_err());
    }

    #[test]
    fn index_sections_rejects_a_size_that_runs_past_eof() {
        let bytes = b"~~~a\nXY".to_vec();
        let entries = vec![entry("a", Some(100))];
        assert!(index_sections(&bytes, 0, &entries).is_err());
    }

    #[test]
    fn index_sections_rejects_an_overflowing_size_instead_of_panicking() {
        let bytes = b"~~~a\nXY".to_vec();
        let entries = vec![entry("a", Some(u64::MAX))];
        assert!(
            index_sections(&bytes, 0, &entries).is_err(),
            "a section size near u64::MAX must be rejected, not overflow `start + size`"
        );
    }
}
