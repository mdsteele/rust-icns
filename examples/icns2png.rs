extern crate icns;

use icns::{IconFamily, OSType};
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::str::FromStr;

fn main() {
    if env::args().count() != 3 {
        println!("Usage: readicns <ostype> <path>");
        return;
    }
    let ostype = OSType::from_str(&env::args().nth(1).unwrap()).unwrap();
    let icns_path = env::args().nth(2).unwrap();
    let icns_path = Path::new(&icns_path);
    let icns_file = BufReader::new(File::open(icns_path)
                                       .expect("failed to open ICNS file"));
    let family = IconFamily::read(icns_file)
                     .expect("failed to read ICNS file");
    let element = family.elements
                        .iter()
                        .find(|el| el.ostype == ostype)
                        .expect("no element with that OSType found");
    let image = element.decode_image().expect("failed to decode image");
    let png_path = icns_path.with_extension(format!("{}.png", ostype));
    let png_file = BufWriter::new(File::create(png_path)
                                      .expect("failed to create PNG file"));
    image.write_png(png_file).expect("failed to write PNG file");
}
