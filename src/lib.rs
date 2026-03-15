#[derive(Clone, Debug)]
pub struct Huffman {
    buf: BitBuffer,
    freqs: [u64; 256]
}

impl Huffman {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut freqs = [0; 256];

        for &i in bytes {
            freqs[i as usize] += 1;
        }

        Self {
            buf: BitBuffer::with_capacity(bytes.len()),
            freqs
        }
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

    /// Write `len` bits of `bits` into the buffer.
    /// Assumes `len` <= 64 and there are no bits above `len`.
    pub fn write(&mut self, bits: u64, len: u8) {
        debug_assert!(len <= 64);

        self.buf |= (bits as u128) << self.len;
        self.len += len;

        while self.len >= 8 {
            self.output.push(self.buf as u8);
            self.buf >>= 8;
            self.len -= 8;
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
