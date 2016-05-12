//! A library for encoding/decoding Apple Icon Image (.icns) files.
//!
//! # ICNS concepts
//!
//! To understand this library, it helps to be familiar with the structure of
//! an ICNS file; this section will give a high-level overview, or see
//! [Wikipedia](https://en.wikipedia.org/wiki/Apple_Icon_Image_format) for more
//! details about the file format.  If you prefer to learn by example, you can
//! just skip down to the [Example usage](#example-usage) section below.
//!
//! An ICNS file encodes a collection of images (typically different versions
//! of the same icon at different resolutions) called an _icon family_.  The
//! file consists of a short header followed by a sequence of data blocks
//! called _icon elements_.  Each icon element consists of a header with an
//! _OSType_ -- which is essentially a four-byte identifier indicating the type
//! of data in the element -- and a blob of binary data.
//!
//! Each image in the ICNS file is encoded either as a single icon element, or
//! as two elements -- one for the color data and one for the alpha mask.  For
//! example, 48x48 pixel icons are stored as two elements: an `ih32` element
//! containing compressed 24-bit RGB data, and an `h8mk` element containing the
//! 8-bit alpha mask.  By contrast, 64x64 pixel icons are stored as a single
//! `icp6` element, which contains either PNG or JPEG 2000 data for the whole
//! 32-bit image.
//!
//! Some icon sizes have multiple possible encodings.  For example, a 128x128
//! icon can be stored either as an `it32` and an `t8mk` element together
//! (containing compressed RGB and alpha, respectively), or as a single `ic07`
//! element (containing PNG or JPEG 2000 data).  And for some icon sizes, there
//! are separate OSTypes for single and double-pixel-density versions of the
//! icon (for "retina" displays).  For example, an `ic08` element encodes a
//! single-density 256x256 image, while an `ic14` element encodes a
//! double-density 256x256 image -- that is, the image data is actually 512x512
//! pixels, and is considered different from the single-density 512x512 pixel
//! image encoded by an `ic09` element.
//!
//! Finally, there are some additional, optional element types that don't
//! encode images at all.  For example, the `TOC` element summarizes the
//! contents of the ICNS file, and the `icnV` element stores version
//! information.
//!
//! # API overview
//!
//! The API for this library is modelled loosely after that of
//! [libicns](http://icns.sourceforge.net/apidocs.html).
//!
//! The icon family stored in an ICNS file is represeted by the
//! [`IconFamily`](struct.IconFamily.html) struct, which provides methods for
//! [reading](struct.IconFamily.html#method.read) and
//! [writing](struct.IconFamily.html#method.write) ICNS files, as well as for
//! high-level operations on the icon set, such as
//! [adding](struct.IconFamily.html#method.add_icon_with_type),
//! [extracting](struct.IconFamily.html#method.get_icon_with_type), and
//! [listing](struct.IconFamily.html#method.available_icons) the encoded
//! images.
//!
//! An `IconFamily` contains a vector of
//! [`IconElement`](struct.IconElement.html) structs, which represent
//! individual data blocks in the ICNS file.  Each `IconElement` has an
//! [`OSType`](struct.OSType.html) indicating the type of data in the element,
//! as well as a `Vec<u8>` containing the data itself.  Usually, you won't
//! need to work with `IconElement`s directly, and can instead use the
//! higher-level operations provided by `IconFamily`.
//!
//! Since raw OSTypes like `t8mk` and `icp4` can be hard to remember, the
//! [`IconType`](enum.IconType.html) type enumerates all the icon element types
//! that are supported by this library, with more mnemonic names (for example,
//! `IconType::RGB24_48x48` indicates 24-bit RGB data for a 48x48 pixel icon,
//! and is a bit more understandable than the corresponding OSType, `ih32`).
//!  The `IconType` enum also provides methods for getting the properties of
//! each icon type, such as the size of the encoded image, or the associated
//! mask type (for icons that are stored as two elements instead of one).
//!
//! Regardless of whether you use the higher-level `IconFamily` methods or the
//! lower-level `IconElement` methods, icons from the ICNS file can be decoded
//! into [`Image`](struct.Image.html) structs, which can be
//! [converted](struct.Image.html#method.convert_to) to and from any of several
//! [`PixelFormats`](struct.PixelFormat.html) to allow the raw pixel data to be
//! easily transferred to another image library for further processing.  Since
//! this library already depends on the PNG codec anyway (since some ICNS icons
//! are PNG-encoded), as a convenience, the [`Image`](struct.Image.html) struct
//! also provides methods for [reading](struct.Image.html#method.read_png) and
//! [writing](struct.Image.html#method.write_png) PNG files.
//!
//! # Limitations
//!
//! The ICNS format allows some icon types to be encoded either as PNG data or
//! as JPEG 2000 data; however, when encoding icons, this library always uses
//! PNG format, and when decoding icons, it cannot decode JPEG 2000 icons at
//! all (it will detect the JPEG 2000 header and return an error).  The reason
//! for this is the apparent lack of JPEG 2000 libraries for Rust; if this ever
//! changes, please feel free to file a bug or a send a pull request.
//!
//! Additionally, this library does not yet support many of the older icon
//! types used by earlier versions of Mac OS (such as `ICN#`, a 32x32 black and
//! white icon).  Again, pull requests (with suitable tests) are welcome.
//!
//! # Example usage
//!
//! ```no_run
//! use icns::{IconFamily, IconType, Image};
//! use std::fs::File;
//! use std::io::{BufReader, BufWriter};
//!
//! // Load an icon family from an ICNS file.
//! let file = BufReader::new(File::open("16.icns").unwrap());
//! let mut icon_family = IconFamily::read(file).unwrap();
//!
//! // Extract an icon from the family and save it as a PNG.
//! let image = icon_family.get_icon_with_type(IconType::RGB24_16x16).unwrap();
//! let file = BufWriter::new(File::create("16.png").unwrap());
//! image.write_png(file).unwrap();
//!
//! // Read in another icon from a PNG file, and add it to the icon family.
//! let file = BufReader::new(File::open("32.png").unwrap());
//! let image = Image::read_png(file).unwrap();
//! icon_family.add_icon(&image).unwrap();
//!
//! // Save the updated icon family to a new ICNS file.
//! let file = BufWriter::new(File::create("16-and-32.icns").unwrap());
//! icon_family.write(file).unwrap();
//! ```

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
