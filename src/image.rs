use std;

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
