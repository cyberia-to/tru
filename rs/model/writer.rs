//! `.model` container — a `.cyb` transformer checkpoint (`specs/model.md`, ct0 §10).
//!
//! Seven sections: `card` `config` `program` `tensors` `vocab` `eval` (text) and
//! `weights` (binary). The text sections are opaque strings the compiler fills
//! (CT-0 pass 8); the `weights` blob is assembled here from [`Tensor`]s — each
//! tensor 4096-byte page-aligned for zero-copy mmap, encoded as integers
//! (no floats on disk). The file is a particle: `hemera(file bytes)`.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::arithmetic::Fx;
use crate::error::{McError, Result};
use crate::graph::frontmatter;

/// Integer storage encoding. CT-0 emits `U16` (projections) and `U32` (norms);
/// `q4`/`q8`/`ternary` are reserved for CT-2.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Encoding {
    U16,
    U32,
}

impl Encoding {
    /// Fractional bits of the fixed-point storage scale (u16 → 2^8, u32 → 2^16).
    fn frac_bits(self) -> u32 {
        match self {
            Encoding::U16 => 8,
            Encoding::U32 => 16,
        }
    }
    fn bytes(self) -> usize {
        match self {
            Encoding::U16 => 2,
            Encoding::U32 => 4,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Encoding::U16 => "u16",
            Encoding::U32 => "u32",
        }
    }
    fn parse(s: &str) -> Result<Self> {
        match s {
            "u16" => Ok(Encoding::U16),
            "u32" => Ok(Encoding::U32),
            other => Err(McError::InvalidGraph(format!("unknown tensor encoding `{other}`"))),
        }
    }
}

/// One named tensor of fixed-point field values (a compile output).
pub struct Tensor {
    pub name: String,
    pub shape: Vec<u64>,
    pub encoding: Encoding,
    pub data: Vec<Fx>,
}

const PAGE: usize = 4096;

/// A `.model` checkpoint. Text sections are opaque; `tensors` becomes `weights`.
pub struct Model {
    pub name: String,
    pub card: String,
    pub config: String,
    pub program: String,
    pub vocab: String,
    pub eval: String,
    pub tensors: Vec<Tensor>,
}

struct Meta {
    name: String,
    shape: Vec<u64>,
    encoding: Encoding,
    offset: usize,
    size: usize,
}

