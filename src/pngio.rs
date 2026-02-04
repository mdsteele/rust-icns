use image::{Image, PixelFormat};
use png;
use std::io::{self, BufRead, Seek, Write};

impl Image {
    /// Reads an image from a PNG file.
    pub fn read_png<R: BufRead + Seek>(input: R) -> io::Result<Image> {
        let mut decoder = png::Decoder::new(input);
        decoder.set_transformations(
            png::Transformations::STRIP_16 | png::Transformations::EXPAND,
        );
        let info = decoder.read_header_info()?;
        let (width, height) = (info.width, info.height);
        let mut reader = decoder.read_info()?;

        let (color_type, bit_depth) = reader.output_color_type();
        assert!(bit_depth == png::BitDepth::Eight);
        let pixel_format = match color_type {
            png::ColorType::Rgba => PixelFormat::RGBA,
            png::ColorType::Rgb => PixelFormat::RGB,
            png::ColorType::GrayscaleAlpha => PixelFormat::GrayAlpha,
            png::ColorType::Grayscale => PixelFormat::Gray,
            _ => unreachable!(), // EXPAND prevents paletted output
        };

        let mut image = Image::new(pixel_format, width, height);
        assert_eq!(Some(image.data().len()), reader.output_buffer_size());
        reader.next_frame(image.data_mut())?;
        reader.finish()?;
        Ok(image)
    }

    /// Writes the image to a PNG file.
    pub fn write_png<W: Write>(&self, output: W) -> io::Result<()> {
        let color_type = match self.format {
            PixelFormat::RGBA => png::ColorType::Rgba,
            PixelFormat::RGB => png::ColorType::Rgb,
            PixelFormat::GrayAlpha => png::ColorType::GrayscaleAlpha,
            PixelFormat::Gray => png::ColorType::Grayscale,
            PixelFormat::Alpha => {
                return self
                    .convert_to(PixelFormat::GrayAlpha)
                    .write_png(output);
            }
        };
        let mut encoder = png::Encoder::new(output, self.width, self.height);
        encoder.set_color(color_type);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&self.data)?;
        Ok(())
    }
}
