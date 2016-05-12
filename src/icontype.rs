use std;
use std::fmt;

/// Types of icon elements that can be decoded as images or masks.
///
/// This type enumerates the kinds of [`IconElement`](struct.IconElement.html)
/// that can be decoded by this library; each `IconType` corresponds to a
/// particular [`OSType`](struct.OSType.html).  The non-mask `IconType` values
/// can also be used with the higher-level
/// [`IconFamily`](struct.IconFamily.html) methods to
/// [encode](struct.IconFamily.html#method.add_icon_with_type) and
/// [decode](struct.IconFamily.html#method.get_icon_with_type) complete icons
/// that consist of multiple `IconElements`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IconType {
    /// 16x16 24-bit icon (without alpha).
    RGB24_16x16,
    /// 16x16 8-bit alpha mask.
    Mask8_16x16,
    /// 32x32 24-bit icon (without alpha).
    RGB24_32x32,
    /// 32x32 8-bit alpha mask.
    Mask8_32x32,
    /// 48x48 24-bit icon (without alpha).
    RGB24_48x48,
    /// 48x48 8-bit alpha mask.
    Mask8_48x48,
    /// 128x128 24-bit icon (without alpha).
    RGB24_128x128,
    /// 128x128 8-bit alpha mask.
    Mask8_128x128,
    /// 16x16 32-bit icon.
    RGBA32_16x16,
    /// 16x16 32-bit icon at 2x "retina" density (so, 32 by 32 pixels).
    RGBA32_16x16_2x,
    /// 32x32 32-bit icon.
    RGBA32_32x32,
    /// 32x32 32-bit icon at 2x "retina" density (so, 64 by 64 pixels).
    RGBA32_32x32_2x,
    /// 64x64 32-bit icon.  (For whatever reason, the ICNS format has no
    /// corresponding type for a 64x64 icon at 2x "retina" density.)
    RGBA32_64x64,
    /// 128x128 32-bit icon.
    RGBA32_128x128,
    /// 128x128 32-bit icon at 2x "retina" density (so, 256 by 256 pixels).
    RGBA32_128x128_2x,
    /// 256x256 32-bit icon.
    RGBA32_256x256,
    /// 256x256 32-bit icon at 2x "retina" density (so, 512 by 512 pixels).
    RGBA32_256x256_2x,
    /// 512x512 32-bit icon.
    RGBA32_512x512,
    /// 512x512 32-bit icon at 2x "retina" density (so, 1024 by 1024 pixels).
    RGBA32_512x512_2x,
}

impl IconType {
    /// Get the icon type associated with the given OSType, if any.
    pub fn from_ostype(ostype: OSType) -> Option<IconType> {
        let OSType(raw_ostype) = ostype;
        match &raw_ostype {
            b"is32" => Some(IconType::RGB24_16x16),
            b"s8mk" => Some(IconType::Mask8_16x16),
            b"il32" => Some(IconType::RGB24_32x32),
            b"l8mk" => Some(IconType::Mask8_32x32),
            b"ih32" => Some(IconType::RGB24_48x48),
            b"h8mk" => Some(IconType::Mask8_48x48),
            b"it32" => Some(IconType::RGB24_128x128),
            b"t8mk" => Some(IconType::Mask8_128x128),
            b"icp4" => Some(IconType::RGBA32_16x16),
            b"ic11" => Some(IconType::RGBA32_16x16_2x),
            b"icp5" => Some(IconType::RGBA32_32x32),
            b"ic12" => Some(IconType::RGBA32_32x32_2x),
            b"icp6" => Some(IconType::RGBA32_64x64),
            b"ic07" => Some(IconType::RGBA32_128x128),
            b"ic13" => Some(IconType::RGBA32_128x128_2x),
            b"ic08" => Some(IconType::RGBA32_256x256),
            b"ic14" => Some(IconType::RGBA32_256x256_2x),
            b"ic09" => Some(IconType::RGBA32_512x512),
            b"ic10" => Some(IconType::RGBA32_512x512_2x),
            _ => None,
        }
    }

