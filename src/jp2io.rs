use hayro_jpeg2000::{self, ColorSpace};
use image::{Image, PixelFormat};
use std::io;

impl Image {
    /// Reads an image from a Jpeg 2000 file.
    pub fn read_jp2(input: &[u8]) -> io::Result<Image> {
        let image = match hayro_jpeg2000::Image::new(
            input,
            &hayro_jpeg2000::DecodeSettings {
                resolve_palette_indices: true,
                strict: false,
                target_resolution: None,
            },
        ) {
            Err(y) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, y));
            }
            Ok(x) => x,
        };

        match image.color_space() {
            ColorSpace::Gray => {
                let img_data = image.decode().map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, e)
                })?;
                let mut out = Image::new(
                    if image.has_alpha() {
                        PixelFormat::GrayAlpha
                    } else {
                        PixelFormat::Gray
                    },
                    image.width(),
                    image.height(),
                );
                assert_eq!(img_data.len(), out.data.len());
                out.data_mut().copy_from_slice(&img_data);
                Ok(out)
            }
            ColorSpace::RGB => {
                let img_data = image.decode().map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, e)
                })?;
                let mut out = Image::new(
                    if image.has_alpha() {
                        PixelFormat::RGBA
                    } else {
                        PixelFormat::RGB
                    },
                    image.width(),
                    image.height(),
                );
                assert_eq!(img_data.len(), out.data.len());
                out.data_mut().copy_from_slice(&img_data);
                Ok(out)
            }
            ColorSpace::CMYK => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "jpeg2000 images with CMYK color space not supported"
                    .to_string(),
            )),
            ColorSpace::Unknown { num_channels } => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "jpeg2000 images with Unknown ({num_channels}\
                    -channel) color space not supported"
                ),
            )),
            ColorSpace::Icc { .. } => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "jpeg2000 images with ICC profile not supported".to_string(),
            )),
        }
    }
}
