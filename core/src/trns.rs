use crate::{bitmap, common::u16_to_u8};

#[derive(Clone)]
pub enum TransparencyChunk {
    Palette(Vec<u8>),
    Grayscale(u8),
    Rgb(u8, u8, u8),
}

impl TransparencyChunk {
    pub fn new(chunk: Vec<u8>, pixel_type: bitmap::PixelType) -> Option<Self> {
        match pixel_type {
            bitmap::PixelType::Grayscale1 => Some(TransparencyChunk::Grayscale(
                chunk[1] & 0b1,
            )),
            bitmap::PixelType::Grayscale2 => Some(TransparencyChunk::Grayscale(
                chunk[1] & 0b11,
            )),
            bitmap::PixelType::Grayscale4 => Some(TransparencyChunk::Grayscale(
                chunk[1] & 0b1111,
            )),
            bitmap::PixelType::Grayscale8 => Some(TransparencyChunk::Grayscale(
                chunk[1],
            )),
            bitmap::PixelType::Grayscale16 => {
                let val = u16::from_be_bytes([chunk[0], chunk[1]]);
                Some(TransparencyChunk::Grayscale(
                    u16_to_u8(val),
                ))
            }
            bitmap::PixelType::Rgb8 => {
                let r = chunk[1];
                let g = chunk[3];
                let b = chunk[5];
                Some(TransparencyChunk::Rgb(
                    r, g, b,
                ))
            }
            bitmap::PixelType::Rgb16 => {
                let r = u16::from_be_bytes([chunk[0], chunk[1]]);
                let g = u16::from_be_bytes([chunk[2], chunk[3]]);
                let b = u16::from_be_bytes([chunk[4], chunk[5]]);
                Some(TransparencyChunk::Rgb(
                    u16_to_u8(r),
                    u16_to_u8(g),
                    u16_to_u8(b),
                ))
            }
            bitmap::PixelType::Palette1 => Some(TransparencyChunk::Palette(
                chunk,
            )),
            bitmap::PixelType::Palette2 => Some(TransparencyChunk::Palette(
                chunk,
            )),
            bitmap::PixelType::Palette4 => Some(TransparencyChunk::Palette(
                chunk,
            )),
            bitmap::PixelType::Palette8 => Some(TransparencyChunk::Palette(
                chunk,
            )),
            bitmap::PixelType::GrayscaleAlpha8 => None,
            bitmap::PixelType::GrayscaleAlpha16 => None,
            bitmap::PixelType::RgbAlpha8 => None,
            bitmap::PixelType::RgbAlpha16 => None,
        }
    }
}
