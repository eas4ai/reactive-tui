//! A digest that stays the same on every Rust release: 64-bit FNV-1a.
//!
//! Recorded goldens and screens store a digest of every cell's colors. The
//! standard library's `DefaultHasher` does not promise the same output
//! across releases, so a toolchain update could change every recording
//! without any change to what was drawn.

/// 64-bit FNV-1a over the bytes written so far.
pub struct Digest(u64);

impl Default for Digest {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Digest {
    /// Add one field's bytes and a separator, so adjacent fields cannot
    /// run together into the same byte stream.
    pub fn field(&mut self, bytes: &[u8]) {
        for &byte in bytes.iter().chain(&[0xff]) {
            self.0 = (self.0 ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    pub fn finish(&self) -> u64 {
        self.0
    }
}
