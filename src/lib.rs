//! Library for encoding/decoding Apple Icon Image (.icns) files
//!
//! See https://en.wikipedia.org/wiki/Apple_Icon_Image_format for more
//! information about the file format.

#![warn(missing_docs)]

extern crate byteorder;
extern crate png;

mod element;
pub use self::element::IconElement;

mod family;
pub use self::family::IconFamily;

mod icontype;
pub use self::icontype::{Encoding, IconType, OSType};

mod image;
pub use self::image::{Image, PixelFormat};
