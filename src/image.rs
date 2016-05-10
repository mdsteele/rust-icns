use png;
use png::HasParameters;
use std;
use std::io::{self, Read, Write};

/// A decoded icon image.
#[derive(Clone)]
pub struct Image {
    format: PixelFormat,
    width: u32,
    height: u32,
    data: Box<[u8]>,
}

impl Image {
    /// Creates a new image with all pixel data set to zero.
    pub fn new(format: PixelFormat, width: u32, height: u32) -> Image {
        let data_bits = format.bits_per_pixel() * width * height;
        let data_bytes = (data_bits + 7) / 8;
        Image {
            format: format,
            width: width,
            height: height,
            data: vec![0u8; data_bytes as usize].into_boxed_slice(),
        }
    }

    /// Creates a copy of this image using the specified pixel format.  This
    /// operation always succeeds, but may lose information (e.g. converting
    /// from RGBA to RGB will silently drop the alpha channel).  If the source
    /// image is already in the requested format, this is equivalant to simply
    /// calling `clone()`.
    pub fn convert_to(&self, format: PixelFormat) -> Image {
        let new_data = match self.format {
            PixelFormat::RGBA => {
                match format {
                    PixelFormat::RGBA => self.data.clone(),
                    PixelFormat::RGB => rgba_to_rgb(&self.data),
                    PixelFormat::Grayscale => rgba_to_grayscale(&self.data),
                    PixelFormat::Alpha => rgba_to_alpha(&self.data),
                }
            }
            PixelFormat::RGB => {
                match format {
                    PixelFormat::RGBA => rgb_to_rgba(&self.data),
                    PixelFormat::RGB => self.data.clone(),
                    PixelFormat::Grayscale => rgb_to_grayscale(&self.data),
                    PixelFormat::Alpha => rgb_to_alpha(&self.data),
                }
            }
            PixelFormat::Grayscale => {
                match format {
                    PixelFormat::RGBA => grayscale_to_rgba(&self.data),
                    PixelFormat::RGB => grayscale_to_rgb(&self.data),
                    PixelFormat::Grayscale => self.data.clone(),
                    PixelFormat::Alpha => grayscale_to_alpha(&self.data),
                }
            }
            PixelFormat::Alpha => {
                match format {
                    PixelFormat::RGBA => alpha_to_rgba(&self.data),
                    PixelFormat::RGB => alpha_to_rgb(&self.data),
                    PixelFormat::Grayscale => alpha_to_grayscale(&self.data),
                    PixelFormat::Alpha => self.data.clone(),
                }
            }
        };
        Image {
            format: format,
            width: self.width,
            height: self.height,
            data: new_data,
        }
    }

    /// Reads an image from a PNG file.
    pub fn read_png<R: Read>(input: R) -> io::Result<Image> {
        let decoder = png::Decoder::new(input);
        let (info, mut reader) = try!(decoder.read_info());
        let pixel_format = match info.color_type {
            png::ColorType::RGBA => PixelFormat::RGBA,
            png::ColorType::RGB => PixelFormat::RGB,
            png::ColorType::Grayscale => PixelFormat::Grayscale,
            _ => {
                // TODO: Support other color types.
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                                          format!("unsupported PNG color \
                                                   type: {:?}",
                                                  info.color_type)));
            }
        };
        if info.bit_depth != png::BitDepth::Eight {
            // TODO: Support other bit depths.
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                      format!("unsupported PNG bit depth: \
                                               {:?}",
                                              info.bit_depth)));

        }
        let mut image = Image::new(pixel_format, info.width, info.height);
        assert_eq!(image.data().len(), info.buffer_size());
        try!(reader.next_frame(image.data_mut()));
        Ok(image)
    }

    /// Writes the image to a PNG file.
    pub fn write_png<W: Write>(&self, output: W) -> io::Result<()> {
        let color_type = match self.format {
            PixelFormat::RGBA => png::ColorType::RGBA,
            PixelFormat::RGB => png::ColorType::RGB,
            PixelFormat::Grayscale => png::ColorType::Grayscale,
            PixelFormat::Alpha => {
                // TODO: Add grayscale-with-alpha pixel format so we can be
                // less wasteful here.
                return self.convert_to(PixelFormat::RGBA).write_png(output);
            }
        };
        let mut encoder = png::Encoder::new(output, self.width, self.height);
        encoder.set(color_type).set(png::BitDepth::Eight);
        let mut writer = try!(encoder.write_header());
        writer.write_image_data(&self.data).map_err(|err| {
            match err {
                png::EncodingError::IoError(err) => err,
                png::EncodingError::Format(msg) => {
                    io::Error::new(io::ErrorKind::InvalidData,
                                   msg.into_owned())
                }
            }
        })
    }

    /// Returns the format in which this image's pixel data is stored.
    pub fn pixel_format(&self) -> PixelFormat {
        self.format
    }

    /// Returns the width of the image, in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the image, in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns a reference to the image's pixel data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns a mutable reference to the image's pixel data.
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

