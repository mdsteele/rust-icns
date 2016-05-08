//! Library for encoding/decoding Apple Icon Image (.icns) files
//!
//! See https://en.wikipedia.org/wiki/Apple_Icon_Image_format for more
//! information about the file format.

#![warn(missing_docs)]

extern crate byteorder;

use byteorder::{WriteBytesExt, BigEndian};
use std::io::{self, Write};

// The first four bytes of an ICNS file:
const ICNS_MAGIC_LITERAL: &'static [u8; 4] = b"icns";

/// A set of icons stored in a single ICNS file.
pub struct IconFamily {
}

impl IconFamily {
    /// Creates a new, empty icon family.
    pub fn new() -> IconFamily {
        IconFamily {}
    }

    /// Returns the number of icons in the icon family.
    pub fn num_icons(&self) -> u32 {
        0
    }

    /// Writes the icon family to an ICNS file (or other writer).
    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        try!(writer.write_all(ICNS_MAGIC_LITERAL));
        try!(writer.write_u32::<BigEndian>(8));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_empty_icon_family() {
        let family = IconFamily::new();
        assert_eq!(0, family.num_icons());
        let mut output: Vec<u8> = vec![];
        family.write(&mut output).expect("write failed");
        assert_eq!(b"icns\0\0\0\x08", &output as &[u8]);
    }
}
