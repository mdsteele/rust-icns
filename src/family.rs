use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Error, ErrorKind, Read, Write};

use super::element::IconElement;
use super::icontype::IconType;
use super::image::Image;

/// The first four bytes of an ICNS file:
const ICNS_MAGIC_LITERAL: &'static [u8; 4] = b"icns";

/// The length of an icon family header, in bytes:
const ICON_FAMILY_HEADER_LENGTH: u32 = 8;

/// A set of icons stored in a single ICNS file.
pub struct IconFamily {
    /// The icon elements stored in the ICNS file.
    pub elements: Vec<IconElement>,
}

impl IconFamily {
    /// Creates a new, empty icon family.
    pub fn new() -> IconFamily {
        IconFamily { elements: Vec::new() }
    }

    /// Returns true if the icon family contains no icons nor any other
    /// elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Encodes the image into the family, automatically choosing an
    /// appropriate icon type based on the dimensions of the image.  Returns
    /// an error if there is no supported icon type matching the image
    /// dimensions.
    pub fn add_icon(&mut self, image: &Image) -> io::Result<()> {
        if let Some(icon_type) = IconType::from_pixel_size(image.width(),
                                                           image.height()) {
            self.add_icon_with_type(image, icon_type)
        } else {
            let msg = format!("no supported icon type has dimensions {}x{}",
                              image.width(),
                              image.height());
            Err(Error::new(ErrorKind::InvalidInput, msg))
        }
    }

    /// Encodes the image into the family using the given icon type.  If the
    /// selected type has an associated mask type, the image mask will also be
    /// added to the family.  Returns an error if the image has the wrong
    /// dimensions for the selected type.
    pub fn add_icon_with_type(&mut self,
                              image: &Image,
                              icon_type: IconType)
                              -> io::Result<()> {
        self.elements
            .push(try!(IconElement::encode_image_with_type(image, icon_type)));
        if let Some(mask_type) = icon_type.mask_type() {
            self.elements
                .push(try!(IconElement::encode_image_with_type(image,
                                                               mask_type)));
        }
        Ok(())
    }

    /// Returns a list of all (non-mask) icon types for which the icon family
    /// contains the necessary element(s) for a complete icon image (including
    /// alpha channel).  These icon types can be passed to the
    /// [`get_icon_with_type`](#method.get_icon_with_type) method to decode the
    /// icons.
    pub fn available_icons(&self) -> Vec<IconType> {
        let mut result = Vec::new();
        for element in &self.elements {
            if let Some(icon_type) = element.icon_type() {
                if !icon_type.is_mask() {
                    if let Some(mask_type) = icon_type.mask_type() {
                        if self.find_element(mask_type).is_ok() {
                            result.push(icon_type);
                        }
                    } else {
                        result.push(icon_type);
                    }
                }
            }
        }
        result
    }

    /// Determines whether the icon family contains a complete icon with the
    /// given type (including the mask, if the given icon type has an
    /// associated mask type).
    pub fn has_icon_with_type(&self, icon_type: IconType) -> bool {
        if self.find_element(icon_type).is_err() {
            return false;
        } else if let Some(mask_type) = icon_type.mask_type() {
            return self.find_element(mask_type).is_ok();
        }
        true
    }

    /// Decodes an image from the family with the given icon type.  If the
    /// selected type has an associated mask type, the two elements will
    /// decoded together into a single image.  Returns an error if the
    /// element(s) for the selected type are not present in the icon family, or
    /// the if the encoded data is malformed.
    pub fn get_icon_with_type(&self,
                              icon_type: IconType)
                              -> io::Result<Image> {
        let element = try!(self.find_element(icon_type));
        if let Some(mask_type) = icon_type.mask_type() {
            let mask = try!(self.find_element(mask_type));
            element.decode_image_with_mask(mask)
        } else {
            element.decode_image()
        }
    }