    /// Get the OSType that represents this icon type.
    pub fn ostype(self) -> OSType {
        match self {
            IconType::RGB24_16x16 => OSType(*b"is32"),
            IconType::Mask8_16x16 => OSType(*b"s8mk"),
            IconType::RGB24_32x32 => OSType(*b"il32"),
            IconType::Mask8_32x32 => OSType(*b"l8mk"),
            IconType::RGB24_48x48 => OSType(*b"ih32"),
            IconType::Mask8_48x48 => OSType(*b"h8mk"),
            IconType::RGB24_128x128 => OSType(*b"it32"),
            IconType::Mask8_128x128 => OSType(*b"t8mk"),
            IconType::RGBA32_16x16 => OSType(*b"icp4"),
            IconType::RGBA32_16x16_2x => OSType(*b"ic11"),
            IconType::RGBA32_32x32 => OSType(*b"icp5"),
            IconType::RGBA32_32x32_2x => OSType(*b"ic12"),
            IconType::RGBA32_64x64 => OSType(*b"icp6"),
            IconType::RGBA32_128x128 => OSType(*b"ic07"),
            IconType::RGBA32_128x128_2x => OSType(*b"ic13"),
            IconType::RGBA32_256x256 => OSType(*b"ic08"),
            IconType::RGBA32_256x256_2x => OSType(*b"ic14"),
            IconType::RGBA32_512x512 => OSType(*b"ic09"),
            IconType::RGBA32_512x512_2x => OSType(*b"ic10"),
        }
    }

    /// Returns true if this is icon type is a mask for some other icon type.
    ///
    /// # Examples
    /// ```
    /// use icns::IconType;
    /// assert!(!IconType::RGB24_16x16.is_mask());
    /// assert!(IconType::Mask8_16x16.is_mask());
    /// assert!(!IconType::RGBA32_16x16.is_mask());
    /// ```
    pub fn is_mask(self) -> bool {
        match self {
            IconType::Mask8_16x16 |
            IconType::Mask8_32x32 |
            IconType::Mask8_48x48 |
            IconType::Mask8_128x128 => true,
            _ => false,
        }
    }

    /// If this icon type has an associated mask type, returns that mask type;
    /// if this is a mask icon type, or a non-mask icon type that has no
    /// associated mask type, returns `None`.
    ///
    /// # Examples
    /// ```
    /// use icns::IconType;
    /// assert_eq!(IconType::RGB24_16x16.mask_type(),
    ///            Some(IconType::Mask8_16x16));
    /// assert_eq!(IconType::Mask8_16x16.mask_type(), None);
    /// assert_eq!(IconType::RGBA32_16x16.mask_type(), None);
    /// ```
    pub fn mask_type(self) -> Option<IconType> {
        match self {
            IconType::RGB24_16x16 => Some(IconType::Mask8_16x16),
            IconType::RGB24_32x32 => Some(IconType::Mask8_32x32),
            IconType::RGB24_48x48 => Some(IconType::Mask8_48x48),
            IconType::RGB24_128x128 => Some(IconType::Mask8_128x128),
            _ => None,
        }
    }

    /// Returns the pixel data width of this icon type.  Normally this is the
    /// same as the screen width, but for 2x "retina" density icons, this will
    /// be twice that value.
    ///
    /// # Examples
    /// ```
    /// use icns::IconType;
    /// assert_eq!(IconType::Mask8_128x128.pixel_width(), 128);
    /// assert_eq!(IconType::RGBA32_256x256.pixel_width(), 256);
    /// assert_eq!(IconType::RGBA32_256x256_2x.pixel_width(), 512);
    /// ```
    pub fn pixel_width(self) -> u32 {
        self.screen_width() * self.pixel_density()
    }

    /// Returns the pixel data height of this icon type.  Normally this is the
    /// same as the screen height, but for 2x "retina" density icons, this will
    /// be twice that value.
    ///
    /// # Examples
    /// ```
    /// use icns::IconType;
    /// assert_eq!(IconType::Mask8_128x128.pixel_height(), 128);
    /// assert_eq!(IconType::RGBA32_256x256.pixel_height(), 256);
    /// assert_eq!(IconType::RGBA32_256x256_2x.pixel_height(), 512);
    /// ```
    pub fn pixel_height(self) -> u32 {
        self.screen_height() * self.pixel_density()
    }

    /// Returns the pixel density for this icon type -- that is, 2 for 2x
    /// "retina" density icons, or 1 for other icon types.
    ///
    /// # Examples
    /// ```
    /// use icns::IconType;
    /// assert_eq!(IconType::Mask8_128x128.pixel_density(), 1);
    /// assert_eq!(IconType::RGBA32_256x256.pixel_density(), 1);
    /// assert_eq!(IconType::RGBA32_256x256_2x.pixel_density(), 2);
    /// ```
    pub fn pixel_density(self) -> u32 {
        match self {
            IconType::RGBA32_16x16_2x |
            IconType::RGBA32_32x32_2x |
            IconType::RGBA32_128x128_2x |
            IconType::RGBA32_256x256_2x |
            IconType::RGBA32_512x512_2x => 2,
            _ => 1,
        }
    }

