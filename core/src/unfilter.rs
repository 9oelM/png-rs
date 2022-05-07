//! Type    Name
//!
//! 0       None
//!
//! 1       Sub
//!
//! 2       Up
//!
//! 3       Average
//!
//! 4       Paeth
//!
//! The first filter leaves the original data intact, and the other four are subtracting from each pixel a value that involves the neighbor pixels from the left, up, and/or the upper left.
//!
//! For example, a 'left' filter would transform a sequence of `2, 3, 4, 5, 6, 7, 8, 9` to `2, 1, 1, 1, 1, 1, 1, 1`. As described by the libpng docs, it is a 'precompression step' because it transforms the data so that it can be compressed more efficiently.

use crate::errors;

/// Variable names in unfilter processor are as elaborate as possible to avoid confusion. Instead, lines have become a bit longer.
/// However short variable names often cause confusion especially in detailed bytewise ops, so let's keep it this way
pub struct UnfilterProcessor {
    height: u32,
    /// 6 bytes per pixel (48-bit RGB, 16-bit mode image type) is the maximum
    /// however calculation with usize integers is often needed, so set the size as `usize`
    bytes_per_pixel: usize,
    /// How many bytes are there per line (`bytes_per_line * height = entire image bytes`).
    bytes_per_line: usize,
}

impl UnfilterProcessor {
    pub fn new(height: u32, bytes_per_pixel: usize, bytes_per_line: usize) -> Self {
        Self {
            height,
            bytes_per_pixel,
            bytes_per_line,
        }
    }

    fn get_out_buffer_index(&self, line_number: usize, nth_byte_in_line: usize) -> usize {
        return line_number * self.bytes_per_line + nth_byte_in_line;
    }

    fn copy_first_pixel_bytes_in_current_line(
        &mut self,
        out_buffer_start_index: usize,
        out_buffer: &mut [u8],
        in_buffer_start_index: usize,
        in_buffer: &[u8],
    ) -> (usize, usize) {
        for i in 0..self.bytes_per_pixel {
            out_buffer[out_buffer_start_index + i] = in_buffer[in_buffer_start_index + i];
        }

        return (
            out_buffer_start_index + self.bytes_per_pixel,
            in_buffer_start_index + self.bytes_per_pixel,
        );
    }

    /// Filter type 0. Basically does nothing other than copying the decompressed output from zlib to the output buffer.
    fn unfilter_none(
        &mut self,
        out_buffer_start_index: usize,
        out_buffer: &mut [u8],
        in_buffer_start_index: usize,
        in_buffer: &[u8],
    ) {
        for i in 0..self.bytes_per_line {
            out_buffer[out_buffer_start_index + i] = in_buffer[in_buffer_start_index + i];
        }
    }
    /// Filter type 1. The first pixel bytes in the scanline are just copied over to the output buffer, just like filter type 0.
    /// For the following bytes after the first pixel bytes, the byte immediately left by one pixel is added to the position of the filtered data.
    /// For example, if:
    /// ```ignore
    /// out_buffer_start_index = 5
    /// bytes_per_pixel = 2
    ///
    /// current_scanline (not yet unfiltered) =
    /// index:  0 1 2 3 4 5 6 7 8 ...
    ///     -------------------------------
    ///     ...|5|5|5|4|3|2|5|6|7|8|5|2|5|
    ///     -------------------------------
    ///
    /// unfiltered_output =
    /// index:  0 1 2 3 4 5 6 7 8 ...
    ///     -------------------------------
    ///     ...|5|5|5|4|3| | | | | | | | |
    /// ```
    ///
    /// then
    ///
    /// ```ignore
    /// unfiltered_output = current_scanline[5] + unfiltered_output[5-2]
    /// = 2 + 4
    /// ```
    /// Also, note that unsigned arithmetic modulo 256 is used,
    /// so that both the inputs and outputs fit into bytes.
    /// this is just another way of saying if you have an overflow in `u8`, you will wrap it around with (a + b) % 256 (`.wrapping_add` in the code), so that you always stay in that 8 bits.
    ///  
    fn unfilter_sub(
        &mut self,
        out_buffer_start_index: usize,
        out_buffer: &mut [u8],
        in_buffer_start_index: usize,
        in_buffer: &[u8],
    ) {
        // first, handle bytes corresponding to the first pixel
        let (out_buffer_start_index, in_buffer_start_index) = self
            .copy_first_pixel_bytes_in_current_line(
                out_buffer_start_index,
                out_buffer,
                in_buffer_start_index,
                in_buffer,
            );

        // next, handle the rest of the bytes in the scanline
        for i in 0..self.bytes_per_line - self.bytes_per_pixel {
            let out_buffer_immediate_pixel_left_index =
                out_buffer_start_index + i - self.bytes_per_pixel;
            out_buffer[out_buffer_start_index + i] = in_buffer[in_buffer_start_index + i]
                .wrapping_add(out_buffer[out_buffer_immediate_pixel_left_index]);
        }
    }

