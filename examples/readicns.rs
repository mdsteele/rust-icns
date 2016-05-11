extern crate icns;

use icns::IconFamily;
use std::env;
use std::fs::File;
use std::io::BufReader;

fn main() {
    if env::args().count() != 2 {
        println!("Usage: readicns <path>");
        return;
    }
    let path = env::args().nth(1).unwrap();
    let file = File::open(path).expect("failed to open file");
    let buffered = BufReader::new(file);
    let family = IconFamily::read(buffered).expect("failed to read ICNS file");
    println!("ICNS file contains {} element(s).", family.elements.len());
    for (index, element) in family.elements.iter().enumerate() {
        println!("Element {}: {} ({} byte payload)",
                 index,
                 element.ostype,
                 element.data.len());
    }
}