    /// Returns the screen width of this icon type.  Normally this is the same
    /// as the pixel width, but for 2x "retina" density icons, this will be
    /// half that value.
    ///
    /// # Examples
    /// ```
    /// use icns::IconType;
    /// assert_eq!(IconType::Mask8_128x128.screen_width(), 128);
    /// assert_eq!(IconType::RGBA32_256x256.screen_width(), 256);
    /// assert_eq!(IconType::RGBA32_256x256_2x.screen_width(), 256);
    /// ```
    pub fn screen_width(self) -> u32 {
        match self {
            IconType::RGB24_16x16 => 16,
            IconType::Mask8_16x16 => 16,
            IconType::RGB24_32x32 => 32,
            IconType::Mask8_32x32 => 32,
            IconType::RGB24_48x48 => 48,
            IconType::Mask8_48x48 => 48,
            IconType::RGB24_128x128 => 128,
            IconType::Mask8_128x128 => 128,
            IconType::RGBA32_16x16 => 16,
            IconType::RGBA32_16x16_2x => 16,
            IconType::RGBA32_32x32 => 32,
            IconType::RGBA32_32x32_2x => 32,
            IconType::RGBA32_64x64 => 64,
            IconType::RGBA32_128x128 => 128,
            IconType::RGBA32_128x128_2x => 128,
            IconType::RGBA32_256x256 => 256,
            IconType::RGBA32_256x256_2x => 256,
            IconType::RGBA32_512x512 => 512,
            IconType::RGBA32_512x512_2x => 512,
        }
    }

    /// Returns the screen height of this icon type.  Normally this is the same
    /// as the pixel height, but for 2x "retina" density icons, this will be
    /// half that value.
    ///
    /// # Examples
    /// ```
    /// use icns::IconType;
    /// assert_eq!(IconType::Mask8_128x128.screen_height(), 128);
    /// assert_eq!(IconType::RGBA32_256x256.screen_height(), 256);
    /// assert_eq!(IconType::RGBA32_256x256_2x.screen_height(), 256);
    /// ```
    pub fn screen_height(self) -> u32 {
        match self {
            IconType::RGB24_16x16 => 16,
            IconType::Mask8_16x16 => 16,
            IconType::RGB24_32x32 => 32,
            IconType::Mask8_32x32 => 32,
            IconType::RGB24_48x48 => 48,
            IconType::Mask8_48x48 => 48,
            IconType::RGB24_128x128 => 128,
            IconType::Mask8_128x128 => 128,
            IconType::RGBA32_16x16 => 16,
            IconType::RGBA32_16x16_2x => 16,
            IconType::RGBA32_32x32 => 32,
            IconType::RGBA32_32x32_2x => 32,
            IconType::RGBA32_64x64 => 64,
            IconType::RGBA32_128x128 => 128,
            IconType::RGBA32_128x128_2x => 128,
            IconType::RGBA32_256x256 => 256,
            IconType::RGBA32_256x256_2x => 256,
            IconType::RGBA32_512x512 => 512,
            IconType::RGBA32_512x512_2x => 512,
        }
    }

    /// Returns the encoding used within an ICNS file for this icon type.
    pub fn encoding(self) -> Encoding {
        match self {
            IconType::RGB24_16x16 |
            IconType::RGB24_32x32 |
            IconType::RGB24_48x48 |
            IconType::RGB24_128x128 => Encoding::RLE24,
            IconType::Mask8_16x16 |
            IconType::Mask8_32x32 |
            IconType::Mask8_48x48 |
            IconType::Mask8_128x128 => Encoding::Mask8,
            IconType::RGBA32_16x16 |
            IconType::RGBA32_16x16_2x |
            IconType::RGBA32_32x32 |
            IconType::RGBA32_32x32_2x |
            IconType::RGBA32_64x64 |
            IconType::RGBA32_128x128 |
            IconType::RGBA32_128x128_2x |
            IconType::RGBA32_256x256 |
            IconType::RGBA32_256x256_2x |
            IconType::RGBA32_512x512 |
            IconType::RGBA32_512x512_2x => Encoding::JP2PNG,
        }
    }
}

/// A Macintosh OSType (also known as a ResType), used in ICNS files to
/// identify the type of each icon element.
///
/// An OSType is a four-byte identifier used throughout Mac OS.  In an ICNS
/// file, it indicates the type of data stored in an
/// [`IconElement`](struct.IconElement.html) data block.  For example, OSType
/// `is32` represents 24-bit color data for a 16x16 icon, while OSType `s8mk`
/// represents the 8-bit alpha mask for that same icon.
///
/// See the [`IconType`](enum.IconType.html) enum for an easier-to-use
/// representation of icon data types.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OSType(pub [u8; 4]);

