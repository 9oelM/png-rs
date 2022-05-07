use crate::errors;

pub fn calc_bytes_per_pixel_and_line(
    channel: u8,
    bit_depth: u8,
    image_pixel_width: u32,
) -> (usize, usize) {
    let bytes_per_line = ((image_pixel_width as u64 * bit_depth as u64 * channel as u64) + 7) / 8;
    let bytes_per_pixel = (((channel * bit_depth) + 7) / 8) as usize;

    return (
        bytes_per_pixel,
        bytes_per_line as usize,
    );
}

const U8_MAX_OUT_SAMPLE: f32 = 255.0;
// 2**16 - 1
const U16_MAX_IN_SAMPLE: f32 = 65535.0;

pub(crate) fn normalize_u16_to_u8(
    num: u16
) -> Result<u8, errors::PngDecodeErrorCode> {
    let normalized_u8 = ((num as f32 * U8_MAX_OUT_SAMPLE) / U16_MAX_IN_SAMPLE + 0.5).floor();
    if normalized_u8 > U8_MAX_OUT_SAMPLE {
        return Err(errors::PngDecodeErrorCode::_24("u16".to_string(), "u8".to_string()))
    }

    // now safe
    return Ok(normalized_u8 as u8)
}