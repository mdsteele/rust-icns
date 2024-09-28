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
the most commonly-used types, but some of the older ones are not yet supported.
The table below indicates which types are currently supported; see
https://en.wikipedia.org/wiki/Apple_Icon_Image_format#Icon_types for more
information about each type.


| OSType | Description                             | Supported? |
|--------|-----------------------------------------|------------|
| `ICON` | 32×32 1-bit icon                        | No         |
| `ICN#` | 32×32 1-bit icon with 1-bit mask        | No         |
| `icm#` | 16×12 1-bit icon with 1-bit mask        | No         |
| `icm4` | 16×12 4-bit icon                        | No         |
| `icm8` | 16×12 8-bit icon                        | No         |
| `ics#` | 16×16 1-bit mask                        | No         |
| `ics4` | 16×16 4-bit icon                        | No         |
| `ics8` | 16x16 8-bit icon                        | No         |
| `is32` | 16×16 24-bit icon                       | Yes        |
| `s8mk` | 16x16 8-bit mask                        | Yes        |
| `icl4` | 32×32 4-bit icon                        | No         |
| `icl8` | 32×32 8-bit icon                        | No         |
| `il32` | 32x32 24-bit icon                       | Yes        |
| `l8mk` | 32×32 8-bit mask                        | Yes        |
| `ich#` | 48×48 1-bit mask                        | No         |
| `ich4` | 48×48 4-bit icon                        | No         |
| `ich8` | 48×48 8-bit icon                        | No         |
| `ih32` | 48×48 24-bit icon                       | Yes        |
| `h8mk` | 48×48 8-bit mask                        | Yes        |
| `it32` | 128×128 24-bit icon                     | Yes        |
| `t8mk` | 128×128 8-bit mask                      | Yes        |
| `icp4` | 16x16 32-bit PNG/JP2 icon               | Yes        |
| `icp5` | 32x32 32-bit PNG/JP2 icon               | Yes        |
| `icp6` | 64x64 32-bit PNG/JP2 icon               | Yes        |
| `ic07` | 128x128 32-bit PNG/JP2 icon             | Yes        |
| `ic08` | 256×256 32-bit PNG/JP2 icon             | Yes        |
| `ic09` | 512×512 32-bit PNG/JP2 icon             | Yes        |
| `ic10` | 512x512@2x "retina" 32-bit PNG/JP2 icon | Yes        |
| `ic11` | 16x16@2x "retina" 32-bit PNG/JP2 icon   | Yes        |
| `ic12` | 32x32@2x "retina" 32-bit PNG/JP2 icon   | Yes        |
| `ic13` | 128x128@2x "retina" 32-bit PNG/JP2 icon | Yes        |
| `ic14` | 256x256@2x "retina" 32-bit PNG/JP2 icon | Yes        |

## License

rust-icns is made available under the
[MIT License](http://spdx.org/licenses/MIT.html).
