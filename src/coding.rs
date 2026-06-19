use std::{
    collections::BinaryHeap,
    cmp::Reverse
};

/// Produce a complete Huffman-encoded file (returns an empty vector on empty input).
/// Format: ([u64; 256], u64, [u8]) for (frequency table, initial input length, encoded input)
pub fn compress(bytes: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();

    if bytes.is_empty() {
        return output;
    }

    let huff = Huffman::from_bytes(bytes);
    let tree = huff.build_tree().unwrap();
    let paths = PathTable::new(&tree);
    let encoded = paths.encode(bytes);

    for &freq in &huff.freqs {
        output.extend_from_slice(&freq.to_le_bytes());
    }

    output.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    output.extend(&encoded);
    output
}

/// Decode a Huffman-encoded file produced by `compress`.
pub fn decompress(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.is_empty() {
        return Some(Vec::new());
    }

    if bytes.len() < 2048 + 8 {
        return None;
    }

    let freqs = bytes[..2048]
        .chunks_exact(8)
        .enumerate()
        .fold([0u64; 256], |mut acc, (i, chunk)| {
            acc[i] = u64::from_le_bytes(chunk.try_into().unwrap());
            acc
        });

    let orig_len = u64::from_le_bytes(bytes[2048..2056].try_into().unwrap()) as usize;

    let huff = Huffman {
        freqs
    };

    let tree = huff.build_tree()?;
    let bits = &bytes[2056..];
    let mut result = Vec::with_capacity(orig_len);
    let mut bit_idx = 0;
    let root = tree.as_ref();

    while result.len() < orig_len {
        let mut node = root;

        loop {
            match node {
                HuffmanNode::Leaf {byte, ..} => {
                    result.push(*byte);
                    break;
                },
                HuffmanNode::Internal {left, right, ..} => {
                    if bit_idx / 8 >= bits.len() {
                        return None; // premature end
                    }

                    let byte = bits[bit_idx / 8];
                    let bit = (byte >> (bit_idx % 8)) & 1;
                    bit_idx += 1;

                    node = if bit == 0 {
                        left
                    } else {
                        right
                    };
                }
            }
        }
    }

    Some(result)
}

#[derive(Clone, Debug)]
pub struct Huffman {
    freqs: [u64; 256]
}

impl Huffman {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            freqs: bytes.iter().fold([0; 256], |mut acc, &i| {
                acc[i as usize] += 1;
                acc
            })
        }
    }

    /// Build the Huffman tree.
    pub fn build_tree(&self) -> Option<Box<HuffmanNode>> {
        let mut heap: BinaryHeap<_> = self
            .freqs
            .iter()
            .enumerate()
            .filter(|&(_, &freq)| freq > 0)
            .map(|(byte, &freq)| {
                Reverse(Box::new(
                    HuffmanNode::Leaf {
                        freq,
                        byte: byte as u8
                    }
                ))
            })
            .collect();

        if heap.len() == 1 {
            let single = heap.pop().unwrap().0;

            let dummy = Box::new(HuffmanNode::Leaf {
                freq: 0,
                byte: 0
            });

            heap.push(Reverse(Box::new(
                HuffmanNode::Internal {
                    freq: single.freq(),
                    left: single,
                    right: dummy
                }
            )));
        }

        while heap.len() > 1 {
            let left = heap.pop().unwrap().0;
            let right = heap.pop().unwrap().0;
            let freq = left.freq() + right.freq();

            heap.push(Reverse(Box::new(
                HuffmanNode::Internal {
                    freq,
                    left,
                    right
                }
            )));
        }

        heap.pop().map(|rev| rev.0)
    }
}

pub struct PathTable {
    paths: [([u8; 256], u8); 256]
}