    /// Filter type 2.
    ///
    /// If it is the first scanline, simply copy each byte over. If not, apply the below pseudocode.
    /// ```ignore
    /// unfiltered_output[i] =
    ///     (current_scanline[i] +
    ///     // the byte that was unfiltered already, directly above the current byte's  
    ///     unfiltered_output[i - bytes_per_line])
    ///     // modulo 256
    ///     % 256
    /// ```
    fn unfilter_up(
        &mut self,
        out_buffer_start_index: usize,
        out_buffer: &mut [u8],
        in_buffer_start_index: usize,
        in_buffer: &[u8],
    ) {
        // first scanline
        if out_buffer_start_index == 0 {
            self.unfilter_none(
                out_buffer_start_index,
                out_buffer,
                in_buffer_start_index,
                in_buffer,
            );
            return;
        }
        // following scanlines
        for i in 0..self.bytes_per_line {
            out_buffer[out_buffer_start_index + i] = in_buffer[in_buffer_start_index + i]
                .wrapping_add(out_buffer[out_buffer_start_index + i - self.bytes_per_line]);
        }
    }
    fn unfilter_avg(
        &mut self,
        out_buffer_start_index: usize,
        out_buffer: &mut [u8],
        in_buffer_start_index: usize,
        in_buffer: &[u8],
    ) {
        // first scanline
        if out_buffer_start_index == 0 {
            let (out_buffer_start_index, in_buffer_start_index) = self
                .copy_first_pixel_bytes_in_current_line(
                    out_buffer_start_index,
                    out_buffer,
                    in_buffer_start_index,
                    in_buffer,
                );

            for i in 0..self.bytes_per_line - self.bytes_per_pixel {
                out_buffer[out_buffer_start_index + i] = in_buffer[in_buffer_start_index + i]
                    .wrapping_add(
                        // floor() indicates that the result of the division is rounded to the next lower integer if fractional; in other words, it is an integer division or right shift operation.
                        // works the same for any other right shift operations seen in other filters
                        out_buffer[out_buffer_start_index + i - self.bytes_per_pixel] >> 1,
                    );
            }

            return;
        }

        // following scanlines
        for i in 0..self.bytes_per_pixel {
            out_buffer[out_buffer_start_index + i] = in_buffer[in_buffer_start_index + i]
                .wrapping_add(out_buffer[out_buffer_start_index + i - self.bytes_per_line] >> 1);
        }

        for i in self.bytes_per_pixel..self.bytes_per_line {
            let rhs = ((out_buffer[out_buffer_start_index + i - self.bytes_per_pixel] as u16
                + out_buffer[out_buffer_start_index + i - self.bytes_per_line] as u16)
                >> 1) as u8;
            out_buffer[out_buffer_start_index + i] =
                in_buffer[in_buffer_start_index + i].wrapping_add(rhs);
        }
    }
    /// | up_left pixel | up pixel |
    /// |-|-|
    /// | left pixel | current pixel |
    fn paeth_predictor(&self, left: i16, up: i16, up_left: i16) -> u8 {
        let paeth = left + up - up_left;
        let position_left = ((paeth - left) as f32).abs() as u8;
        let position_up = ((paeth - up) as f32).abs() as u8;
        let position_up_left = ((paeth - up_left) as f32).abs() as u8;

        if position_left <= position_up && position_left <= position_up_left {
            return left as u8;
        }
        if position_up <= position_up_left {
            return up as u8;
        }
        return up_left as u8;
    }

    fn unfilter_paeth(
        &mut self,
        out_buffer_start_index: usize,
        out_buffer: &mut [u8],
        in_buffer_start_index: usize,
        in_buffer: &[u8],
    ) {
        if out_buffer_start_index == 0 {
            self.unfilter_sub(
                out_buffer_start_index,
                out_buffer,
                in_buffer_start_index,
                in_buffer,
            );
            return;
        }

        for i in 0..self.bytes_per_pixel {
            out_buffer[out_buffer_start_index + i] = in_buffer[in_buffer_start_index + i]
                .wrapping_add(out_buffer[out_buffer_start_index + i - self.bytes_per_line]);
        }
        for i in self.bytes_per_pixel..self.bytes_per_line {
            let left = out_buffer[out_buffer_start_index + i - self.bytes_per_pixel] as i16;
            let up = out_buffer[out_buffer_start_index + i - self.bytes_per_line] as i16;
            let up_left = out_buffer
                [out_buffer_start_index + i - self.bytes_per_line - self.bytes_per_pixel]
                as i16;

            out_buffer[out_buffer_start_index + i] = in_buffer[in_buffer_start_index + i]
                .wrapping_add(self.paeth_predictor(left, up, up_left));
        }
    }

    /// unfilters scanlines with possibly varying filter types.
    /// * `in_buffer` - the bytes decompressed by zlib
    pub fn unfilter(&mut self, in_buffer: &[u8], out_buffer: &mut [u8]) -> Result<(), errors::PngDecodeErrorCode> {
        let mut filter_byte_index: usize = 0;

        for line_number in 0..self.height.try_into().expect("Height doesn't fit in usize") {
            let filter_type = in_buffer[filter_byte_index];
            let out_buffer_start_index = self.get_out_buffer_index(line_number, 0);
            let in_buffer_start_index = filter_byte_index + 1;

            match filter_type {
                0 => self.unfilter_none(
                    out_buffer_start_index,
                    out_buffer,
                    in_buffer_start_index,
                    in_buffer,
                ),
                1 => self.unfilter_sub(
                    out_buffer_start_index,
                    out_buffer,
                    in_buffer_start_index,
                    in_buffer,
                ),
                2 => self.unfilter_up(
                    out_buffer_start_index,
                    out_buffer,
                    in_buffer_start_index,
                    in_buffer,
                ),
                3 => self.unfilter_avg(
                    out_buffer_start_index,
                    out_buffer,
                    in_buffer_start_index,
                    in_buffer,
                ),
                4 => self.unfilter_paeth(
                    out_buffer_start_index,
                    out_buffer,
                    in_buffer_start_index,
                    in_buffer,
                ),
                _ => {
                    return Err(errors::PngDecodeErrorCode::_17(
                        filter_type,
                    ))
                }
            }
            filter_byte_index += self.bytes_per_line + 1;
        }

        Ok(())
    }
}
