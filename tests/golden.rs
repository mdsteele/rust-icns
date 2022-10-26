extern crate icns;

use icns::{IconFamily, IconType, Image};
use std::fs::File;
use std::io::{self, BufReader};

#[test]
fn decode_is32() {
    decoder_test("is32.icns", IconType::RGB24_16x16, "16x16.png");
}

#[test]
fn encode_is32() {
    encoder_test("16x16.png", IconType::RGB24_16x16, "is32.icns");
}

#[test]
fn decode_il32() {
    decoder_test("il32.icns", IconType::RGB24_32x32, "32x32.png");
}

#[test]
fn encode_il32() {
    encoder_test("32x32.png", IconType::RGB24_32x32, "il32.icns");
}

#[test]
fn decode_it32() {
    decoder_test("it32.icns", IconType::RGB24_128x128, "128x128.png");
}

#[test]
fn encode_it32() {
    encoder_test("128x128.png", IconType::RGB24_128x128, "it32.icns");
}

#[test]
fn decode_icp4() {
    decoder_test("icp4.icns", IconType::RGBA32_16x16, "16x16.png");
}

#[test]
fn encode_icp4() {
    encoder_test("16x16.png", IconType::RGBA32_16x16, "icp4.icns");
}

#[test]
fn decode_icp5() {
    decoder_test("icp5.icns", IconType::RGBA32_32x32, "32x32.png");
}

#[test]
fn encode_icp5() {
    encoder_test("32x32.png", IconType::RGBA32_32x32, "icp5.icns");
}

#[test]
fn decode_ic07() {
    decoder_test("ic07.icns", IconType::RGBA32_128x128, "128x128.png");
}

#[test]
fn encode_ic07() {
    encoder_test("128x128.png", IconType::RGBA32_128x128, "ic07.icns");
}

#[test]
fn decode_ic11() {
    decoder_test("ic11.icns", IconType::RGBA32_16x16_2x, "32x32.png");
}

#[test]
fn encode_ic11() {
    encoder_test("32x32.png", IconType::RGBA32_16x16_2x, "ic11.icns");
}

#[test]
fn decode_ic13() {
    decoder_test("ic13.icns", IconType::RGBA32_128x128_2x, "256x256.png");
}

#[test]
fn decode_ic08(){
    decoder_test("ic08.icns", IconType::RGBA32_256x256, "256x256.png");
}




fn decoder_test(icns_name: &str, icon_type: IconType, png_name: &str) {
    let family = load_icns_file(icns_name).unwrap();
    let image = family.get_icon_with_type(icon_type).unwrap();
    let reference = load_png_file(png_name).unwrap();
    assert_images_match(&image, &reference);
}

fn encoder_test(png_name: &str, icon_type: IconType, icns_name: &str) {
    let image = load_png_file(png_name).unwrap();
    let mut family = IconFamily::new();
    family.add_icon_with_type(&image, icon_type).unwrap();
    let reference = load_icns_file(icns_name).unwrap();
    assert_families_match(&family, &reference);
}

fn load_icns_file(name: &str) -> io::Result<IconFamily> {
    let path = format!("tests/icns/{}", name);
    let file = BufReader::new(File::open(path)?);
    IconFamily::read(file)
}

fn load_png_file(name: &str) -> io::Result<Image> {
    let path = format!("tests/png/{}", name);
    let file = BufReader::new(File::open(path)?);
    Image::read_png(file)
}

fn assert_images_match(image: &Image, reference: &Image) {
    assert_eq!(image.width(), reference.width());
    assert_eq!(image.height(), reference.height());
    assert_eq!(image.pixel_format(), reference.pixel_format());
    assert!(image.data() == reference.data());
}

fn assert_families_match(family: &IconFamily, reference: &IconFamily) {
    let mut family_data = Vec::<u8>::new();
    family.write(&mut family_data).unwrap();
    let mut reference_data = Vec::<u8>::new();
    reference.write(&mut reference_data).unwrap();
    assert!(family_data == reference_data);
}
