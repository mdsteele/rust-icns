use png;
use png::HasParameters;
use std;
use std::io::{self, Write};

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

    /// Creates a copy of this image using the RGBA pixel format (that is,
    /// `foo.to_rgba().pixel_format()` will always return `PixelFormat::RGBA`).
    /// If the source image is already in RGBA format, this is equivalant to
    /// simply calling `clone()`.
    pub fn to_rgba(&self) -> Image {
        let rgba_data = match self.format {
            PixelFormat::RGBA => self.data.clone(),
            PixelFormat::RGB => rgb_to_rgba(&self.data),
            PixelFormat::Grayscale => grayscale_to_rgba(&self.data),
        };
        Image {
            format: PixelFormat::RGBA,
            width: self.width,
            height: self.height,
            data: rgba_data,
        }
    }

    /// Writes the image to a PNG file (or other writer).
    pub fn write_png<W: Write>(&self, output: W) -> io::Result<()> {
        let color_type = match self.format {
            PixelFormat::RGBA => png::ColorType::RGBA,
            PixelFormat::RGB => png::ColorType::RGB,
            PixelFormat::Grayscale => png::ColorType::Grayscale,
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
}

impl PixelFormat {
    /// Returns the number of bits needed to store a single pixel in this
    /// format.
    pub fn bits_per_pixel(self) -> u32 {
        match self {
            PixelFormat::RGBA => 32,
            PixelFormat::RGB => 24,
            PixelFormat::Grayscale => 8,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grayscale_to_rgba() {
        let gray_data: Vec<u8> = vec![63, 127, 191, 255];
        let mut gray_image = Image::new(PixelFormat::Grayscale, 2, 2);
        gray_image.data_mut().clone_from_slice(&gray_data);
        let rgba_image = gray_image.to_rgba();
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
        let rgba_image = rgb_image.to_rgba();
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
}