impl fmt::Display for OSType {
    fn fmt(&self, out: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let &OSType(raw) = self;
        for &byte in &raw {
            let character = std::char::from_u32(u32::from(byte)).unwrap();
            try!(write!(out, "{}", character));
        }
        Ok(())
    }
}

impl std::str::FromStr for OSType {
    type Err = String;

    fn from_str(input: &str) -> Result<OSType, String> {
        let chars: Vec<char> = input.chars().collect();
        if chars.len() != 4 {
            return Err(format!("OSType string must be 4 chars (was {})",
                               chars.len()));
        }
        let mut bytes = [0u8; 4];
        for (i, &ch) in chars.iter().enumerate() {
            let value = ch as u32;
            if value > std::u8::MAX as u32 {
                return Err(format!("OSType chars must have value of at \
                                    most 0x{:X} (found 0x{:X})",
                                   std::u8::MAX,
                                   value));
            }
            bytes[i] = value as u8;
        }
        Ok(OSType(bytes))
    }
}

/// Methods of encoding an image within an icon element.
///
/// Each [`IconType`](enum.IconType.html) uses a particular encoding within
/// an ICNS file; this type enumerates those encodings.
///
/// (This type is used internally by the library, but is irrelvant to most
/// library users; if you're not sure whether you need to use it, you probably
/// don't.)
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Encoding {
    /// Icon element data payload is an uncompressed 8-bit alpha mask.
    Mask8,
    /// Icon element data payload is an RLE-compressed 24-bit RGB image.
    RLE24,
    /// Icon element data payload is a JPEG 2000 or PNG file.
    JP2PNG,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const ALL_ICON_TYPES: [IconType; 19] = [IconType::RGB24_16x16,
                                            IconType::Mask8_16x16,
                                            IconType::RGB24_32x32,
                                            IconType::Mask8_32x32,
                                            IconType::RGB24_48x48,
                                            IconType::Mask8_48x48,
                                            IconType::RGB24_128x128,
                                            IconType::Mask8_128x128,
                                            IconType::RGBA32_16x16,
                                            IconType::RGBA32_16x16_2x,
                                            IconType::RGBA32_32x32,
                                            IconType::RGBA32_32x32_2x,
                                            IconType::RGBA32_64x64,
                                            IconType::RGBA32_128x128,
                                            IconType::RGBA32_128x128_2x,
                                            IconType::RGBA32_256x256,
                                            IconType::RGBA32_256x256_2x,
                                            IconType::RGBA32_512x512,
                                            IconType::RGBA32_512x512_2x];

    #[test]
    fn icon_type_ostype_round_trip() {
        for icon_type in &ALL_ICON_TYPES {
            let ostype = icon_type.ostype();
            let from = IconType::from_ostype(ostype);
            assert_eq!(Some(*icon_type), from);
        }
    }

    #[test]
    fn icon_type_mask_type() {
        for icon_type in &ALL_ICON_TYPES {
            match icon_type.encoding() {
                Encoding::Mask8 => {
                    assert!(icon_type.is_mask());
                    assert_eq!(icon_type.mask_type(), None);
                }
                Encoding::RLE24 => {
                    assert!(!icon_type.is_mask());
                    if let Some(mask_type) = icon_type.mask_type() {
                        assert_eq!(mask_type.encoding(), Encoding::Mask8);
                        assert_eq!(icon_type.pixel_width(),
                                   mask_type.pixel_width());
                        assert_eq!(icon_type.pixel_height(),
                                   mask_type.pixel_height());
                    } else {
                        panic!("{:?} is missing a mask type", icon_type);
                    }
                }
                Encoding::JP2PNG => {
                    assert!(!icon_type.is_mask());
                    assert_eq!(icon_type.mask_type(), None);
                }
            }
        }
    }

    #[test]
    fn ostype_to_and_from_str() {
        let ostype = OSType::from_str("abcd").expect("failed to parse OSType");
        assert_eq!(ostype.to_string(), "abcd".to_string());
    }

    #[test]
    fn ostype_to_and_from_str_non_ascii() {
        let ostype = OSType(*b"sp\xf6b");
        let string = ostype.to_string();
        assert_eq!(string, "sp\u{f6}b".to_string());
        assert_eq!(OSType::from_str(&string), Ok(ostype));
    }

    #[test]
    fn ostype_from_str_failure() {
        assert_eq!(OSType::from_str("abc"),
                   Err("OSType string must be 4 chars (was 3)".to_string()));
        assert_eq!(OSType::from_str("abcde"),
                   Err("OSType string must be 4 chars (was 5)".to_string()));
        assert_eq!(OSType::from_str("ab\u{2603}d"),
                   Err("OSType chars must have value of at most 0xFF \
                        (found 0x2603)"
                           .to_string()));
    }
}
