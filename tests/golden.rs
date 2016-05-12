extern crate icns;

use icns::{IconFamily, IconType, Image};
use std::fs::File;
use std::io::{self, BufReader};

#[test]
fn decode_it32() {
    let family = load_icns_file("it32.icns").unwrap();
    let image = family.get_icon_with_type(IconType::RGB24_128x128).unwrap();
    let reference = load_png_file("128x128.png").unwrap();
    assert_images_match(&image, &reference);
}

#[test]
fn encode_it32() {
    let image = load_png_file("128x128.png").unwrap();
    let mut family = IconFamily::new();
    family.add_icon_with_type(&image, IconType::RGB24_128x128).unwrap();
    let reference = load_icns_file("it32.icns").unwrap();
    assert_families_match(&family, &reference);
}

#[test]
fn decode_ic07() {
    let family = load_icns_file("ic07.icns").unwrap();
    let image = family.get_icon_with_type(IconType::RGBA32_128x128).unwrap();
    let reference = load_png_file("128x128.png").unwrap();
    assert_images_match(&image, &reference);
}

#[test]
fn encode_ic07() {
    let image = load_png_file("128x128.png").unwrap();
    let mut family = IconFamily::new();
    family.add_icon_with_type(&image, IconType::RGBA32_128x128).unwrap();
    let reference = load_icns_file("ic07.icns").unwrap();
    assert_families_match(&family, &reference);
}

fn load_icns_file(name: &str) -> io::Result<IconFamily> {
    let path = format!("tests/icns/{}", name);
    let file = BufReader::new(try!(File::open(path)));
    IconFamily::read(file)
}

fn load_png_file(name: &str) -> io::Result<Image> {
    let path = format!("tests/png/{}", name);
    let file = BufReader::new(try!(File::open(path)));
    Image::read_png(file)
}

fn assert_images_match(image: &Image, reference: &Image) {
    assert_eq!(image.width(), reference.width());
    assert_eq!(image.height(), reference.height());
    assert_eq!(image.pixel_format(), reference.pixel_format());
    assert_eq!(image.data(), reference.data());
}

fn assert_families_match(family: &IconFamily, reference: &IconFamily) {
    let mut family_data = Vec::<u8>::new();
    family.write(&mut family_data).unwrap();
    let mut reference_data = Vec::<u8>::new();
    reference.write(&mut reference_data).unwrap();
    assert_eq!(family_data, reference_data);
}
