//! Creates an ICNS file containing a single image, read from a PNG file.
//!
//! To create an ICNS file from a PNG, run:
//!
//! ```shell
//! cargo run --example png2icns <path/to/file.png>
//! # ICNS will be saved to path/to/file.icns
//! ```
//!
//! Note that the dimensions of the input image must be exactly those of one of
//! the supported icon types (for example, 32x32 or 128x128).
//!
//! To create an ICNS file from a PNG using a specific icon type within the
//! ICNS file, run:
//!
//! ```shell
//! cargo run --example png2icns <path/to/file.png> <ostype>
//! # ICNS will be saved to path/to/file.<ostype>.icns
//! ```
//!
//! Where <ostype> is the OSType for the icon type you want to encode in.  In
//! this case, the dimensions of the input image must match the particular
//! chosen icon type.

extern crate icns;

use icns::{IconFamily, IconType, Image, OSType};
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::str::FromStr;

fn main() {
    let num_args = env::args().count();
    if num_args < 2 || num_args > 3 {
        println!("Usage: png2icns <path> [<ostype>]");
        return;
    }
    let png_path = env::args().nth(1).unwrap();
    let png_path = Path::new(&png_path);
    let png_file = BufReader::new(File::open(png_path)
        .expect("failed to open PNG file"));
    let image = Image::read_png(png_file).expect("failed to read PNG file");
    let mut family = IconFamily::new();
    let icns_path = if num_args == 3 {
        let ostype = OSType::from_str(&env::args().nth(2).unwrap()).unwrap();
        let icon_type = IconType::from_ostype(ostype)
            .expect("unsupported ostype");
        family.add_icon_with_type(&image, icon_type)
            .expect("failed to encode image");
        png_path.with_extension(format!("{}.icns", ostype))
    } else {
        family.add_icon(&image).expect("failed to encode image");
        png_path.with_extension("icns")
    };
    let icns_file = BufWriter::new(File::create(icns_path)
        .expect("failed to create ICNS file"));
    family.write(icns_file).expect("failed to write ICNS file");
}
