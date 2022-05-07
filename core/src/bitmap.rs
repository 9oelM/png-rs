/// ##########################################
/// Initially adopted codes from https://github.com/bschwind/png-decoder/blob/master/src/lib.rs
/// ##########################################
use crate::{
    chunk_helpers,
    errors::{self},
    trns::TransparencyChunk,
};

#[derive(Debug, Copy, Clone)]
pub enum PixelType {
    Grayscale1,
    Grayscale2,
    Grayscale4,
    Grayscale8,
    Grayscale16,

    Rgb8,
    Rgb16,

    Palette1,
    Palette2,
    Palette4,
    Palette8,

    GrayscaleAlpha8,
    GrayscaleAlpha16,

    RgbAlpha8,
    RgbAlpha16,
}

impl PixelType {
    pub fn new(
        color_type: chunk_helpers::ColorType,
        bit_depth: u8,
    ) -> Result<Self, errors::PngDecodeErrorCode> {

        let result = match color_type {
            chunk_helpers::ColorType::Greyscale => match bit_depth {
                1 => PixelType::Grayscale1,
                2 => PixelType::Grayscale2,
                4 => PixelType::Grayscale4,
                8 => PixelType::Grayscale8,
                16 => PixelType::Grayscale16,
                _ => return Err(errors::PngDecodeErrorCode::_22(color_type, bit_depth)),
            },
            chunk_helpers::ColorType::Truecolor => match bit_depth {
                8 => PixelType::Rgb8,
                16 => PixelType::Rgb16,
                _ => return Err(errors::PngDecodeErrorCode::_22(color_type, bit_depth)),
            },
            chunk_helpers::ColorType::IndexedColor => match bit_depth {
                1 => PixelType::Palette1,
                2 => PixelType::Palette2,
                4 => PixelType::Palette4,
                8 => PixelType::Palette8,
                _ => return Err(errors::PngDecodeErrorCode::_22(color_type, bit_depth)),
            },
            chunk_helpers::ColorType::GreyscaleAlpha => match bit_depth {
                8 => PixelType::GrayscaleAlpha8,
                16 => PixelType::GrayscaleAlpha16,
                _ => return Err(errors::PngDecodeErrorCode::_22(color_type, bit_depth)),
            },
            chunk_helpers::ColorType::TruecolorAlpha => match bit_depth {
                8 => PixelType::RgbAlpha8,
                16 => PixelType::RgbAlpha16,
                _ => return Err(errors::PngDecodeErrorCode::_22(color_type, bit_depth)),
            },
        };

        Ok(result)
    }
}

const U8_MAX_OUT_SAMPLE: f32 = 255.0;
// 2**16 - 1
const U16_MAX_IN_SAMPLE: f32 = 65535.0;

fn normalize_u16_to_u8(
    num: u16
) -> Result<u8, errors::PngDecodeErrorCode> {
    let normalized_u8 = ((num as f32 * U8_MAX_OUT_SAMPLE) / U16_MAX_IN_SAMPLE + 0.5).floor();
    if normalized_u8 > U8_MAX_OUT_SAMPLE {
        return Err(errors::PngDecodeErrorCode::_24("u16".to_string(), "u8".to_string()))
    }

    // now safe
    return Ok(normalized_u8 as u8)
}

