# rust-icns

[![Build Status](https://github.com/mdsteele/rust-icns/actions/workflows/tests.yml/badge.svg)](https://github.com/mdsteele/rust-icns/actions/workflows/tests.yml)
[![Crates.io](https://img.shields.io/crates/v/icns.svg)](https://crates.io/crates/icns)
[![Documentation](https://docs.rs/icns/badge.svg)](https://docs.rs/icns)

A Rust library for encoding/decoding Apple Icon Image (.icns) files.

## Overview

The `icns` crate implements reading and writing of ICNS files, encoding and
decoding images into and out of an ICNS icon family, converting those images to
other pixel formats (in case you need to transfer the image data to another
library that expects the data in a particular format), and saving/loading those
images to/from PNG files.

The [crate documentation](https://docs.rs/icns) has more information about how
to use the library.

## Example usage

```rust
extern crate icns;
use icns::{IconFamily, IconType, Image};
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() {
    // Load an icon family from an ICNS file.
    let file = BufReader::new(File::open("16.icns").unwrap());
    let mut icon_family = IconFamily::read(file).unwrap();

    // Extract an icon from the family and save it as a PNG.
    let image = icon_family.get_icon_with_type(IconType::RGB24_16x16).unwrap();
    let file = BufWriter::new(File::create("16.png").unwrap());
    image.write_png(file).unwrap();

    // Read in another icon from a PNG file, and add it to the icon family.
    let file = BufReader::new(File::open("32.png").unwrap());
    let image = Image::read_png(file).unwrap();
    icon_family.add_icon(&image).unwrap();

    // Save the updated icon family to a new ICNS file.
    let file = BufWriter::new(File::create("16-and-32.icns").unwrap());
    icon_family.write(file).unwrap();
}
```

## Supported icon types

ICNS files can contain a number of different icon types.  This library supports
all known simple icon data types, but not additional metadata or nested entry
types. See https://en.wikipedia.org/wiki/Apple_Icon_Image_format#Icon_types and
https://github.com/relikd/icns-analysis for more information about each type.

## License

rust-icns is made available under the
[MIT License](http://spdx.org/licenses/MIT.html).
