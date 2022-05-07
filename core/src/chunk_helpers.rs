use crate::errors;

pub const PNG_HEADER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

/// Color type is a single-byte integer that describes the interpretation of the image data.
/// Color type codes represent sums of the following values: 1 (palette used), 2 (color used), and 4 (alpha channel used).
/// Valid values are 0, 2, 3, 4, and 6.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ColorType {
    Greyscale = 0,
    Truecolor = 2,
    /// Palette-based color
    IndexedColor = 3,
    GreyscaleAlpha = 4,
    TruecolorAlpha = 6,
}

impl TryFrom<u8> for ColorType {
    type Error = errors::PngDecodeErrorCode;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == ColorType::Greyscale as u8 => Ok(ColorType::Greyscale),
            x if x == ColorType::Truecolor as u8 => Ok(ColorType::Truecolor),
            x if x == ColorType::IndexedColor as u8 => Ok(ColorType::IndexedColor),
            x if x == ColorType::GreyscaleAlpha as u8 => Ok(ColorType::GreyscaleAlpha),
            x if x == ColorType::TruecolorAlpha as u8 => Ok(ColorType::TruecolorAlpha),
            _ => Err(errors::PngDecodeErrorCode::_18(
                v,
            )),
        }
    }
}

///  Color    Allowed    Interpretation
///
///  Type    Bit Depths
///
///  0       1,2,4,8,16  Each pixel is a grayscale sample.
///
///  2       8,16        Each pixel is an R,G,B triple.
///
///  3       1,2,4,8     Each pixel is a palette index;
///                      a PLTE chunk must appear.  
///                      1, 2, 4, and 8 bits correspond to a maximum of 2, 4, 16, or 256 palette entries.
///
///  4       8,16        Each pixel is a grayscale sample,
///                      followed by an alpha sample.
///
///  6       8,16        Each pixel is an R,G,B triple,
///                      followed by an alpha sample.
pub fn get_supported_color_type_to_bit_depths(color_type: ColorType) -> Vec<u8> {
    match color_type {
        ColorType::Greyscale => vec![1, 2, 4, 8, 16],
        ColorType::Truecolor => vec![8, 16],
        ColorType::IndexedColor => vec![1, 2, 4, 8],
        ColorType::GreyscaleAlpha => vec![8, 16],
        ColorType::TruecolorAlpha => vec![8, 16],
    }
}

/// https://en.wikipedia.org/wiki/Portable_Network_Graphics#Pixel_format
pub fn colortype_to_channel(color_type: ColorType) -> u8 {
    match color_type {
        ColorType::Greyscale => 1,
        ColorType::Truecolor => 3,
        ColorType::IndexedColor => 1,
        ColorType::GreyscaleAlpha => 2,
        ColorType::TruecolorAlpha => 4,
    }
}

pub fn colortype_to_alpha_byte_length(color_type: ColorType) -> u8 {
    match color_type {
        ColorType::Greyscale => 2,
        ColorType::Truecolor => 6,
        // The tRNS chunk shall not contain more alpha values than there are palette entries, but a tRNS chunk may contain fewer values than there are palette entries.
        // ColorType::IndexedColor => 3,
        // tRNS must not exist for ColorType::GreyscaleAlpha and TruecolorAlpha
        _ => 0,
    }
}

pub fn does_colortype_support_alpha_channel(color_type: ColorType) -> bool {
    return ColorType::GreyscaleAlpha == color_type || ColorType::TruecolorAlpha == color_type;
}

/// Compression method is a single-byte integer that indicates the method used to compress
/// the image data. At present, only compression method 0 (deflate/inflate compression with a
/// sliding window of at most 32768 bytes) is defined.
#[derive(Debug, Clone)]
pub enum CompressionMethod {
    Deflate,
}

impl TryFrom<u8> for CompressionMethod {
    // todo error
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == CompressionMethod::Deflate as u8 => Ok(CompressionMethod::Deflate),
            _ => Err(()),
        }
    }
}

/// Filter method is a single-byte integer that indicates the preprocessing method applied
/// to the image data before compression. At present, only filter method 0
/// (adaptive filtering with five basic filter types) is defined.
#[derive(Debug, Clone)]
pub enum FilterMethod {
    Adaptive,
}

impl TryFrom<u8> for FilterMethod {
    // todo error
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == FilterMethod::Adaptive as u8 => Ok(FilterMethod::Adaptive),
            _ => Err(()),
        }
    }
}

/// Interlace method is a single-byte integer that indicates the transmission order of the
/// image data. Two values are currently defined: 0 (no interlace) or 1 (Adam7 interlace).
/// See Interlaced data order for details.
#[derive(Clone, Copy, Debug)]
pub enum InterlaceMethod {
    None,
    Adam7,
}

impl TryFrom<u8> for InterlaceMethod {
    type Error = errors::PngDecodeErrorCode;
    fn try_from(v: u8) -> Result<Self, errors::PngDecodeErrorCode> {
        match v {
            x if x == InterlaceMethod::None as u8 => Ok(InterlaceMethod::None),
            x if x == InterlaceMethod::Adam7 as u8 => Ok(InterlaceMethod::Adam7),
            _ => Err(errors::PngDecodeErrorCode::_12(v)),
        }
    }
}
