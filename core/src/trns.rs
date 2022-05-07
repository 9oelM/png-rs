use crate::{bitmap, common::{normalize_u16_to_u8}, errors};

#[derive(Clone)]
pub enum TransparencyChunk {
    Palette(Vec<u8>),
    Grayscale(u8),
    Rgb(u8, u8, u8),
}

impl TransparencyChunk {
    pub fn new(chunk: Vec<u8>, pixel_type: bitmap::PixelType) -> Result<Option<Self>, errors::PngDecodeErrorCode> {
        match pixel_type {
            bitmap::PixelType::Grayscale1 => Ok(Some(TransparencyChunk::Grayscale(
                chunk[1] & 0b1,
            ))),
            bitmap::PixelType::Grayscale2 => Ok(Some(TransparencyChunk::Grayscale(
                chunk[1] & 0b11,
            ))),
            bitmap::PixelType::Grayscale4 => Ok(Some(TransparencyChunk::Grayscale(
                chunk[1] & 0b1111,
            ))),
            bitmap::PixelType::Grayscale8 => Ok(Some(TransparencyChunk::Grayscale(
                chunk[1],
            ))),
            bitmap::PixelType::Grayscale16 => {
                let val = normalize_u16_to_u8(u16::from_be_bytes([chunk[0], chunk[1]]))?;
                Ok(Some(TransparencyChunk::Grayscale(
                    val,
                )))
            }
            bitmap::PixelType::Rgb8 => {
                let r = chunk[1];
                let g = chunk[3];
                let b = chunk[5];
                Ok(Some(TransparencyChunk::Rgb(
                    r, g, b,
                )))
            }
            bitmap::PixelType::Rgb16 => {
                let r =  normalize_u16_to_u8(u16::from_be_bytes([chunk[0], chunk[1]]))?;
                let g = normalize_u16_to_u8(u16::from_be_bytes([chunk[2], chunk[3]]))?;
                let b = normalize_u16_to_u8(u16::from_be_bytes([chunk[4], chunk[5]]))?;
                Ok(Some(TransparencyChunk::Rgb(
                    r,
                    g,
                    b,
                )))
            }
            bitmap::PixelType::Palette1 => Ok(Some(TransparencyChunk::Palette(
                chunk,
            ))),
            bitmap::PixelType::Palette2 => Ok(Some(TransparencyChunk::Palette(
                chunk,
            ))),
            bitmap::PixelType::Palette4 => Ok(Some(TransparencyChunk::Palette(
                chunk,
            ))),
            bitmap::PixelType::Palette8 => Ok(Some(TransparencyChunk::Palette(
                chunk,
            ))),
            bitmap::PixelType::GrayscaleAlpha8 => Ok(None),
            bitmap::PixelType::GrayscaleAlpha16 => Ok(None),
            bitmap::PixelType::RgbAlpha8 => Ok(None),
            bitmap::PixelType::RgbAlpha16 => Ok(None),
        }
    }
}