impl Model {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            card: String::new(),
            config: String::new(),
            program: String::new(),
            vocab: String::new(),
            eval: String::new(),
            tensors: Vec::new(),
        }
    }

    /// Assemble the weights blob (page-aligned per tensor) and its index.
    fn build_weights(&self) -> (Vec<u8>, Vec<Meta>) {
        let mut w = Vec::new();
        let mut metas = Vec::with_capacity(self.tensors.len());
        for t in &self.tensors {
            let pad = (PAGE - w.len() % PAGE) % PAGE;
            w.resize(w.len() + pad, 0);
            let offset = w.len();
            let fb = t.encoding.frac_bits();
            for v in &t.data {
                let s = v.to_i64_scaled(fb);
                match t.encoding {
                    Encoding::U16 => w.extend_from_slice(&(s.clamp(i16::MIN as i64, i16::MAX as i64) as i16).to_le_bytes()),
                    Encoding::U32 => w.extend_from_slice(&(s.clamp(i32::MIN as i64, i32::MAX as i64) as i32).to_le_bytes()),
                }
            }
            metas.push(Meta { name: t.name.clone(), shape: t.shape.clone(), encoding: t.encoding, offset, size: w.len() - offset });
        }
        (w, metas)
    }

    fn tensors_toml(metas: &[Meta]) -> String {
        let mut s = String::new();
        for m in metas {
            let shape: Vec<String> = m.shape.iter().map(|d| d.to_string()).collect();
            s.push_str(&format!(
                "[\"{}\"]\nshape = [{}]\nencoding = \"{}\"\noffset = {}\nsize = {}\n\n",
                m.name,
                shape.join(", "),
                m.encoding.name(),
                m.offset,
                m.size
            ));
        }
        s
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let (weights, metas) = self.build_weights();
        let tensors_toml = Self::tensors_toml(&metas);

        let fm = format!(
            "[cyb]\ntypes = [\"model\"]\nname = \"{}\"\n\n\
             [[files]]\nname = \"card\"\nformat = \"md\"\n\n\
             [[files]]\nname = \"config\"\nformat = \"toml\"\n\n\
             [[files]]\nname = \"program\"\nformat = \"rs\"\n\n\
             [[files]]\nname = \"tensors\"\nformat = \"toml\"\n\n\
             [[files]]\nname = \"vocab\"\nformat = \"toml\"\n\n\
             [[files]]\nname = \"eval\"\nformat = \"toml\"\n\n\
             [[files]]\nname = \"weights\"\nformat = \"tensors\"\nsize = {}\n",
            self.name,
            weights.len()
        );

        let mut out = Vec::with_capacity(fm.len() + tensors_toml.len() + weights.len() + 256);
        out.extend_from_slice(fm.as_bytes());
        for (name, text) in [
            ("card", &self.card),
            ("config", &self.config),
            ("program", &self.program),
            ("tensors", &tensors_toml),
            ("vocab", &self.vocab),
            ("eval", &self.eval),
        ] {
            out.extend_from_slice(format!("~~~{name}\n").as_bytes());
            out.extend_from_slice(text.as_bytes());
            out.push(b'\n');
        }
        out.extend_from_slice(b"~~~weights\n");
        out.extend_from_slice(&weights);
        out
    }

    /// The model's own particle: `hemera(file bytes)`.
    pub fn particle(&self) -> [u8; 32] {
        let mut p = [0u8; 32];
        p.copy_from_slice(cyber_hemera::hash(&self.to_bytes()).as_bytes());
        p
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
        if !fm.cyb.types.iter().any(|t| t == "model") {
            return Err(McError::InvalidGraph(format!("container types {:?} does not include \"model\"", fm.cyb.types)));
        }
        let sec = frontmatter::index_sections(bytes, body_start, &fm.files)?;
        let text = |name: &str| -> Result<String> {
            let &(s, e) = sec.get(name).ok_or(McError::MissingSection("model section"))?;
            Ok(std::str::from_utf8(&bytes[s..e]).map_err(|err| McError::InvalidGraph(format!("{name} not utf-8: {err}")))?.to_string())
        };

        let tensors_toml = text("tensors")?;
        let &(ws, we) = sec.get("weights").ok_or(McError::MissingSection("weights"))?;
        let weights = &bytes[ws..we];

        #[derive(Deserialize)]
        struct T {
            shape: Vec<u64>,
            encoding: String,
            offset: u64,
            size: u64,
        }
        let index: HashMap<String, T> = toml::from_str(&tensors_toml)?;
        let mut tensors: Vec<(u64, Tensor)> = Vec::with_capacity(index.len());
        for (name, t) in index {
            let enc = Encoding::parse(&t.encoding)?;
            let (o, sz) = (t.offset as usize, t.size as usize);
            if o + sz > weights.len() {
                return Err(McError::InvalidGraph(format!("tensor `{name}` past weights section")));
            }
            let fb = enc.frac_bits();
            let scale = 1i64 << fb;
            let data: Vec<Fx> = weights[o..o + sz]
                .chunks_exact(enc.bytes())
                .map(|c| match enc {
                    Encoding::U16 => Fx::from_ratio(i16::from_le_bytes([c[0], c[1]]) as i64, scale),
                    Encoding::U32 => Fx::from_ratio(i32::from_le_bytes([c[0], c[1], c[2], c[3]]) as i64, scale),
                })
                .collect();
            tensors.push((t.offset, Tensor { name, shape: t.shape, encoding: enc, data }));
        }
        // Restore write order (weights are laid out by offset) so a reloaded
        // model re-serializes byte-identically.
        tensors.sort_by_key(|(offset, _)| *offset);
        let tensors: Vec<Tensor> = tensors.into_iter().map(|(_, t)| t).collect();

        Ok(Self {
            name: fm.cyb.name,
            card: text("card")?,
            config: text("config")?,
            program: text("program")?,
            vocab: text("vocab")?,
            eval: text("eval")?,
            tensors,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Model {
        let mut m = Model::new("ct0-tiny");
        m.card = "# tiny\ncompiled from a 4-node graph.".into();
        m.config = "hidden_size = 8\nnum_hidden_layers = 1\n".into();
        m.program = "transformer_llama".into();
        m.vocab = "[tokens]\n0 = \"0x1a\"\n".into();
        m.eval = "[ct0_conformance]\nP_DET = 1000\n".into();
        m.tensors = vec![
            Tensor { name: "model.embed_tokens.weight".into(), shape: vec![4, 8], encoding: Encoding::U16, data: (0..32).map(|i| Fx::from_ratio(i - 16, 20)).collect() },
            Tensor { name: "model.norm.weight".into(), shape: vec![8], encoding: Encoding::U32, data: vec![Fx::ONE; 8] },
        ];
        m
    }

    #[test]
    fn tensors_are_page_aligned() {
        let m = sample();
        let (_, metas) = m.build_weights();
        for meta in &metas {
            assert_eq!(meta.offset % PAGE, 0, "tensor `{}` not page-aligned", meta.name);
        }
    }

    #[test]
    fn deterministic_bytes_and_particle() {
        let m = sample();
        assert_eq!(m.to_bytes(), m.to_bytes(), "P-DET: emission is byte-identical");
        assert_eq!(m.particle(), Model::from_bytes(&m.to_bytes()).unwrap().particle());
    }

    #[test]
    fn round_trips_text_and_tensors() {
        let m = sample();
        let r = Model::from_bytes(&m.to_bytes()).unwrap();
        assert_eq!(r.name, m.name);
        assert_eq!(r.config, m.config);
        assert_eq!(r.eval, m.eval);
        assert_eq!(r.tensors.len(), 2);
        for t in &m.tensors {
            let got = r.tensors.iter().find(|x| x.name == t.name).unwrap();
            assert_eq!(got.shape, t.shape);
            assert_eq!(got.encoding, t.encoding);
            let ulp = 1.0 / (1i64 << t.encoding.frac_bits()) as f64;
            for (a, b) in t.data.iter().zip(&got.data) {
                assert!((a.to_f64() - b.to_f64()).abs() <= ulp, "tensor `{}` value drift > ULP", t.name);
            }
        }
    }

    #[test]
    fn writes_and_reads_a_file() {
        let m = sample();
        let path = std::env::temp_dir().join("tru-model-test.model");
        m.write(&path).unwrap();
        let r = Model::read(&path).unwrap();
        assert_eq!(r.particle(), m.particle());
        std::fs::remove_file(&path).ok();
    }
}
