use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::cmp;
use std::io::{self, Error, ErrorKind, Read, Write};

use super::icontype::{Encoding, IconType, OSType};
use super::image::{Image, PixelFormat};

/// The length of an icon element header, in bytes:
const ICON_ELEMENT_HEADER_LENGTH: u32 = 8;

/// The first twelve bytes of a JPEG 2000 file are always this:
#[cfg(feature = "pngio")]
const JPEG_2000_FILE_MAGIC_NUMBER: [u8; 12] =
    [0x00, 0x00, 0x00, 0x0C, 0x6A, 0x50, 0x20, 0x20, 0x0D, 0x0A, 0x87, 0x0A];

/// One data block in an ICNS file.  Depending on the resource type, this may
/// represent an icon, or part of an icon (such as an alpha mask, or color
/// data without the mask).
pub struct IconElement {
    /// The OSType for this element (e.g. `it32` or `t8mk`).
    pub ostype: OSType,
    /// The raw data payload for this element.
    pub data: Vec<u8>,
}

impl IconElement {
    /// Creates an icon element with the given OSType and data payload.
    pub fn new(ostype: OSType, data: Vec<u8>) -> IconElement {
        IconElement { ostype, data }
    }

    /// Creates an icon element that encodes the given image as the given icon
    /// type.  Image color channels that aren't relevant to the specified icon
    /// type will be ignored (e.g. if the icon type is a mask, then only the
    /// alpha channel of the image will be used).  Returns an error if the
    /// image dimensions don't match the icon type.
    ///
    /// Note that if `icon_type` has an associated mask type, this method will
    /// _not_ encode the mask, and will in fact ignore any alpha channel in the
    /// image; you'll need to encode a second `IconElement` with the mask type.
    /// For a higher-level interface that will encode both elements at once,
    /// see the [`IconFamily.add_icon_with_type`](
    /// struct.IconFamily.html#method.add_icon_with_type) method.
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
            #[cfg(feature = "pngio")]
            Encoding::JP2PNG => {
                data = Vec::new();
                image.write_png(&mut data)?;
            }
            #[cfg(not(feature = "pngio"))]
            Encoding::JP2PNG => unimplemented!(),
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
            Encoding::Mono => {
                let image = image.convert_to(PixelFormat::Gray);
                assert!(image.data().len().is_multiple_of(8));
                data = vec![0; image.data.len() / 8];
                for (i, e) in image.into_data().iter().enumerate() {
                    // Arbitrarily threshold gray values to black and white
                    if *e < 128 {
                        data[i / 8] |= 1 << (7 - (i % 8));
                    }
                }
            }
            Encoding::MonoA => {
                let image = image.convert_to(PixelFormat::GrayAlpha);
                assert!(image.data().len().is_multiple_of(16));
                data = vec![0; image.data.len() / 8];
                let (mono, alpha) = data.split_at_mut(image.data.len() / 16);
                for (i, e) in image.into_data().chunks_exact(2).enumerate() {
                    // Arbitrarily threshold gray values to black and white
                    if e[0] < 128 {
                        mono[i / 8] |= 1 << (7 - (i % 8));
                    }
                    if e[1] >= 128 {
                        alpha[i / 8] |= 1 << (7 - (i % 8));
                    }
                }
            }
        }
        Ok(IconElement::new(icon_type.ostype(), data))
    }

    /// Decodes the icon element into an image.  Returns an error if this
    /// element does not represent an icon type supported by this library, or
    /// if the data is malformed.
    ///
    /// Note that if the element's icon type has an associated mask type, this
    /// method will simply produce an image with no alpha channel (since the
    /// mask lives in a separate `IconElement`).  To decode image and mask
    /// together into a single image, you can either use the
    /// [`decode_image_with_mask`](#method.decode_image_with_mask) method,
    /// or the higher-level [`IconFamily.get_icon_with_type`](
    /// struct.IconFamily.html#method.get_icon_with_type) method.
    pub fn decode_image(&self) -> io::Result<Image> {
        let icon_type = self.icon_type().ok_or_else(|| {
            Error::new(ErrorKind::InvalidInput,
                       format!("unsupported OSType: {}", self.ostype))
        })?;
        let width = icon_type.pixel_width();
        let height = icon_type.pixel_height();
        match icon_type.encoding() {
            #[cfg(feature = "pngio")]
            Encoding::JP2PNG => {
                if self.data.starts_with(&JPEG_2000_FILE_MAGIC_NUMBER) {
                    let msg = "element to be decoded contains JPEG 2000 \
                               data, which is not yet supported";
                    return Err(Error::new(ErrorKind::InvalidInput, msg));
                }
                let image = Image::read_png(io::Cursor::new(&self.data))?;
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
            #[cfg(not(feature = "pngio"))]
            Encoding::JP2PNG => unimplemented!(),
            Encoding::RLE24 => {
                let mut image = Image::new(PixelFormat::RGB, width, height);
                decode_rle(&self.data, 3, image.data_mut())?;
                Ok(image)
            }
            Encoding::Mask8 => {
                let num_pixels = width * height;
                if self.data.len() != num_pixels as usize {
                    let msg = format!(
                        "wrong data payload length ({} \
                                       instead of {})",
                        self.data.len(),
                        num_pixels
                    );
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }
                let mut image = Image::new(PixelFormat::Alpha, width, height);
                image.data_mut().copy_from_slice(&self.data);
                Ok(image)
            }
            Encoding::Mono => {
                assert!((width * height).is_multiple_of(8));
                let num_bytes = (width * height) / 8;
                if self.data.len() != num_bytes as usize {
                    let msg = format!(
                        "wrong data payload length ({} \
                                       instead of {})",
                        self.data.len(),
                        num_bytes
                    );
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }
                let mut image = Image::new(PixelFormat::Gray, width, height);
                let out = image.data_mut();
                for (i, b) in self.data.iter().enumerate() {
                    for d in 0..8 {
                        out[8 * i + d] = 0xff - 0xff * ((b >> (7 - d)) & 0x1)
                    }
                }
                Ok(image)
            }
            Encoding::MonoA => {
                assert!((width * height).is_multiple_of(8));
                let num_bytes = (width * height) / 4;
                if self.data.len() != num_bytes as usize {
                    let msg = format!(
                        "wrong data payload length ({} \
                                       instead of {})",
                        self.data.len(),
                        num_bytes
                    );
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }
                let mut image =
                    Image::new(PixelFormat::GrayAlpha, width, height);
                let out = image.data_mut();
                let (mono, alpha) =
                    self.data.split_at((num_bytes / 2) as usize);

                for (i, b) in mono.iter().enumerate() {
                    for d in 0..8 {
                        out[16 * i + 2 * d] =
                            0xff - 0xff * ((b >> (7 - d)) & 0x1)
                    }
                }
                for (i, b) in alpha.iter().enumerate() {
                    for d in 0..8 {
                        out[16 * i + 2 * d + 1] = 0xff * ((b >> (7 - d)) & 0x1)
                    }
                }
                Ok(image)
            }
        }
    }

    /// Decodes this element, together with a separate mask element, into a
    /// single image with alpha channel.  Returns an error if this element does
    /// not represent an icon type supported by this library, or if the given
    /// mask element does not represent the correct mask type for this element,
    /// or if any of the data is malformed.
    ///
    /// For a more convenient alternative to this method, consider using the
    /// higher-level [`IconFamily.get_icon_with_type`](
    /// struct.IconFamily.html#method.get_icon_with_type) method instead.
    pub fn decode_image_with_mask(&self,
                                  mask: &IconElement)
                                  -> io::Result<Image> {
        let icon_type = self.icon_type().ok_or_else(|| {
            Error::new(ErrorKind::InvalidInput,
                       format!("unsupported OSType: {}", self.ostype))
        })?;
        let mask_type = icon_type.mask_type().ok_or_else(|| {
            let msg = format!("icon type {:?} does not use a mask", icon_type);
            Error::new(ErrorKind::InvalidInput, msg)
        })?;
        assert_eq!(icon_type.encoding(), Encoding::RLE24);
        if mask.ostype != mask_type.ostype() {
            let msg = format!("wrong OSType for mask ('{}' instead of '{}')",
                              mask.ostype,
                              mask_type.ostype());
            return Err(Error::new(ErrorKind::InvalidInput, msg));
        }
        let width = icon_type.pixel_width();
        let height = icon_type.pixel_height();
        let num_pixels = (width * height) as usize;
        if mask.data.len() != num_pixels {
            let msg = format!("wrong mask data payload length ({} instead \
                               of {})",
                              mask.data.len(),
                              num_pixels);
            return Err(Error::new(ErrorKind::InvalidInput, msg));
        }
        let mut image = Image::new(PixelFormat::RGBA, width, height);
        decode_rle(&self.data, 4, image.data_mut())?;
        for (i, &alpha) in mask.data.iter().enumerate() {
            image.data_mut()[4 * i + 3] = alpha;
        }
        Ok(image)
    }

    /// Returns the type of icon encoded by this element, or `None` if this
    /// element does not encode a supported icon type.
    pub fn icon_type(&self) -> Option<IconType> {
        IconType::from_ostype(self.ostype)
    }

    /// Reads an icon element from within an ICNS file.
    pub fn read<R: Read>(mut reader: R) -> io::Result<IconElement> {
        let mut raw_ostype = [0u8; 4];
        reader.read_exact(&mut raw_ostype)?;
        let element_length = reader.read_u32::<BigEndian>()?;
        if element_length < ICON_ELEMENT_HEADER_LENGTH {
            return Err(Error::new(ErrorKind::InvalidData,
                                  "invalid element length"));
        }
        let data_length = element_length - ICON_ELEMENT_HEADER_LENGTH;
        let mut data = vec![0u8; data_length as usize];
        reader.read_exact(&mut data)?;
        Ok(IconElement::new(OSType(raw_ostype), data))
    }

    /// Writes the icon element to within an ICNS file.
    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let OSType(ref raw_ostype) = self.ostype;
        writer.write_all(raw_ostype)?;
        writer.write_u32::<BigEndian>(self.total_length())?;
        writer.write_all(&self.data)?;
        Ok(())
    }

    /// Returns the encoded length of the element, in bytes, including the
    /// length of the header.
    pub fn total_length(&self) -> u32 {
        ICON_ELEMENT_HEADER_LENGTH + (self.data.len() as u32)
    }
}