/// A format for storing pixel data in an image.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PixelFormat {
    /// 32-bit color with alpha channel.
    RGBA,
    /// 24-bit color with no alpha.
    RGB,
    /// 8-bit grayscale with no alpha.
    Grayscale,
    /// 8-bit alpha mask with no color.
    Alpha,
}

impl PixelFormat {
    /// Returns the number of bits needed to store a single pixel in this
    /// format.
    pub fn bits_per_pixel(self) -> u32 {
        match self {
            PixelFormat::RGBA => 32,
            PixelFormat::RGB => 24,
            PixelFormat::Grayscale => 8,
            PixelFormat::Alpha => 8,
        }
    }
}

/// Converts RGBA image data into RGB.
fn rgba_to_rgb(rgba: &[u8]) -> Box<[u8]> {
    assert_eq!(rgba.len() % 4, 0);
    let num_pixels = rgba.len() / 4;
    let mut rgb = Vec::with_capacity(num_pixels * 3);
    for i in 0..num_pixels {
        rgb.extend_from_slice(&rgba[(4 * i)..(4 * i + 3)]);
    }
    rgb.into_boxed_slice()
}

/// Converts RGB image data into RGBA.
fn rgb_to_rgba(rgb: &[u8]) -> Box<[u8]> {
    assert_eq!(rgb.len() % 3, 0);
    let num_pixels = rgb.len() / 3;
    let mut rgba = Vec::with_capacity(num_pixels * 4);
    for i in 0..num_pixels {
        rgba.extend_from_slice(&rgb[(3 * i)..(3 * i + 3)]);
        rgba.push(std::u8::MAX);
    }
    rgba.into_boxed_slice()
}

/// Converts RGBA image data into grayscale.
fn rgba_to_grayscale(rgba: &[u8]) -> Box<[u8]> {
    assert_eq!(rgba.len() % 4, 0);
    let num_pixels = rgba.len() / 4;
    let mut gray = Vec::with_capacity(num_pixels);
    for i in 0..num_pixels {
        let red = u32::from(rgba[4 * i]);
        let green = u32::from(rgba[4 * i + 1]);
        let blue = u32::from(rgba[4 * i + 2]);
        gray.push(((red + green + blue) / 3) as u8);
    }
    gray.into_boxed_slice()
}

/// Converts RGB image data into grayscale.
fn rgb_to_grayscale(rgb: &[u8]) -> Box<[u8]> {
    assert_eq!(rgb.len() % 3, 0);
    let num_pixels = rgb.len() / 3;
    let mut gray = Vec::with_capacity(num_pixels);
    for i in 0..num_pixels {
        let red = u32::from(rgb[3 * i]);
        let green = u32::from(rgb[3 * i + 1]);
        let blue = u32::from(rgb[3 * i + 2]);
        gray.push(((red + green + blue) / 3) as u8);
    }
    gray.into_boxed_slice()
}

/// Converts RGBA image data into an alpha mask.
fn rgba_to_alpha(rgba: &[u8]) -> Box<[u8]> {
    assert_eq!(rgba.len() % 4, 0);
    let num_pixels = rgba.len() / 4;
    let mut alpha = Vec::with_capacity(num_pixels);
    for i in 0..num_pixels {
        alpha.push(rgba[4 * i + 3]);
    }
    alpha.into_boxed_slice()
}

/// Converts RGB image data into an alpha mask.
fn rgb_to_alpha(rgb: &[u8]) -> Box<[u8]> {
    assert_eq!(rgb.len() % 3, 0);
    let num_pixels = rgb.len() / 3;
    vec![std::u8::MAX; num_pixels].into_boxed_slice()
}

