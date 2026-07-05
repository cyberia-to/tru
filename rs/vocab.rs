//! `.vocab` — the particle → data dictionary (`specs/vocab.md`).
//!
//! A `.vocab` is a `.cyb` container with two sections: `card` (markdown) and
//! `particles` (binary entries). Each entry pairs a 32-byte [[hemera]] particle
//! with the bytes that hash to it. The file is itself a particle:
//! `particle(.vocab) = hemera(file bytes)`.

use std::path::Path;

use crate::error::{McError, Result};
use crate::graph::frontmatter;

/// One `particle → data` mapping. `data` may be empty (a length-zero entry
/// registers a particle's existence without bundling its bytes).
pub struct VocabEntry {
    pub particle: [u8; 32],
    pub data: Vec<u8>,
}

/// A particle dictionary.
pub struct Vocab {
    pub name: String,
    pub card: String,
    pub entries: Vec<VocabEntry>,
}

/// `hemera(bytes)` as a 32-byte particle.
fn particle_of(bytes: &[u8]) -> [u8; 32] {
    let mut p = [0u8; 32];
    p.copy_from_slice(cyber_hemera::hash(bytes).as_bytes());
    p
}

impl Vocab {
    pub fn new(name: impl Into<String>, card: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            card: card.into(),
            entries: Vec::new(),
        }
    }

    /// Append `data`, computing its particle. Returns the particle.
    pub fn push(&mut self, data: Vec<u8>) -> [u8; 32] {
        let particle = particle_of(&data);
        self.entries.push(VocabEntry { particle, data });
        particle
    }

    /// Register a particle with no inlined data (a length-zero entry).
    pub fn register(&mut self, particle: [u8; 32]) {
        self.entries.push(VocabEntry {
            particle,
            data: Vec::new(),
        });
    }

    /// The bytes a particle resolves to, if present with inline data.
    pub fn lookup(&self, particle: &[u8; 32]) -> Option<&[u8]> {
        self.entries
            .iter()
            .find(|e| &e.particle == particle && !e.data.is_empty())
            .map(|e| e.data.as_slice())
    }

    /// The vocab's own particle: `hemera(file bytes)`.
    pub fn particle(&self) -> [u8; 32] {
        particle_of(&self.to_bytes())
    }

    /// Every inlined entry hashes to its declared particle.
    pub fn verify(&self) -> Result<()> {
        for (i, e) in self.entries.iter().enumerate() {
            if !e.data.is_empty() && particle_of(&e.data) != e.particle {
                return Err(McError::Conformance(format!(
                    "vocab entry {i}: hemera(data) ≠ particle"
                )));
            }
        }
        Ok(())
    }

    /// Serialize to `.cyb` container bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // particles section: n (u32 LE) then [particle | len (u64 LE) | data]*.
        let mut particles = Vec::new();
        particles.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        for e in &self.entries {
            particles.extend_from_slice(&e.particle);
            particles.extend_from_slice(&(e.data.len() as u64).to_le_bytes());
            particles.extend_from_slice(&e.data);
        }

        let fm = format!(
            "[cyb]\ntypes = [\"vocab\"]\nname = \"{}\"\n\n[[files]]\nname = \"card\"\nformat = \"md\"\n\n[[files]]\nname = \"particles\"\nformat = \"particles\"\nsize = {}\n",
            self.name,
            particles.len()
        );

        let mut out = Vec::with_capacity(fm.len() + self.card.len() + particles.len() + 32);
        out.extend_from_slice(fm.as_bytes());
        out.extend_from_slice(b"~~~card\n");
        out.extend_from_slice(self.card.as_bytes());
        out.extend_from_slice(b"\n~~~particles\n");
        out.extend_from_slice(&particles);
        out
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        std::fs::write(path, self.to_bytes())?;
        Ok(())
    }

    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_bytes(&std::fs::read(path)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let (fm_str, body_start) = frontmatter::split(bytes)?;
        let fm = frontmatter::parse(fm_str)?;
        if !fm.cyb.types.iter().any(|t| t == "vocab") {
            return Err(McError::InvalidGraph(format!(
                "container types {:?} does not include \"vocab\"",
                fm.cyb.types
            )));
        }
        let sections = frontmatter::index_sections(bytes, body_start, &fm.files)?;

        let &(cs, ce) = sections
            .get("card")
            .ok_or(McError::MissingSection("card"))?;
        let card = std::str::from_utf8(&bytes[cs..ce])
            .map_err(|e| McError::InvalidGraph(format!("card not utf-8: {e}")))?
            .to_string();

        let &(ps, pe) = sections
            .get("particles")
            .ok_or(McError::MissingSection("particles"))?;
        let entries = parse_particles(&bytes[ps..pe])?;

        Ok(Self {
            name: fm.cyb.name,
            card,
            entries,
        })
    }
}