fn encode_rle(input: &[u8],
              num_input_channels: usize,
              num_pixels: usize)
              -> Vec<u8> {
    assert!(num_input_channels == 3 || num_input_channels == 4);
    let mut output = Vec::new();
    if num_pixels == 128 * 128 {
        // The 128x128 RLE icon (it32) starts with four extra zeros.
        output.extend_from_slice(&[0, 0, 0, 0]);
    }
    for channel in 0..3 {
        let mut pixel: usize = 0;
        let mut literal_start: usize = 0;
        while pixel < num_pixels {
            let value = input[num_input_channels * pixel + channel];
            let mut run_length = 1;
            while pixel + run_length < num_pixels &&
                  input[num_input_channels * (pixel + run_length) +
                  channel] == value && run_length < 130 {
                run_length += 1;
            }
            if run_length >= 3 {
                while literal_start < pixel {
                    let literal_length = cmp::min(128, pixel - literal_start);
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
            let literal_length = cmp::min(128, pixel - literal_start);
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

fn decode_rle(input: &[u8],
              num_output_channels: usize,
              output: &mut [u8])
              -> io::Result<()> {
    assert!(num_output_channels == 3 || num_output_channels == 4);
    assert_eq!(output.len() % num_output_channels, 0);
    let num_pixels = output.len() / num_output_channels;
    // Sometimes, RLE-encoded data starts with four extra zeros that must be
    // skipped.
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
                let next: u8 = *iter.next().ok_or_else(rle_error)?;
                if next < 128 {
                    remaining = (next as usize) + 1;
                    within_run = false;
                } else {
                    remaining = (next as usize) - 125;
                    within_run = true;
                    run_value = *iter.next().ok_or_else(rle_error)?;
                }
            }
            output[num_output_channels * pixel + channel] = if within_run {
                run_value
            } else {
                *iter.next().ok_or_else(rle_error)?
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
        assert_eq!(element.ostype, OSType(*b"is32"));
        assert_eq!(element.data[0..5], [1, 44, 55, 128, 66]);
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
        assert_eq!(element.ostype, OSType(*b"s8mk"));
        assert_eq!(element.data[2], 127);
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

    #[test]
    fn decode_rle_with_mask() {
        let color_data: Vec<u8> = vec![0, 12, 255, 0, 250, 0, 128, 34, 255,
                                       0, 248, 0, 1, 56, 99, 255, 0, 249, 0];
        let color_element = IconElement::new(OSType(*b"is32"), color_data);
        let mask_data = vec![78u8; 256];
        let mask_element = IconElement::new(OSType(*b"s8mk"), mask_data);
        let image = color_element.decode_image_with_mask(&mask_element)
            .expect("failed to decode image");
        assert_eq!(image.pixel_format(), PixelFormat::RGBA);
        assert_eq!(image.width(), 16);
        assert_eq!(image.height(), 16);
        assert_eq!(image.data()[0], 12);
        assert_eq!(image.data()[1], 34);
        assert_eq!(image.data()[2], 56);
        assert_eq!(image.data()[3], 78);
    }
}
