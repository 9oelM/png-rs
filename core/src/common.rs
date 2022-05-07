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

#[inline(always)]
pub fn u16_to_u8(val: u16) -> u8 {
    (val >> 8) as u8
}