/// Converts grayscale image data into RGBA.
fn grayscale_to_rgba(gray: &[u8]) -> Box<[u8]> {
    let num_pixels = gray.len();
    let mut rgba = Vec::with_capacity(num_pixels * 4);
    for &value in gray {
        rgba.push(value);
        rgba.push(value);
        rgba.push(value);
        rgba.push(std::u8::MAX);
    }
    rgba.into_boxed_slice()
}

/// Converts grayscale image data into RGB.
fn grayscale_to_rgb(gray: &[u8]) -> Box<[u8]> {
    let num_pixels = gray.len();
    let mut rgb = Vec::with_capacity(num_pixels * 3);
    for &value in gray {
        rgb.push(value);
        rgb.push(value);
        rgb.push(value);
    }
    rgb.into_boxed_slice()
}

/// Converts grayscale image data into an alpha mask.
fn grayscale_to_alpha(gray: &[u8]) -> Box<[u8]> {
    vec![std::u8::MAX; gray.len()].into_boxed_slice()
}

/// Converts alpha mask image data into RGBA.
fn alpha_to_rgba(alpha: &[u8]) -> Box<[u8]> {
    let num_pixels = alpha.len();
    let mut rgba = Vec::with_capacity(num_pixels * 4);
    for &value in alpha {
        rgba.push(0);
        rgba.push(0);
        rgba.push(0);
        rgba.push(value);
    }
    rgba.into_boxed_slice()
}

/// Converts alpha mask image data into RGB.
fn alpha_to_rgb(alpha: &[u8]) -> Box<[u8]> {
    vec![0u8; alpha.len() * 3].into_boxed_slice()
}