    /// Private helper method.
    fn find_element(&self, icon_type: IconType) -> io::Result<&IconElement> {
        let ostype = icon_type.ostype();
        self.elements.iter().find(|el| el.ostype == ostype).ok_or_else(|| {
            let msg = format!("the icon family does not contain a '{}' \
                               element",
                              ostype);
            Error::new(ErrorKind::NotFound, msg)
        })
    }

    /// Reads an icon family from an ICNS file.
    pub fn read<R: Read>(mut reader: R) -> io::Result<IconFamily> {
        let mut magic = [0u8; 4];
        try!(reader.read_exact(&mut magic));
        if magic != *ICNS_MAGIC_LITERAL {
            let msg = "not an icns file (wrong magic literal)";
            return Err(Error::new(ErrorKind::InvalidData, msg));
        }
        let file_length = try!(reader.read_u32::<BigEndian>());
        let mut file_position: u32 = ICON_FAMILY_HEADER_LENGTH;
        let mut family = IconFamily::new();
        while file_position < file_length {
            let element = try!(IconElement::read(reader.by_ref()));
            file_position += element.total_length();
            family.elements.push(element);
        }
        Ok(family)
    }

    /// Writes the icon family to an ICNS file.
    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        try!(writer.write_all(ICNS_MAGIC_LITERAL));
        try!(writer.write_u32::<BigEndian>(self.total_length()));
        for element in &self.elements {
            try!(element.write(writer.by_ref()));
        }
        Ok(())
    }

    /// Returns the encoded length of the file, in bytes, including the
    /// length of the header.
    pub fn total_length(&self) -> u32 {
        let mut length = ICON_FAMILY_HEADER_LENGTH;
        for element in &self.elements {
            length += element.total_length();
        }
        length
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::element::IconElement;
    use super::super::icontype::{IconType, OSType};
    use super::super::image::{Image, PixelFormat};
    use std::io::Cursor;

    #[test]
    fn icon_with_type() {
        let mut family = IconFamily::new();
        assert!(!family.has_icon_with_type(IconType::RGB24_16x16));
        let image = Image::new(PixelFormat::Gray, 16, 16);
        family.add_icon_with_type(&image, IconType::RGB24_16x16).unwrap();
        assert!(family.has_icon_with_type(IconType::RGB24_16x16));
        assert!(family.get_icon_with_type(IconType::RGB24_16x16).is_ok());
    }

    #[test]
    fn write_empty_icon_family() {
        let family = IconFamily::new();
        assert!(family.is_empty());
        assert_eq!(0, family.elements.len());
        let mut output: Vec<u8> = vec![];
        family.write(&mut output).expect("write failed");
        assert_eq!(b"icns\0\0\0\x08", &output as &[u8]);
    }

    #[test]
    fn read_icon_family_with_fake_elements() {
        let input: Cursor<&[u8]> =
            Cursor::new(b"icns\0\0\0\x1fquux\0\0\0\x0efoobarbaz!\0\0\0\x09#");
        let family = IconFamily::read(input).expect("read failed");
        assert_eq!(2, family.elements.len());
        assert_eq!(OSType(*b"quux"), family.elements[0].ostype);
        assert_eq!(6, family.elements[0].data.len());
        assert_eq!(OSType(*b"baz!"), family.elements[1].ostype);
        assert_eq!(1, family.elements[1].data.len());
    }

    #[test]
    fn write_icon_family_with_fake_elements() {
        let mut family = IconFamily::new();
        family.elements
            .push(IconElement::new(OSType(*b"quux"), b"foobar".to_vec()));
        family.elements
            .push(IconElement::new(OSType(*b"baz!"), b"#".to_vec()));
        let mut output: Vec<u8> = vec![];
        family.write(&mut output).expect("write failed");
        assert_eq!(b"icns\0\0\0\x1fquux\0\0\0\x0efoobarbaz!\0\0\0\x09#",
                   &output as &[u8]);
    }
}
