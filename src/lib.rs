//! Library for encoding/decoding Apple Icon Image (.icns) files
//!
//! See https://en.wikipedia.org/wiki/Apple_Icon_Image_format for more
//! information about the file format.

#![warn(missing_docs)]

extern crate byteorder;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::fmt;
use std::io::{self, Read, Write};

/// The first four bytes of an ICNS file:
const ICNS_MAGIC_LITERAL: &'static [u8; 4] = b"icns";

/// The length of an icon family header, in bytes:
const ICON_FAMILY_HEADER_LENGTH: u32 = 8;

/// The length of an icon element header, in bytes:
const ICON_ELEMENT_HEADER_LENGTH: u32 = 8;

/// A set of icons stored in a single ICNS file.
pub struct IconFamily {
    elements: Vec<IconElement>,
}

/// One entry in an ICNS file.  Depending on the resource type, this may
/// represent an icon, or part of an icon (such as an alpha mask, or color
/// data without the mask).
pub struct IconElement {
    ostype: OSType,
    data: Vec<u8>,
}

/// A Macintosh OSType (also known as a ResType), used in ICNS files to
/// identify the type of each icon element.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OSType(pub [u8; 4]);

impl fmt::Display for OSType {
    fn fmt(&self, out: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let &OSType(raw) = self;
        for &byte in &raw {
            let character = std::char::from_u32(u32::from(byte)).unwrap();
            try!(write!(out, "{}", character));
        }
        Ok(())
    }
}

impl IconFamily {
    /// Creates a new, empty icon family.
    pub fn new() -> IconFamily {
        IconFamily { elements: Vec::new() }
    }

    /// Returns the icon elements in the family.
    pub fn elements(&self) -> &[IconElement] {
        &self.elements
    }

    /// Add an icon element to the family.
    pub fn add_element(&mut self, element: IconElement) {
        self.elements.push(element);
    }

    /// Returns the encoded length of the file, in bytes, including the
    /// length of the header.
    fn length(&self) -> u32 {
        let mut length = ICON_FAMILY_HEADER_LENGTH;
        for element in &self.elements {
            length += element.length();
        }
        length
    }

    /// Reads an icon family from an ICNS file (or other reader).
    pub fn read<R: Read>(mut reader: R) -> io::Result<IconFamily> {
        let mut magic = [0u8; 4];
        try!(reader.read_exact(&mut magic));
        if magic != *ICNS_MAGIC_LITERAL {
            return read_error("not an icns file (wrong magic literal)");
        }
        let file_length = try!(reader.read_u32::<BigEndian>());
        let mut file_position: u32 = ICON_FAMILY_HEADER_LENGTH;
        let mut family = IconFamily::new();
        while file_position < file_length {
            let element = try!(IconElement::read(reader.by_ref()));
            file_position += element.length();
            family.add_element(element);
        }
        Ok(family)
    }

    /// Writes the icon family to an ICNS file (or other writer).
    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        try!(writer.write_all(ICNS_MAGIC_LITERAL));
        try!(writer.write_u32::<BigEndian>(self.length()));
        for element in &self.elements {
            try!(element.write(writer.by_ref()));
        }
        Ok(())
    }
}

impl IconElement {
    /// Creates an icon element with the given OSType and data payload.
    pub fn new(ostype: OSType, data: Vec<u8>) -> IconElement {
        IconElement {
            ostype: ostype,
            data: data,
        }
    }

    /// Returns the raw OSType for this element (e.g. `it32` or `t8mk`).
    pub fn ostype(&self) -> OSType {
        self.ostype
    }

    /// Returns the encoded data for this element.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the encoded length of the element, in bytes, including the
    /// length of the header.
    fn length(&self) -> u32 {
        ICON_ELEMENT_HEADER_LENGTH + (self.data.len() as u32)
    }

    /// Reads an icon element from an ICNS file (or other reader).
    fn read<R: Read>(mut reader: R) -> io::Result<IconElement> {
        let mut raw_ostype = [0u8; 4];
        try!(reader.read_exact(&mut raw_ostype));
        let element_length = try!(reader.read_u32::<BigEndian>());
        if element_length < ICON_ELEMENT_HEADER_LENGTH {
            return read_error("invalid element length");
        }
        let data_length = element_length - ICON_ELEMENT_HEADER_LENGTH;
        let mut data = vec![0u8; data_length as usize];
        try!(reader.read_exact(&mut data));
        Ok(IconElement::new(OSType(raw_ostype), data))
    }

    /// Writes the icon element into an ICNS file (or other writer).
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let OSType(ref raw_ostype) = self.ostype;
        try!(writer.write_all(raw_ostype));
        try!(writer.write_u32::<BigEndian>(self.length()));
        try!(writer.write_all(&self.data));
        Ok(())
    }
}

fn read_error<T>(msg: &str) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::InvalidData, msg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};

    #[test]
    fn write_empty_icon_family() {
        let family = IconFamily::new();
        assert_eq!(0, family.elements().len());
        let mut output: Vec<u8> = vec![];
        family.write(&mut output).expect("write failed");
        assert_eq!(b"icns\0\0\0\x08", &output as &[u8]);
    }

    #[test]
    fn read_icon_family_with_fake_elements() {
        let input: Cursor<&[u8]> =
            Cursor::new(b"icns\0\0\0\x1fquux\0\0\0\x0efoobarbaz!\0\0\0\x09#");
        let family = IconFamily::read(input).expect("read failed");
        assert_eq!(2, family.elements().len());
        assert_eq!(OSType(*b"quux"), family.elements()[0].ostype());
        assert_eq!(6, family.elements()[0].data().len());
        assert_eq!(OSType(*b"baz!"), family.elements()[1].ostype());
        assert_eq!(1, family.elements()[1].data().len());
    }

    #[test]
    fn write_icon_family_with_fake_elements() {
        let mut family = IconFamily::new();
        family.add_element(IconElement::new(OSType(*b"quux"),
                                            b"foobar".to_vec()));
        family.add_element(IconElement::new(OSType(*b"baz!"), b"#".to_vec()));
        let mut output: Vec<u8> = vec![];
        family.write(&mut output).expect("write failed");
        assert_eq!(b"icns\0\0\0\x1fquux\0\0\0\x0efoobarbaz!\0\0\0\x09#",
                   &output as &[u8]);
    }
}
