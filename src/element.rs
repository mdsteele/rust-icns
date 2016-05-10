use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor, Read, Write};

use super::icontype::{Encoding, IconType, OSType};
use super::image::Image;

/// The length of an icon element header, in bytes:
const ICON_ELEMENT_HEADER_LENGTH: u32 = 8;

/// One entry in an ICNS file.  Depending on the resource type, this may
/// represent an icon, or part of an icon (such as an alpha mask, or color
/// data without the mask).
pub struct IconElement {
    ostype: OSType,
    data: Vec<u8>,
}

impl IconElement {
    /// Creates an icon element with the given OSType and data payload.
    pub fn new(ostype: OSType, data: Vec<u8>) -> IconElement {
        IconElement {
            ostype: ostype,
            data: data,
        }
    }

    /// Decodes the icon element into an image.  Returns an error if this
    /// element does not represent an icon type supported by this library, or
    /// if the data is malformed.
    pub fn decode_image(&self) -> io::Result<Image> {
        if let Some(icon_type) = self.icon_type() {
            match icon_type.encoding() {
                Encoding::JP2PNG => {
                    // TODO: Detect/Decode JPEG 2000 images.
                    let image = try!(Image::read_png(Cursor::new(&self.data)));
                    if image.width() != icon_type.pixel_width() ||
                       image.height() != icon_type.pixel_height() {
                        Err(io::Error::new(io::ErrorKind::InvalidData,
                                           format!("decoded PNG has wrong \
                                                    dimensions ({}x{} \
                                                    instead of {}x{})",
                                                   image.width(),
                                                   image.height(),
                                                   icon_type.pixel_width(),
                                                   icon_type.pixel_height())))
                    } else {
                        Ok(image)
                    }
                }
                _ => {
                    // TODO: Support RLE and mask icons.
                    Err(io::Error::new(io::ErrorKind::InvalidInput,
                                       format!("unsupported icon type: {:?}",
                                               icon_type)))
                }
            }
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidInput,
                               format!("unsupported OSType: {}",
                                       self.ostype())))
        }
    }

    /// Returns the OSType for this element (e.g. `it32` or `t8mk`).
    pub fn ostype(&self) -> OSType {
        self.ostype
    }

    /// Returns the type of icon encoded by this element, or `None` if this
    /// element does not encode a supported icon type.
    pub fn icon_type(&self) -> Option<IconType> {
        IconType::from_ostype(self.ostype)
    }

    /// Returns the encoded data for this element.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the encoded length of the element, in bytes, including the
    /// length of the header.
    pub fn total_length(&self) -> u32 {
        ICON_ELEMENT_HEADER_LENGTH + (self.data.len() as u32)
    }

    /// Reads an icon element from within an ICNS file.
    pub fn read<R: Read>(mut reader: R) -> io::Result<IconElement> {
        let mut raw_ostype = [0u8; 4];
        try!(reader.read_exact(&mut raw_ostype));
        let element_length = try!(reader.read_u32::<BigEndian>());
        if element_length < ICON_ELEMENT_HEADER_LENGTH {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                      "invalid element length"));
        }
        let data_length = element_length - ICON_ELEMENT_HEADER_LENGTH;
        let mut data = vec![0u8; data_length as usize];
        try!(reader.read_exact(&mut data));
        Ok(IconElement::new(OSType(raw_ostype), data))
    }

    /// Writes the icon element to within an ICNS file.
    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let OSType(ref raw_ostype) = self.ostype;
        try!(writer.write_all(raw_ostype));
        try!(writer.write_u32::<BigEndian>(self.total_length()));
        try!(writer.write_all(&self.data));
        Ok(())
    }
}