fn parse_particles(b: &[u8]) -> Result<Vec<VocabEntry>> {
    if b.len() < 4 {
        return Err(McError::InvalidGraph(
            "particles section too short for count".into(),
        ));
    }
    let n = u32::from_le_bytes(b[0..4].try_into().unwrap()) as usize;
    let mut entries = Vec::with_capacity(n);
    let mut c = 4;
    for i in 0..n {
        if c + 40 > b.len() {
            return Err(McError::InvalidGraph(format!(
                "particles truncated at entry {i}"
            )));
        }
        let mut particle = [0u8; 32];
        particle.copy_from_slice(&b[c..c + 32]);
        let len = u64::from_le_bytes(b[c + 32..c + 40].try_into().unwrap()) as usize;
        c += 40;
        if c + len > b.len() {
            return Err(McError::InvalidGraph(format!(
                "particles entry {i} data past section end"
            )));
        }
        entries.push(VocabEntry {
            particle,
            data: b[c..c + len].to_vec(),
        });
        c += len;
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vocab {
        let mut v = Vocab::new("test-vocab", "# test\nordering: first-appearance.");
        v.push(b"wiki: a collaborative encyclopedia".to_vec());
        v.push(b"BOOT".to_vec());
        v.register([9u8; 32]); // length-zero entry
        v
    }

    #[test]
    fn round_trips_through_bytes() {
        let v = sample();
        let w = Vocab::from_bytes(&v.to_bytes()).unwrap();
        assert_eq!(w.name, v.name);
        assert_eq!(w.card, v.card);
        assert_eq!(w.entries.len(), 3);
        for (a, b) in v.entries.iter().zip(&w.entries) {
            assert_eq!(a.particle, b.particle);
            assert_eq!(a.data, b.data);
        }
        // the length-zero entry survives with empty data
        assert!(w.entries[2].data.is_empty());
    }

    #[test]
    fn file_particle_is_content_addressed() {
        let v = sample();
        // stable across serializations
        assert_eq!(
            v.particle(),
            Vocab::from_bytes(&v.to_bytes()).unwrap().particle()
        );
        // reordering changes the file particle
        let mut r = Vocab::new("test-vocab", "# test\nordering: first-appearance.");
        r.push(b"BOOT".to_vec());
        r.push(b"wiki: a collaborative encyclopedia".to_vec());
        r.register([9u8; 32]);
        assert_ne!(v.particle(), r.particle());
    }

    #[test]
    fn entries_hash_to_their_particle() {
        let v = sample();
        v.verify().unwrap();
        // a corrupted entry fails verification
        let mut bad = Vocab::new("x", "y");
        bad.entries.push(VocabEntry {
            particle: [7u8; 32],
            data: b"not the preimage".to_vec(),
        });
        assert!(bad.verify().is_err());
    }

    #[test]
    fn lookup_resolves_data() {
        let v = sample();
        let p = particle_of(b"BOOT");
        assert_eq!(v.lookup(&p), Some(&b"BOOT"[..]));
        assert_eq!(v.lookup(&[9u8; 32]), None); // registered but no data
        assert_eq!(v.lookup(&[0u8; 32]), None); // absent
    }

    #[test]
    fn writes_and_reads_a_file() {
        let v = sample();
        let path = std::env::temp_dir().join("tru-vocab-test.vocab");
        v.write(&path).unwrap();
        let w = Vocab::read(&path).unwrap();
        assert_eq!(w.particle(), v.particle());
        std::fs::remove_file(&path).ok();
    }
}