impl PathTable {
    pub fn new(tree: &HuffmanNode) -> Self {
        let mut table = [([0u8; 256], 0u8); 256];
        let mut stack = vec![(tree, [0u8; 256], 0u8)];

        while let Some((node, path, path_len)) = stack.pop() {
            match node {
                HuffmanNode::Leaf {byte, ..} => {
                    let mut dirs = [0u8; 256];
                    dirs[..path_len as usize].copy_from_slice(&path[..path_len as usize]);
                    table[*byte as usize] = (dirs, path_len);
                },
                HuffmanNode::Internal {left, right, ..} => {
                    let mut left_path = path;
                    left_path[path_len as usize] = 0;
                    stack.push((left, left_path, path_len + 1));

                    let mut right_path = path;
                    right_path[path_len as usize] = 1;
                    stack.push((right, right_path, path_len + 1));
                }
            }
        }

        Self {
            paths: table
        }
    }

    /// Encode a byte sequence using the path table.
    pub fn encode(&self, data: &[u8]) -> Vec<u8> {
        let mut buf = BitBuffer::with_capacity(data.len());

        for &byte in data {
            let (ref dirs, len) = self.paths[byte as usize];

            for &i in dirs.iter().take(len as usize) {
                buf.write_bit(i);
            }
        }

        buf.flush()
    }
}

#[derive(Clone, Debug)]
pub enum HuffmanNode {
    Leaf {
        freq: u64,
        byte: u8
    },
    Internal {
        freq: u64,
        left: Box<HuffmanNode>,
        right: Box<HuffmanNode>
    }
}

impl HuffmanNode {
    /// Return the frequency associated with the node.
    pub const fn freq(&self) -> u64 {
        match self {
            Self::Leaf {freq, ..} => *freq,
            Self::Internal {freq, ..} => *freq
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, HuffmanNode::Leaf {..})
    }

    /// Return the byte associated with the leaf (or `None` if it's an internal node).
    pub fn byte(&self) -> Option<u8> {
        match self {
            Self::Leaf {byte, ..} => Some(*byte),
            Self::Internal {..} => None
        }
    }
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        self.freq() == other.freq()
    }
}

impl Eq for HuffmanNode {}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.freq()
            .cmp(&other.freq())
            .then_with(|| self.byte().cmp(&other.byte()))
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
pub struct BitBuffer {
    output: Vec<u8>,
    buf: u128,
    len: u8
}

impl BitBuffer {
    pub const fn new() -> Self {
        Self {
            output: Vec::new(),
            buf: 0,
            len: 0
        }
    }

    /// Like `BifBuffer::new`, but with the ability to preallocate the output vector.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            output: Vec::with_capacity(cap),
            buf: 0,
            len: 0
        }
    }

    /// Write a single bit (0 or 1).
    pub fn write_bit(&mut self, bit: u8) {
        debug_assert!(bit == 0 || bit == 1);

        self.buf |= (bit as u128) << self.len;
        self.len += 1;

        if self.len == 8 {
            self.output.push(self.buf as u8);
            self.buf = 0;
            self.len = 0;
        }
    }

    /// Flush remaining bits (padded with zeros if necessary).
    pub fn flush(mut self) -> Vec<u8> {
        if self.len > 0 {
            self.output.push(self.buf as u8);
        }

        self.output
    }
}

impl Default for BitBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn random_bytes(data in prop::collection::vec(any::<u8>(), 0..10000)) {
            let compressed = compress(&data);
            let decompressed = decompress(&compressed).expect("decompression failed");
            prop_assert_eq!(decompressed, data);
        }
    }

    #[test]
    fn empty_input() {
        assert!(compress(&[]).is_empty());
        assert_eq!(decompress(&[]), Some(vec![]));
    }

    #[test]
    fn single_symbol() {
        let data = vec![0xFF];
        let compressed = compress(&data);
        assert_eq!(decompress(&compressed).unwrap(), data);
    }

    #[test]
    fn single_symbol_repeated() {
        let data = vec![0xFF; 1024];
        let compressed = compress(&data);
        assert_eq!(decompress(&compressed).unwrap(), data);
    }
}
