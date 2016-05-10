use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor, Error, ErrorKind, Read, Write};

use super::icontype::{Encoding, IconType, OSType};
use super::image::{Image, PixelFormat};

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
        let icon_type = try!(self.icon_type().ok_or_else(|| {
            Error::new(ErrorKind::InvalidInput,
                       format!("unsupported OSType: {}", self.ostype()))
        }));
        let width = icon_type.pixel_width();
        let height = icon_type.pixel_width();
        match icon_type.encoding() {
            Encoding::JP2PNG => {
                // TODO: Detect/Decode JPEG 2000 images.
                let image = try!(Image::read_png(Cursor::new(&self.data)));
                if image.width() != width || image.height() != height {
                    let msg = format!("decoded PNG has wrong dimensions \
                                       ({}x{} instead of {}x{})",
                                      image.width(),
                                      image.height(),
                                      width,
                                      height);
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }
                Ok(image)
            }
            Encoding::RLE24 => {
                let mut image = Image::new(PixelFormat::RGB, width, height);
                try!(decode_rle_rgb(&self.data, image.data_mut()));
                Ok(image)
            }
            Encoding::Mask8 => {
                let num_pixels = width * height;
                if self.data.len() as u32 != num_pixels {
                    let msg = format!("wrong data payload length ({} \
                                       instead of {})",
                                      self.data.len(),
                                      num_pixels);
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }
                let mut image = Image::new(PixelFormat::Alpha, width, height);
                image.data_mut().clone_from_slice(&self.data);
                Ok(image)
            }
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
            return Err(Error::new(ErrorKind::InvalidData,
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

fn decode_rle_rgb(input: &[u8], output: &mut [u8]) -> io::Result<()> {
    assert_eq!(output.len() % 3, 0);
    let num_pixels = output.len() / 3;
    let mut iter = input.iter();
    let mut remaining: usize = 0;
    let mut within_run = false;
    let mut run_value: u8 = 0;
    for channel in 0..3 {
        for pixel in 0..num_pixels {
            if remaining == 0 {
                let next: u8 = *try!(iter.next().ok_or_else(rle_error));
                if next < 128 {
                    remaining = (next as usize) + 1;
                    within_run = false;
                } else {
                    remaining = (next as usize) - 125;
                    within_run = true;
                    run_value = *try!(iter.next().ok_or_else(rle_error));
                }
            }
            output[3 * pixel + channel] = if within_run {
                run_value
            } else {
                *try!(iter.next().ok_or_else(rle_error))
            };
            remaining -= 1;
        }
        if remaining != 0 {
            return Err(rle_error());
        }
    }
    if iter.next().is_some() {
        Err(rle_error())
    } else {
        Ok(())
    }
}

fn rle_error() -> Error {
    Error::new(ErrorKind::InvalidData, "invalid RLE-compressed data")
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::icontype::OSType;
    use super::super::image::PixelFormat;

    #[test]
    fn decode_rle() {
        let data: Vec<u8> = vec![0, 12, 255, 0, 250, 0, 128, 34, 255, 0, 248,
                                 0, 1, 56, 99, 255, 0, 249, 0];
        let element = IconElement::new(OSType(*b"is32"), data);
        let image = element.decode_image().expect("failed to decode image");
        assert_eq!(image.pixel_format(), PixelFormat::RGB);
        assert_eq!(image.width(), 16);
        assert_eq!(image.height(), 16);
        assert_eq!(image.data()[0], 12);
        assert_eq!(image.data()[1], 34);
        assert_eq!(image.data()[2], 56);
    }

    #[test]
    fn decode_mask() {
        let mut data = vec![0u8; 256];
        data[2] = 127;
        let element = IconElement::new(OSType(*b"s8mk"), data);
        let image = element.decode_image().expect("failed to decode image");
        assert_eq!(image.pixel_format(), PixelFormat::Alpha);
        assert_eq!(image.width(), 16);
        assert_eq!(image.height(), 16);
        assert_eq!(image.data()[2], 127);
    }
}