pub fn to_rgba_pixel_bytes(
    pixel_type: PixelType,
    transparency_chunk: Option<&TransparencyChunk>,
    palette_chunk: Option<&Vec<u8>>,
    pixel_start_byte_position: usize,
    unfiltered_data: &[u8],
) -> Result<(u8, u8, u8, u8), errors::PngDecodeErrorCode> {
    let pixel = match pixel_type {
        PixelType::Grayscale1 => {
            let byte = unfiltered_data[pixel_start_byte_position / 8];
            let bit_offset = 7 - pixel_start_byte_position % 8;
            let grayscale_val = (byte >> bit_offset) & 1;

            let alpha = match transparency_chunk {
                Some(TransparencyChunk::Grayscale(transparent_val))
                    if grayscale_val == *transparent_val =>
                {
                    0
                }
                _ => 255,
            };

            let pixel_val = grayscale_val * 255;

            (
                pixel_val, pixel_val, pixel_val, alpha,
            )
        }
        PixelType::Grayscale2 => {
            let byte = unfiltered_data[pixel_start_byte_position / 4];
            let bit_offset = 6 - ((pixel_start_byte_position % 4) * 2);
            let grayscale_val = (byte >> bit_offset) & 0b11;

            let alpha = match transparency_chunk {
                Some(TransparencyChunk::Grayscale(transparent_val))
                    if grayscale_val == *transparent_val =>
                {
                    0
                }
                _ => 255,
            };

            // TODO - use a lookup table
            let pixel_val = ((grayscale_val as f32 / 3.0) * 255.0) as u8;

            (
                pixel_val, pixel_val, pixel_val, alpha,
            )
        }
        PixelType::Grayscale4 => {
            let byte = unfiltered_data[pixel_start_byte_position / 2];
            let bit_offset = 4 - ((pixel_start_byte_position % 2) * 4);
            let grayscale_val = (byte >> bit_offset) & 0b1111;

            let alpha = match transparency_chunk {
                Some(TransparencyChunk::Grayscale(transparent_val))
                    if grayscale_val == *transparent_val =>
                {
                    0
                }
                _ => 255,
            };

            // TODO - use a lookup table
            let pixel_val = ((grayscale_val as f32 / 15.0) * 255.0) as u8;
            (
                pixel_val, pixel_val, pixel_val, alpha,
            )
        }
        PixelType::Grayscale8 => {
            let byte = unfiltered_data[pixel_start_byte_position];

            let alpha = match transparency_chunk {
                Some(TransparencyChunk::Grayscale(transparent_val)) if byte == *transparent_val => {
                    0
                }
                _ => 255,
            };
            (byte, byte, byte, alpha)
        }
        PixelType::Grayscale16 => {
            let offset = pixel_start_byte_position * 2;
            let pixel_val =
                normalize_u16_to_u8(u16::from_be_bytes([unfiltered_data[offset], unfiltered_data[offset + 1]]))?; 

            let alpha = match transparency_chunk {
                Some(TransparencyChunk::Grayscale(transparent_val))
                    if pixel_val == *transparent_val =>
                {
                    0
                }
                _ => 255,
            };

            (
                pixel_val, pixel_val, pixel_val, alpha,
            )
        }
        PixelType::Rgb8 => {
            let offset = pixel_start_byte_position * 3;
            let r = unfiltered_data[offset];
            let g = unfiltered_data[offset + 1];
            let b = unfiltered_data[offset + 2];

            let alpha = match transparency_chunk {
                Some(TransparencyChunk::Rgb(t_r, t_g, t_b))
                    if r == *t_r && g == *t_g && b == *t_b =>
                {
                    0
                }
                _ => 255,
            };

            (r, g, b, alpha)
        }
        PixelType::Rgb16 => {
            let offset = pixel_start_byte_position * 6;
            let r = normalize_u16_to_u8(u16::from_be_bytes([unfiltered_data[offset], unfiltered_data[offset + 1]]))?;
            let g = normalize_u16_to_u8(u16::from_be_bytes([unfiltered_data[offset + 2], unfiltered_data[offset + 3]]))?;
            let b = normalize_u16_to_u8(u16::from_be_bytes([unfiltered_data[offset + 4], unfiltered_data[offset + 5]]))?;

            let alpha = match transparency_chunk {
                Some(TransparencyChunk::Rgb(t_r, t_g, t_b))
                    if r == *t_r && g == *t_g && b == *t_b =>
                {
                    0
                }
                _ => 255,
            };

            (r, g, b, alpha)
        }
        PixelType::Palette1 => {
            let byte = unfiltered_data[pixel_start_byte_position / 8];
            let bit_offset = 7 - pixel_start_byte_position % 8;
            let palette_idx = ((byte >> bit_offset) & 1) as usize;

            let offset = palette_idx * 3;

            let palette = palette_chunk.unwrap();
            let r = palette[offset];
            let g = palette[offset + 1];
            let b = palette[offset + 2];

            let alpha: u8 = match transparency_chunk {
                Some(TransparencyChunk::Palette(data)) => *data.get(palette_idx).unwrap_or(&255),
                Some(_) | None => 255,
            };

            (r, g, b, alpha)
        }
        PixelType::Palette2 => {
            let byte = unfiltered_data[pixel_start_byte_position / 4];
            let bit_offset = 6 - ((pixel_start_byte_position % 4) * 2);
            let palette_idx = ((byte >> bit_offset) & 0b11) as usize;

            let offset = palette_idx * 3;

            let palette = palette_chunk.unwrap();
            let r = palette[offset];
            let g = palette[offset + 1];
            let b = palette[offset + 2];

            let alpha: u8 = match transparency_chunk {
                Some(TransparencyChunk::Palette(data)) => *data.get(palette_idx).unwrap_or(&255),
                Some(_) | None => 255,
            };

            (r, g, b, alpha)
        }
        PixelType::Palette4 => {
            let byte = unfiltered_data[pixel_start_byte_position / 2];
            let bit_offset = 4 - ((pixel_start_byte_position % 2) * 4);
            let palette_idx = ((byte >> bit_offset) & 0b1111) as usize;

            let offset = palette_idx * 3;

            let palette = palette_chunk.unwrap();
            let r = palette[offset];
            let g = palette[offset + 1];
            let b = palette[offset + 2];

            let alpha: u8 = match transparency_chunk {
                Some(TransparencyChunk::Palette(data)) => *data.get(palette_idx).unwrap_or(&255),
                Some(_) | None => 255,
            };

            (r, g, b, alpha)
        }
        PixelType::Palette8 => {
            let offset = unfiltered_data[pixel_start_byte_position] as usize * 3;

            let palette = palette_chunk.unwrap();
            let r = palette[offset];
            let g = palette[offset + 1];
            let b = palette[offset + 2];

            let alpha: u8 = match transparency_chunk {
                Some(TransparencyChunk::Palette(data)) => *data.get(offset).unwrap_or(&255),
                Some(_) | None => 255,
            };

            (r, g, b, alpha)
        }
        PixelType::GrayscaleAlpha8 => {
            let offset = pixel_start_byte_position * 2;
            let grayscale_val = unfiltered_data[offset];
            let alpha = unfiltered_data[offset + 1];

            (
                grayscale_val,
                grayscale_val,
                grayscale_val,
                alpha,
            )
        }
        PixelType::GrayscaleAlpha16 => {
            let offset = pixel_start_byte_position * 4;
            let grayscale_val =
            normalize_u16_to_u8(u16::from_be_bytes([unfiltered_data[offset], unfiltered_data[offset + 1]]))?;
            let alpha =
            normalize_u16_to_u8(u16::from_be_bytes([unfiltered_data[offset + 2], unfiltered_data[offset + 3]]))?;

            (
                grayscale_val,
                grayscale_val,
                grayscale_val,
                alpha,
            )
        }
        PixelType::RgbAlpha8 => {
            let offset = pixel_start_byte_position * 4;
            let r = unfiltered_data[offset];
            let g = unfiltered_data[offset + 1];
            let b = unfiltered_data[offset + 2];
            let a = unfiltered_data[offset + 3];

            (r, g, b, a)
        }
        PixelType::RgbAlpha16 => {
            let offset = pixel_start_byte_position * 8;
            let r = normalize_u16_to_u8(u16::from_be_bytes([unfiltered_data[offset], unfiltered_data[offset + 1]]))?;
            let g = normalize_u16_to_u8(u16::from_be_bytes([unfiltered_data[offset + 2], unfiltered_data[offset + 3]]))?;
            let b = normalize_u16_to_u8(u16::from_be_bytes([unfiltered_data[offset + 4], unfiltered_data[offset + 5]]))?;
            let a = normalize_u16_to_u8(u16::from_be_bytes([unfiltered_data[offset + 6], unfiltered_data[offset + 7]]))?;

            (r, g, b, a)
        }
    };

    return Ok(pixel);
}
