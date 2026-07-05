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
            .ok_or(McError::MissingSection("section"))?;
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
