//! Cyberlink record: fixed 128-byte layout per cyb-graph spec §cyberlinks.
//!
//! ```text
//! [0..32]    ν   neuron id (hemera hash, 32 B)
//! [32..64]   p   source particle id (hemera hash, 32 B)
//! [64..96]   q   target particle id (hemera hash, 32 B)
//! [96..100]  τ   token denomination index (u32 little-endian)
//! [100..116] a   stake amount (u128 little-endian, smallest unit)
//! [116..117] v   valence (i8: -1, 0, +1)
//! [117..125] t   block height (u64 little-endian)
//! [125..128] _   padding (zero)
//! ```

pub const RECORD_SIZE: usize = 128;

#[derive(Debug, Clone, Copy)]
pub struct Cyberlink {
    pub neuron: [u8; 32],
    pub from: [u8; 32],
    pub to: [u8; 32],
    pub token: u32,
    pub amount: u128,
    pub valence: i8,
    pub block: u64,
}

impl Cyberlink {
    pub fn decode(bytes: &[u8; RECORD_SIZE]) -> Self {
        let mut neuron = [0u8; 32];
        neuron.copy_from_slice(&bytes[0..32]);
        let mut from = [0u8; 32];
        from.copy_from_slice(&bytes[32..64]);
        let mut to = [0u8; 32];
        to.copy_from_slice(&bytes[64..96]);
        let token = u32::from_le_bytes(bytes[96..100].try_into().unwrap());
        let amount = u128::from_le_bytes(bytes[100..116].try_into().unwrap());
        let valence = bytes[116] as i8;
        let block = u64::from_le_bytes(bytes[117..125].try_into().unwrap());
        Self {
            neuron,
            from,
            to,
            token,
            amount,
            valence,
            block,
        }
    }
}

/// Iterate fixed-size records from the mmap'd `cyberlinks` section.
pub struct CyberlinkIter<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> CyberlinkIter<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
}

impl<'a> Iterator for CyberlinkIter<'a> {
    type Item = Cyberlink;
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + RECORD_SIZE > self.bytes.len() {
            return None;
        }
        let chunk: &[u8; RECORD_SIZE] = self.bytes[self.offset..self.offset + RECORD_SIZE]
            .try_into()
            .unwrap();
        self.offset += RECORD_SIZE;
        Some(Cyberlink::decode(chunk))
    }
}

impl<'a> ExactSizeIterator for CyberlinkIter<'a> {
    fn len(&self) -> usize {
        (self.bytes.len() - self.offset) / RECORD_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(
        neuron: u8,
        from: u8,
        to: u8,
        token: u32,
        amount: u128,
        valence: i8,
        block: u64,
    ) -> [u8; RECORD_SIZE] {
        let mut r = [0u8; RECORD_SIZE];
        r[0..32].fill(neuron);
        r[32..64].fill(from);
        r[64..96].fill(to);
        r[96..100].copy_from_slice(&token.to_le_bytes());
        r[100..116].copy_from_slice(&amount.to_le_bytes());
        r[116] = valence as u8;
        r[117..125].copy_from_slice(&block.to_le_bytes());
        r[125..128].fill(0xEE); // padding: decode must not read it.
        r
    }

    #[test]
    fn decode_reads_every_field_at_its_documented_offset() {
        let bytes = record(0x11, 0x22, 0x33, 0xDEAD_BEEF, u128::MAX - 1, 1, 999_999);
        let link = Cyberlink::decode(&bytes);
        assert_eq!(link.neuron, [0x11; 32]);
        assert_eq!(link.from, [0x22; 32]);
        assert_eq!(link.to, [0x33; 32]);
        assert_eq!(link.token, 0xDEAD_BEEF);
        assert_eq!(link.amount, u128::MAX - 1);
        assert_eq!(link.valence, 1);
        assert_eq!(link.block, 999_999);
    }

    #[test]
    fn decode_reads_valence_as_a_signed_byte() {
        // valence is documented as -1, 0, +1, but decode reads the raw byte
        // as i8, so the full signed range must round-trip correctly.
        for (raw, expected) in [(0i8, 0i8), (1, 1), (-1, -1), (i8::MIN, i8::MIN), (i8::MAX, i8::MAX)] {
            let bytes = record(0, 0, 0, 0, 0, raw, 0);
            assert_eq!(Cyberlink::decode(&bytes).valence, expected);
        }
    }

    #[test]
    fn decode_ignores_the_trailing_padding_bytes() {
        let mut a = record(1, 2, 3, 4, 5, 1, 6);
        let mut b = a;
        a[125..128].fill(0x00);
        b[125..128].fill(0xFF);
        assert_eq!(Cyberlink::decode(&a).block, Cyberlink::decode(&b).block);
    }

    #[test]
    fn iter_yields_nothing_on_empty_bytes() {
        let mut it = CyberlinkIter::new(&[]);
        assert!(it.next().is_none());
        assert_eq!(CyberlinkIter::new(&[]).len(), 0);
    }

    #[test]
    fn iter_stops_before_a_partial_trailing_chunk() {
        let mut bytes = record(1, 1, 1, 1, 1, 1, 1).to_vec();
        bytes.extend_from_slice(&[0u8; 17]); // a short, incomplete second record.
        let mut it = CyberlinkIter::new(&bytes);
        assert_eq!(it.len(), 1);
        assert!(it.next().is_some());
        assert!(
            it.next().is_none(),
            "a partial trailing chunk must not be yielded"
        );
    }

    #[test]
    fn iter_yields_every_record_for_an_exact_multiple_of_record_size() {
        let r1 = record(0xAA, 1, 2, 10, 100, 1, 1);
        let r2 = record(0xBB, 3, 4, 20, 200, -1, 2);
        let r3 = record(0xCC, 5, 6, 30, 300, 0, 3);
        let bytes: Vec<u8> = [r1, r2, r3].concat();

        let links: Vec<Cyberlink> = CyberlinkIter::new(&bytes).collect();
        assert_eq!(links.len(), 3);
        assert_eq!(links[0].neuron, [0xAA; 32]);
        assert_eq!(links[1].neuron, [0xBB; 32]);
        assert_eq!(links[2].neuron, [0xCC; 32]);
        assert_eq!(
            (links[0].block, links[1].block, links[2].block),
            (1, 2, 3)
        );
    }

    #[test]
    fn exact_size_iterator_len_tracks_remaining_records_as_it_advances() {
        let bytes: Vec<u8> = [record(1, 1, 1, 1, 1, 1, 1); 3].concat();
        let mut it = CyberlinkIter::new(&bytes);
        assert_eq!(it.len(), 3);
        it.next();
        assert_eq!(it.len(), 2);
        it.next();
        assert_eq!(it.len(), 1);
        it.next();
        assert_eq!(it.len(), 0);
        assert!(it.next().is_none());
    }
}
