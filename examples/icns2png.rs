//! Extracts a single icon from an ICNS file and saves it as a PNG.
//!
//! To extract the highest-resolution icon in the ICNS file, run:
//!
//! ```shell
//! cargo run --example icns2png <path/to/file.icns>
//! # image will be saved to path/to/file.png
//! ```
//!
//! To extract a specific icon from the file, run:
//!
//! ```shell
//! cargo run --example icns2png <path/to/file.icns> <ostype>
//! # image will be saved to path/to/file.<ostype>.png
//! ```
//!
//! Where <ostype> is the OSType for the icon you want to extract (e.g. il32
//! for the 32x32 RLE-encoded icon, or ic08 for the 256x256 PNG-encoded icon).
//! See https://en.wikipedia.org/wiki/Apple_Icon_Image_format#Icon_types for a
//! list of possible OSTypes.

extern crate icns;

use icns::{IconFamily, IconType, OSType};
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::str::FromStr;

fn main() {
    let num_args = env::args().count();
    if num_args < 2 || num_args > 3 {
        println!("Usage: icns2png <path> [<ostype>]");
        return;
    }
    let icns_path = env::args().nth(1).unwrap();
    let icns_path = Path::new(&icns_path);
    let icns_file = BufReader::new(File::open(icns_path)
        .expect("failed to open ICNS file"));
    let family = IconFamily::read(icns_file)
        .expect("failed to read ICNS file");
    let (icon_type, png_path) = if num_args == 3 {
        let ostype = OSType::from_str(&env::args().nth(2).unwrap()).unwrap();
        let icon_type = IconType::from_ostype(ostype)
            .expect("unsupported ostype");
        let png_path = icns_path.with_extension(format!("{}.png", ostype));
        (icon_type, png_path)
    } else {
        // If no OSType is specified, extract the highest-resolution icon.
        let &icon_type = family.available_icons()
            .iter()
            .max_by_key(|icon_type| {
                icon_type.pixel_width() * icon_type.pixel_height()
            })
            .expect("ICNS file contains no icons");
        let png_path = icns_path.with_extension("png");
        (icon_type, png_path)
    };
    let image = family.get_icon_with_type(icon_type)
        .expect("ICNS file does not contain that icon type");
    let png_file = BufWriter::new(File::create(png_path)
        .expect("failed to create PNG file"));
    image.write_png(png_file).expect("failed to write PNG file");
}
