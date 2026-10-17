//! Open and index a `.graph` file: validate frontmatter, locate `~~~` sections
//! (via [`frontmatter::index_sections`]), expose them as slices over the mmap.

use std::collections::HashMap;
use std::path::Path;

use memmap2::Mmap;

use crate::error::{McError, Result};
use crate::graph::frontmatter::{self, Frontmatter};
use crate::graph::record::CyberlinkIter;

pub struct Graph {
    mmap: Mmap,
    frontmatter: Frontmatter,
    sections: HashMap<String, (usize, usize)>,
}

impl Graph {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { Mmap::map(&file) }?;

        let (fm_str, body_start) = frontmatter::split(&mmap[..])?;
        let frontmatter = frontmatter::parse(fm_str)?;

        if !frontmatter.cyb.types.iter().any(|t| t == "graph") {
            return Err(McError::InvalidGraph(format!(
                "container types {:?} does not include \"graph\"",
                frontmatter.cyb.types
            )));
        }

        let sections = frontmatter::index_sections(&mmap[..], body_start, &frontmatter.files)?;
        Ok(Self {
            mmap,
            frontmatter,
            sections,
        })
    }

    pub fn name(&self) -> &str {
        &self.frontmatter.cyb.name
    }

    pub fn frontmatter(&self) -> &Frontmatter {
        &self.frontmatter
    }

    pub fn section(&self, name: &str) -> Result<&[u8]> {
        let &(start, end) = self
            .sections
            .get(name)
            .ok_or_else(|| McError::MissingSection(name.to_string()))?;
        Ok(&self.mmap[start..end])
    }

    pub fn section_str(&self, name: &str) -> Result<&str> {
        let bytes = self.section(name)?;
        std::str::from_utf8(bytes)
            .map_err(|e| McError::InvalidGraph(format!("section `{name}` not utf-8: {e}")))
    }

    pub fn cyberlinks(&self) -> Result<CyberlinkIter<'_>> {
        let bytes = self.section("cyberlinks")?;
        Ok(CyberlinkIter::new(bytes))
    }

    pub fn config_raw(&self) -> Result<&str> {
        self.section_str("config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// A unique scratch path per test, avoiding a new dev-dependency on a
    /// tempfile crate for this small a need.
    fn temp_path(tag: &str) -> std::path::PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("tru-reader-test-{}-{n}-{tag}", std::process::id()))
    }

    fn write_file(tag: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = temp_path(tag);
        let mut f = std::fs::File::create(&path).expect("create scratch file");
        f.write_all(bytes).expect("write scratch file");
        path
    }

    /// A minimal well-formed `.graph` container: `cyb.types` as given, one
    /// zero-length `cyberlinks` section.
    fn minimal_graph_bytes(types: &str) -> Vec<u8> {
        format!(
            "[cyb]\ntypes = [{types}]\nname = \"test\"\n\n\
             [[files]]\nname = \"cyberlinks\"\nformat = \"bin\"\nsize = 0\n\
             ~~~cyberlinks\n"
        )
        .into_bytes()
    }

    #[test]
    fn open_reads_a_minimal_valid_graph() {
        let path = write_file("valid", &minimal_graph_bytes("\"graph\""));
        let g = Graph::open(&path).unwrap();
        assert_eq!(g.name(), "test");
        assert_eq!(g.frontmatter().cyb.types, vec!["graph".to_string()]);
        let links: Vec<_> = g.cyberlinks().unwrap().collect();
        assert!(links.is_empty(), "the zero-length section has no records");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn open_rejects_container_without_graph_type() {
        let path = write_file("wrong-type", &minimal_graph_bytes("\"vocab\""));
        let err = Graph::open(&path).err().unwrap();
        assert!(matches!(err, McError::InvalidGraph(_)));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn open_reports_io_error_for_a_missing_file() {
        let path = temp_path("does-not-exist");
        let err = Graph::open(&path).err().unwrap();
        assert!(matches!(err, McError::Io(_)));
    }

    #[test]
    fn section_names_the_actual_missing_section() {
        // Regression: `section()` used to report the literal string
        // "section" for every miss, not the name that was actually
        // requested — useless for diagnosing a `.graph` file missing one
        // of several declared sections.
        let path = write_file("missing-section", &minimal_graph_bytes("\"graph\""));
        let g = Graph::open(&path).unwrap();
        let err = g.section("config").unwrap_err();
        assert!(matches!(&err, McError::MissingSection(name) if name == "config"));
        assert_eq!(err.to_string(), "missing required section `config`");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn section_str_rejects_non_utf8_bytes() {
        let mut bytes = "[cyb]\ntypes = [\"graph\"]\nname = \"test\"\n\n\
             [[files]]\nname = \"blob\"\nformat = \"bin\"\nsize = 1\n\
             ~~~blob\n"
            .to_string()
            .into_bytes();
        bytes.push(0xFF); // the whole one-byte declared section, not valid utf-8
        let path = write_file("non-utf8", &bytes);
        let g = Graph::open(&path).unwrap();
        let err = g.section_str("blob").unwrap_err();
        assert!(matches!(err, McError::InvalidGraph(_)));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn config_raw_surfaces_the_config_section() {
        let bytes = "[cyb]\ntypes = [\"graph\"]\nname = \"test\"\n\n\
             [[files]]\nname = \"config\"\nformat = \"toml\"\nsize = 4\n\
             ~~~config\nhi=1"
            .to_string()
            .into_bytes();
        let path = write_file("config", &bytes);
        let g = Graph::open(&path).unwrap();
        assert_eq!(g.config_raw().unwrap(), "hi=1");
        let _ = std::fs::remove_file(&path);
    }
}
