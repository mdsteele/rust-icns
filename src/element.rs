use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::cmp;
use std::io::{self, Cursor, Error, ErrorKind, Read, Write};

use super::icontype::{Encoding, IconType, OSType};
use super::image::{Image, PixelFormat};

/// The length of an icon element header, in bytes:
const ICON_ELEMENT_HEADER_LENGTH: u32 = 8;

/// The first twelve bytes of a JPEG 2000 file are always this:
const JPEG_2000_FILE_MAGIC_NUMBER: [u8; 12] = [0x00, 0x00, 0x00, 0x0C, 0x6A,
                                               0x50, 0x20, 0x20, 0x0D, 0x0A,
                                               0x87, 0x0A];

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

    /// Creates an icon element that encodes the given image as the given icon
    /// type.  Returns an error if the image has the wrong dimensions for that
    /// icon type.
    pub fn encode_image_with_type(image: &Image,
                                  icon_type: IconType)
                                  -> io::Result<IconElement> {
        let width = icon_type.pixel_width();
        let height = icon_type.pixel_height();
        if image.width() != width || image.height() != height {
            let msg = format!("image has wrong dimensions for {:?} ({}x{} \
                               instead of {}x{}))",
                              icon_type,
                              image.width(),
                              image.height(),
                              width,
                              height);
            return Err(Error::new(ErrorKind::InvalidInput, msg));
        }
        let mut data: Vec<u8>;
        match icon_type.encoding() {
            Encoding::JP2PNG => {
                data = Vec::new();
                try!(image.write_png(&mut data));
            }
            Encoding::RLE24 => {
                let num_pixels = (width * height) as usize;
                match image.pixel_format() {
                    PixelFormat::RGBA => {
                        data = encode_rle(image.data(), 4, num_pixels);
                    }
                    PixelFormat::RGB => {
                        data = encode_rle(image.data(), 3, num_pixels);
                    }
                    // Convert to RGB if the image isn't already RGB or RGBA.
                    _ => {
                        let image = image.convert_to(PixelFormat::RGB);
                        data = encode_rle(image.data(), 3, num_pixels);
                    }
                }
            }
            Encoding::Mask8 => {
                // Convert to Alpha format unconditionally -- if the image is
                // already Alpha format, this will simply clone its data array,
                // which we'd need to do anyway.
                let image = image.convert_to(PixelFormat::Alpha);
                data = image.into_data().into_vec();
            }
        }
        Ok(IconElement::new(icon_type.ostype(), data))
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
                if self.data.starts_with(&JPEG_2000_FILE_MAGIC_NUMBER) {
                    let msg = "element to be decoded contains JPEG 2000 \
                               data, which is not yet supported";
                    return Err(Error::new(ErrorKind::InvalidInput, msg));
                }
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
                try!(decode_rle(&self.data, image.data_mut()));
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

fn encode_rle(input: &[u8],
              num_input_channels: usize,
              num_pixels: usize)
              -> Vec<u8> {
    assert!(num_input_channels == 3 || num_input_channels == 4);
    let mut output = Vec::with_capacity(num_pixels);
    for channel in 0..3 {
        let mut pixel: usize = 0;
        let mut literal_start: usize = 0;
        while pixel < num_pixels {
            let value = input[num_input_channels * pixel + channel];
            let mut run_length = 1;
            while pixel + run_length < num_pixels &&
                  input[num_input_channels * (pixel + run_length) +
                        channel] == value &&
                  run_length < 130 {
                run_length += 1;
            }
            if run_length >= 3 {
                while literal_start < pixel {
                    let literal_length = cmp::min(256, pixel - literal_start);
                    output.push((literal_length - 1) as u8);
                    for i in 0..literal_length {
                        output.push(input[num_input_channels *
                                          (literal_start + i) +
                                          channel]);
                    }
                    literal_start += literal_length;
                }
                output.push((run_length + 125) as u8);
                output.push(value);
                pixel += run_length;
                literal_start = pixel;
            } else {
                pixel += run_length;
            }
        }
        while literal_start < pixel {
            let literal_length = cmp::min(256, pixel - literal_start);
            output.push((literal_length - 1) as u8);
            for i in 0..literal_length {
                output.push(input[num_input_channels * (literal_start + i) +
                                  channel]);
            }
            literal_start += literal_length;
        }
    }
    output
}

fn decode_rle(input: &[u8], output: &mut [u8]) -> io::Result<()> {
    assert_eq!(output.len() % 3, 0);
    let num_pixels = output.len() / 3;
    // Sometimes, RLE-encoded data starts with four extra zeros that must be
    // skipped.  The internet doesn't seem to know why.
    let skip: usize = if input.starts_with(&[0, 0, 0, 0]) {
        4
    } else {
        0
    };
    let input = &input[skip..input.len()];
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
    use super::super::icontype::{IconType, OSType};
    use super::super::image::{Image, PixelFormat};

    #[test]
    fn encode_rle() {
        let mut image = Image::new(PixelFormat::Gray, 16, 16);
        image.data_mut()[0] = 44;
        image.data_mut()[1] = 55;
        image.data_mut()[2] = 66;
        image.data_mut()[3] = 66;
        image.data_mut()[4] = 66;
        let element =
            IconElement::encode_image_with_type(&image, IconType::RGB24_16x16)
                .expect("failed to encode image");
        assert_eq!(element.ostype(), OSType(*b"is32"));
        assert_eq!(element.data()[0..5], [1, 44, 55, 128, 66]);
    }

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
    fn decode_rle_skip_extra_zeros() {
        let data: Vec<u8> = vec![0, 0, 0, 0, 0, 12, 255, 0, 250, 0, 128, 34,
                                 255, 0, 248, 0, 1, 56, 99, 255, 0, 249, 0];
        let element = IconElement::new(OSType(*b"is32"), data);
        let image = element.decode_image().expect("failed to decode image");
        assert_eq!(image.data()[0], 12);
        assert_eq!(image.data()[1], 34);
        assert_eq!(image.data()[2], 56);
    }

    #[test]
    fn encode_mask() {
        let mut image = Image::new(PixelFormat::Alpha, 16, 16);
        image.data_mut()[2] = 127;
        let element =
            IconElement::encode_image_with_type(&image, IconType::Mask8_16x16)
                .expect("failed to encode image");
        assert_eq!(element.ostype(), OSType(*b"s8mk"));
        assert_eq!(element.data()[2], 127);
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
