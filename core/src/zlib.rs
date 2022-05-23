use std::vec::Vec;

use miniz_oxide::inflate::{
    decompress_to_vec_zlib,
    core::{
        decompress,
        inflate_flags::{self},
        DecompressorOxide,
    },
    TINFLStatus,
};

use crate::errors::{PngDecodeError, PngDecodeErrorCode};

/// Continuously receive image data, decompress it, and append the result to the output buffer
pub struct ZlibDecompressStream {
    // Vector filled with zeros, twice the size of default buffer size
    // it will need to grow as the program discovers image data of greater size
    out_buffer: Vec<u8>,
    // This keeps track of the latest cursor position on the output buffer
    out_buffer_byte_pos: usize,
    // Stores the state of miniz_oxide's decompress function
    decompressor_state: Box<DecompressorOxide>,
}

const DEFAULT_ZLIB_STREAM_BUFFER_SIZE: usize = 1 * 1024;

const BASE_FLAGS: u32 = inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER
    | inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF
    | inflate_flags::TINFL_FLAG_HAS_MORE_INPUT;

impl ZlibDecompressStream {
    pub fn new(buffer_size: Option<usize>) -> ZlibDecompressStream {
        let effective_buffer_size = buffer_size.unwrap_or(DEFAULT_ZLIB_STREAM_BUFFER_SIZE);

        ZlibDecompressStream {
            out_buffer: vec![0; 2 * effective_buffer_size],
            decompressor_state: Box::new(DecompressorOxide::new()),
            out_buffer_byte_pos: 0,
        }
    }

    /// `self.out_buffer` size is increased to avoid index out of range when `self.out_buffer_byte_pos` * 2 is greater
    fn resize_out_buffer_if_needed(&mut self) {
        if self.out_buffer.len() <= self.out_buffer_byte_pos * 2 {
            self.out_buffer.resize(self.out_buffer.len() * 2, 0u8);
        }
    }

    pub fn get_out_buffer(&self) -> &Vec<u8> {
        &self.out_buffer
    }

    /// Decompresses image bytes as they come in.
    /// * `raw_image_bytes` - this is the vector of u8 image data from an IDAT chunk. Favorably should be possible to receive more than a single IDAT chunk or a part of an IDAT chunk because the size of an IDAT chunk varies greatly. But for now we are just sticking to a single IDAT chunk.
    pub fn decompress(&mut self, raw_image_bytes: &Vec<u8>) -> Result<(), PngDecodeErrorCode> {
        let mut in_buffer_byte_pos: usize = 0;
        while in_buffer_byte_pos < raw_image_bytes.len() {
            self.resize_out_buffer_if_needed();
            #[allow(non_snake_case)]
            let (current_TINFL_status, num_bytes_read, num_bytes_written) = decompress(
                &mut self.decompressor_state,
                &raw_image_bytes[in_buffer_byte_pos..],
                &mut self.out_buffer[..],
                self.out_buffer_byte_pos,
                BASE_FLAGS,
            );

            in_buffer_byte_pos += num_bytes_read;
            self.out_buffer_byte_pos += num_bytes_written;

            match current_TINFL_status {
                TINFLStatus::BadParam
                | TINFLStatus::Failed
                | TINFLStatus::FailedCannotMakeProgress => return Err(PngDecodeErrorCode::_14(current_TINFL_status)),
                _ => (),
            }
        }

        Ok(())
    }

    pub fn decompress_once(&mut self, raw_image_bytes: &Vec<u8>) -> Result<(), PngDecodeErrorCode> {
        let result = decompress_to_vec_zlib(raw_image_bytes);

        match result {
            Err(err) => return Err(PngDecodeErrorCode::_14(err)),
            Ok(buf) => {
                self.out_buffer = buf;

                return Ok(());
            }
        }
    }
}