/// Converts alpha mask image data into grayscale.
fn alpha_to_grayscale(alpha: &[u8]) -> Box<[u8]> {
    vec![0u8; alpha.len()].into_boxed_slice()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn grayscale_to_rgba() {
        let gray_data: Vec<u8> = vec![63, 127, 191, 255];
        let mut gray_image = Image::new(PixelFormat::Grayscale, 2, 2);
        gray_image.data_mut().clone_from_slice(&gray_data);
        let rgba_image = gray_image.convert_to(PixelFormat::RGBA);
        assert_eq!(rgba_image.pixel_format(), PixelFormat::RGBA);
        assert_eq!(rgba_image.width(), 2);
        assert_eq!(rgba_image.height(), 2);
        let rgba_data: Vec<u8> = vec![63, 63, 63, 255, 127, 127, 127, 255,
                                      191, 191, 191, 255, 255, 255, 255, 255];
        assert_eq!(rgba_image.data(), &rgba_data as &[u8]);
    }

    #[test]
    fn rgb_to_rgba() {
        let rgb_data: Vec<u8> = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 127,
                                     127, 127];
        let mut rgb_image = Image::new(PixelFormat::RGB, 2, 2);
        rgb_image.data_mut().clone_from_slice(&rgb_data);
        let rgba_image = rgb_image.convert_to(PixelFormat::RGBA);
        assert_eq!(rgba_image.pixel_format(), PixelFormat::RGBA);
        assert_eq!(rgba_image.width(), 2);
        assert_eq!(rgba_image.height(), 2);
        let rgba_data: Vec<u8> = vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0,
                                      255, 255, 127, 127, 127, 255];
        assert_eq!(rgba_image.data(), &rgba_data as &[u8]);
    }

    #[test]
    fn write_grayscale_png() {
        let gray_data: Vec<u8> = vec![63, 127, 191, 255];
        let mut image = Image::new(PixelFormat::Grayscale, 2, 2);
        image.data_mut().clone_from_slice(&gray_data);
        let mut output: Vec<u8> = Vec::new();
        image.write_png(&mut output).expect("failed to write PNG");
        let expected: Vec<u8> = vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0,
                                     0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0,
                                     0, 2, 8, 0, 0, 0, 0, 87, 221, 82, 248,
                                     0, 0, 0, 17, 73, 68, 65, 84, 120, 1, 1,
                                     6, 0, 249, 255, 1, 63, 64, 1, 191, 64,
                                     4, 8, 1, 129, 255, 68, 9, 75, 0, 0, 0,
                                     0, 73, 69, 78, 68, 174, 66, 96, 130];
        assert_eq!(output, expected);
    }

    #[test]
    fn write_rgb_png() {
        let rgb_data: Vec<u8> = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 127,
                                     127, 127];
        let mut image = Image::new(PixelFormat::RGB, 2, 2);
        image.data_mut().clone_from_slice(&rgb_data);
        let mut output: Vec<u8> = Vec::new();
        image.write_png(&mut output).expect("failed to write PNG");
        let expected: Vec<u8> = vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0,
                                     0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0,
                                     0, 2, 8, 2, 0, 0, 0, 253, 212, 154, 115,
                                     0, 0, 0, 25, 73, 68, 65, 84, 120, 1, 1,
                                     14, 0, 241, 255, 1, 255, 0, 0, 1, 255,
                                     0, 1, 0, 0, 255, 127, 127, 128, 29, 14,
                                     4, 127, 112, 15, 131, 27, 0, 0, 0, 0,
                                     73, 69, 78, 68, 174, 66, 96, 130];
        assert_eq!(output, expected);
    }

    #[test]
    fn write_rgba_png() {
        let rgba_data: Vec<u8> = vec![255, 0, 0, 63, 0, 255, 0, 127, 0, 0,
                                      255, 191, 127, 127, 127, 255];
        let mut image = Image::new(PixelFormat::RGBA, 2, 2);
        image.data_mut().clone_from_slice(&rgba_data);
        let mut output: Vec<u8> = Vec::new();
        image.write_png(&mut output).expect("failed to write PNG");
        let expected: Vec<u8> = vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0,
                                     0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0,
                                     0, 2, 8, 6, 0, 0, 0, 114, 182, 13, 36,
                                     0, 0, 0, 29, 73, 68, 65, 84, 120, 1, 1,
                                     18, 0, 237, 255, 1, 255, 0, 0, 63, 1,
                                     255, 0, 64, 1, 0, 0, 255, 191, 127, 127,
                                     128, 64, 49, 125, 5, 253, 198, 70, 247,
                                     56, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66,
                                     96, 130];
        assert_eq!(output, expected);
    }

    #[test]
    fn read_rgba_png() {
        let png: Vec<u8> = vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13,
                                73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 2, 8, 6,
                                0, 0, 0, 114, 182, 13, 36, 0, 0, 0, 29, 73,
                                68, 65, 84, 120, 1, 1, 18, 0, 237, 255, 1,
                                255, 0, 0, 63, 1, 255, 0, 64, 1, 0, 0, 255,
                                191, 127, 127, 128, 64, 49, 125, 5, 253, 198,
                                70, 247, 56, 0, 0, 0, 0, 73, 69, 78, 68, 174,
                                66, 96, 130];
        let image = Image::read_png(Cursor::new(&png))
                        .expect("failed to read PNG");
        assert_eq!(image.pixel_format(), PixelFormat::RGBA);
        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 2);
        let rgba_data: Vec<u8> = vec![255, 0, 0, 63, 0, 255, 0, 127, 0, 0,
                                      255, 191, 127, 127, 127, 255];
        assert_eq!(image.data(), &rgba_data as &[u8]);
    }

    #[test]
    fn png_round_trip() {
        let rgba_data: Vec<u8> = vec![127, 0, 0, 63, 0, 191, 0, 127, 0, 0,
                                      255, 191, 127, 127, 127, 255];
        let mut rgba_image = Image::new(PixelFormat::RGBA, 2, 2);
        rgba_image.data_mut().clone_from_slice(&rgba_data);
        let pixel_formats = [PixelFormat::RGBA,
                             PixelFormat::RGB,
                             PixelFormat::Grayscale,
                             PixelFormat::Alpha];
        for &format in pixel_formats.iter() {
            // For each pixel format, try writing a PNG from an image in that
            // format.
            let image_1 = rgba_image.convert_to(format);
            let mut png_data = Vec::<u8>::new();
            image_1.write_png(&mut png_data).expect("failed to write PNG");
            // We should be able to read the PNG back in successfully.
            let mut image_2 = Image::read_png(Cursor::new(&png_data))
                                  .expect("failed to read PNG");
            // We may get the image back in a different pixel format.  However,
            // in such cases we should be able to convert back to the original
            // pixel format and still get back exactly the same data.
            if image_2.pixel_format() != image_1.pixel_format() {
                image_2 = image_2.convert_to(image_1.pixel_format());
            }
            assert_eq!(image_1.data(), image_2.data());
        }
    }
}
