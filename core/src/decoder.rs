#![allow(non_camel_case_types)]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    bitmap::{self, PixelType},
    byte_reader,
    chunk_helpers::{self, colortype_to_channel, ColorType, InterlaceMethod},
    chunk_types, common,
    deinterlace::{self, ReducedImage},
    errors::{self, PngDecodeErrorCode},
    trns::TransparencyChunk,
    unfilter, zlib,
};

pub struct PngDecoderResult {
    pub bytes: Vec<u8>,
    pub errors: errors::PngDecodeError,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct PngDecoderOptions {
    pub fail_fast: bool,
    pub validate_crc: bool,
}

#[wasm_bindgen]
impl PngDecoderOptions {
    #[wasm_bindgen(constructor)]
    pub fn new(fail_fast: bool, validate_crc: bool) -> Self {
        PngDecoderOptions {
            fail_fast,
            validate_crc,
        }
    }
}

pub struct PngDecoder<'a> {
    /// http://www.libpng.org/pub/png/spec/1.2/PNG-Chunks.html#C.IHDR
    /// 
    /// PNG four-byte unsigned integers are limited to the range 0 to (2^31)-1 to accommodate languages that have difficulty with unsigned four-byte values. Thus it makes sense to use u32 for width and height.
    /// 
    /// Width in pixels from IHDR chunk
    width: Option<u32>,
    /// height in pixels from IHDR chunk
    height: Option<u32>,
    /// bit depth from IHDR chunk
    bit_depth: Option<u8>,
    /// color type from IHDR chunk
    color_type: Option<chunk_helpers::ColorType>,
    /// interlace method, from IHDR chunk
    interlace_method: Option<chunk_helpers::InterlaceMethod>,
    /// filter method from IHDR chunk
    filter_method: Option<chunk_helpers::FilterMethod>,
    /// compression method from IHDR chunk (0 or 1)
    compression_method: Option<chunk_helpers::CompressionMethod>,
    /// additional one byte flag that appears right aftert compression method, from the first IDAT chunk
    additional_flag_after_compression_method: Option<u8>,
    /// zlib compression method from the first IDAT chunk (specifically used for zlib header)
    /// this is not the same as `PngDecoder.compression_method`
    zlib_compression_method: Option<u8>,
    /// The PLTE chunk contains from 1 to 256 palette entries, each a three-byte series of the form:
    ///
    /// Red:   1 byte (0 = black, 255 = red)
    ///
    /// Green: 1 byte (0 = black, 255 = green)
    ///
    /// Blue:  1 byte (0 = black, 255 = blue)
    ///
    /// The number of entries is determined from the chunk length. A chunk length not divisible by 3 is an error.
    ///
    /// http://www.libpng.org/pub/png/spec/1.2/PNG-Chunks.html#C.PLTE
    palette: Option<Vec<u8>>,
    /// true if the decoder has encountered an IHDR chunk
    has_ihdr: bool,
    /// true if the decoder has encountered an IDAT chunk
    has_idat: bool,
    /// true if the decoder has encountered an PLTE chunk
    has_plte: bool,
    /// true if the color type from IHDR chunk supports alpha channel.
    /// see [ALPHA_CHANNEL_COLOR_TYPES]
    has_alpha_channel: Option<bool>,
    /// alpha information from tRNS chunk if available
    transparency_chunk: Option<TransparencyChunk>,
    pixel_type: Option<PixelType>,
    /// Byte reader. Takes care of reading the raw bytes from the input file/raw pixels.
    /// PngDecoder can do that as well, but separated into a different impl for separation of concerns, so that PngDecoder only focuses on parsing the actual PNG data.
    byte_reader: &'a mut byte_reader::ByteReader<'a>,
    /// zlib stream to decompress raw image data
    zlib_decompress_stream: zlib::ZlibDecompressStream,
    // final unfiltered output
    unfiltered_output: Vec<u8>,
    /// cli params
    decoder_options: &'a PngDecoderOptions,
    /// the decoder manages errors throughout the program
    /// using ErrorManager.
    multi_errors_manager: errors::MultiErrorsManager,
    /// 6 bytes per pixel (48-bit RGB, 16-bit mode image type) is the maximum.
    /// However, calculation with usize integers is often needed, so set the size as `usize`
    bytes_per_pixel: usize,
    /// How many bytes are there per line (`bytes_per_line * height = entire image bytes`).
    /// this does NOT include the filter byte, meaning that the decoder will need to account for it when unfiltering the image.
    bytes_per_line: usize,
}

impl<'a> PngDecoder<'a> {
    pub fn new(
        byte_reader: &'a mut byte_reader::ByteReader<'a>,
        decoder_options: &'a PngDecoderOptions,
    ) -> PngDecoder<'a> {
        PngDecoder {
            width: None,
            height: None,
            bit_depth: None,
            color_type: None,
            interlace_method: None,
            filter_method: None,
            compression_method: None,
            additional_flag_after_compression_method: None,
            zlib_compression_method: None,
            palette: None,
            has_ihdr: false,
            has_idat: false,
            has_plte: false,
            has_alpha_channel: None,
            transparency_chunk: None,
            pixel_type: None,
            byte_reader,
            zlib_decompress_stream: zlib::ZlibDecompressStream::new(None),
            unfiltered_output: vec![],
            decoder_options,
            multi_errors_manager: errors::MultiErrorsManager::new(
                decoder_options.fail_fast.clone(),
            ),
            bytes_per_pixel: 0,
            bytes_per_line: 0,
        }
    }

    fn create_error(&mut self, code: errors::PngDecodeErrorCode) -> errors::PngDecodeError {
        let err = errors::PngDecodeError::new(
            code,
            self.byte_reader.get_current_byte_pos(),
        );
        self.multi_errors_manager.handle_err(err.clone());

        return err;
    }

    /// Checks if magic header is correct.
    /// https://www.w3.org/TR/PNG-Rationale.html#R.PNG-file-signature
    fn read_header(&mut self) -> Result<(), errors::PngDecodeError> {
        let buffer: Vec<u8> = self.byte_reader.read_next_n_bytes(8);
        return match buffer[0..8]
            .try_into()
            .expect("PNG header buffer should have length of 8")
        {
            chunk_helpers::PNG_HEADER => Ok(()),
            invalid_header => {
                Err(self.create_error(errors::PngDecodeErrorCode::_1(invalid_header)))
            }
        };
    }

    fn finalize_at_iend_chunk(&mut self) {
        if !self.has_ihdr {
            self.create_error(PngDecodeErrorCode::_15);
        }
        let color_type = self
            .color_type
            .expect("Color type must have been obtained already from IHDR chunk");

        self.has_alpha_channel =
            Some(chunk_helpers::does_colortype_support_alpha_channel(color_type));

        // See [decode_plte_chunk]
        if !self.has_plte && color_type == chunk_helpers::ColorType::IndexedColor {
            self.create_error(PngDecodeErrorCode::_16(
                color_type,
            ));
        }
    }

    /// validates ihdr chunk and returns bit depth, color type, compression method, filter method, interlace method
    /// in order
    fn validate_ihdr_chunk(
        &mut self,
        chunk: &Vec<u8>,
    ) -> (
        u8,
        chunk_helpers::ColorType,
        chunk_helpers::CompressionMethod,
        chunk_helpers::FilterMethod,
        chunk_helpers::InterlaceMethod,
    ) {
        let ihdr_chunk_data_length = chunk.len();
        if ihdr_chunk_data_length != 13 {
            self.create_error(errors::PngDecodeErrorCode::_3(ihdr_chunk_data_length));
        }

        let color_type: chunk_helpers::ColorType = match chunk[9].try_into() {
            Ok(ct) => ct,
            Err(_) => {
                self.create_error(PngDecodeErrorCode::_18(
                    chunk[9],
                ));
                ColorType::Greyscale
            }
        };
        
        let bit_depth = chunk[8];
        let supported_bit_depths =
            chunk_helpers::get_supported_color_type_to_bit_depths(color_type);

        if !supported_bit_depths.contains(&bit_depth) {
            self.create_error(
                errors::PngDecodeErrorCode::_4(
                    bit_depth,
                    supported_bit_depths,
                ),
            );
        }

        let compression_method: chunk_helpers::CompressionMethod = match chunk[10].try_into() {
            Ok(compression_method) => compression_method,
            Err(_) => {
                self.create_error(errors::PngDecodeErrorCode::_10(chunk[10]));

                chunk_helpers::CompressionMethod::Deflate
            }
        };

        let filter_method = match chunk[11].try_into() {
            Ok(filter_method) => filter_method,
            Err(_) => {
                self.create_error(errors::PngDecodeErrorCode::_11(chunk[11]));

                chunk_helpers::FilterMethod::Adaptive
            }
        };

        let interlace_method: InterlaceMethod = match chunk[12].try_into() {
            Ok(interlace_method) => interlace_method,
            Err(_) => {
                self.create_error(errors::PngDecodeErrorCode::_12(chunk[12]));
                self.multi_errors_manager.force_end(errors::ForceExitReason::Unrecoverable)
            }
        };

        return (
            bit_depth,
            color_type,
            compression_method,
            filter_method,
            interlace_method,
        );
    }

    /// The IHDR chunk must appear FIRST. It contains:
    ///
    /// Width:              4 bytes
    ///
    /// Height:             4 bytes
    ///
    /// Bit depth:          1 byte
    ///
    /// Color type:         1 byte
    ///
    /// Compression method: 1 byte
    ///
    /// Filter method:      1 byte
    ///
    /// Interlace method:   1 byte
    fn decode_ihdr_chunk(&mut self, chunk: &Vec<u8>) {
        let (bit_depth, color_type, compression_method, filter_method, interlace_method) =
            self.validate_ihdr_chunk(chunk);

        self.has_ihdr = true;
        let width = self.byte_reader.read_next_u32_num(&chunk[0..4].to_vec());
        self.width = Some(width);
        let height = self.byte_reader.read_next_u32_num(&chunk[4..8].to_vec());
        self.height = Some(height);
        self.bit_depth = Some(bit_depth);
        self.color_type = Some(color_type);
        self.compression_method = Some(compression_method);
        self.filter_method = Some(filter_method);
        self.interlace_method = Some(interlace_method);

        let (bytes_per_pixel, bytes_per_line) = common::calc_bytes_per_pixel_and_line(
            colortype_to_channel(color_type),
            bit_depth,
            width,
        );

        self.bytes_per_line = bytes_per_line;
        self.bytes_per_pixel = bytes_per_pixel;
        self.pixel_type = match PixelType::new(color_type, bit_depth) {
            Ok(pt) => Some(pt),
            Err(err_code) => {
                self.create_error(err_code);
                self.multi_errors_manager
                    .force_end(errors::ForceExitReason::Unrecoverable)
            }
        };
    }

    fn validate_plte_chunk(&mut self, chunk: &Vec<u8>) {
        let chunk_length = chunk.len();
        if chunk_length % 3 != 0 {
            self.create_error(errors::PngDecodeErrorCode::_9(chunk_length));
        }
        if self.has_idat {
            self.create_error(errors::PngDecodeErrorCode::_2);
        }
        if self.has_plte {
            self.create_error(errors::PngDecodeErrorCode::_5);
        }
        if !self.has_ihdr {
            self.create_error(errors::PngDecodeErrorCode::_2);
        }

        let color_type = self
            .color_type
            .expect("Color type must have been obtained from IHDR chunk when parsing PLTE chunk");

        match color_type {
            // we're not using the returned value
            chunk_helpers::ColorType::Greyscale | chunk_helpers::ColorType::GreyscaleAlpha => {
                drop(self.create_error(errors::PngDecodeErrorCode::_6(color_type)))
            }
            _ => (),
        };
    }

    ///     The PLTE chunk contains from 1 to 256 palette entries, each a three-byte series of the form:
    ///
    ///     Red:   1 byte (0 = black, 255 = red)
    ///     Green: 1 byte (0 = black, 255 = green)
    ///     Blue:  1 byte (0 = black, 255 = blue)
    ///     The number of entries is determined from the chunk length.
    ///     A chunk length not divisible by 3 is an error.
    ///
    ///  This chunk must appear for color type 3, and can appear for color types 2 and 6;
    ///  it must not appear for color types 0 and 4.
    ///  If this chunk does appear, it must precede the first IDAT chunk.
    ///  There must not be more than one PLTE chunk.
    fn decode_plte_chunk(&mut self, chunk: &Vec<u8>) {
        self.validate_plte_chunk(&chunk);

        self.has_plte = true;
        self.palette = Some(chunk.to_vec());
    }

    /// Parses the IDAT chunk.
    ///
    /// If it is the first IDAT chunk in the entire image,
    /// the first TWO bytes must represent compression method and
    /// an additional flag after compression method.
    /// the following bytes represent raw image data.
    /// If it is not the first chunk,
    /// all bytes in the chunk data represent the image data.
    ///
    ///  It is important to emphasize that IDAT chunk boundaries have no semantic significance
    ///  and can occur at any point in the compressed datastream.
    /// A PNG file in which each IDAT chunk contains only one data byte is valid,
    /// though remarkably wasteful of space.
    /// (For that matter, zero-length IDAT chunks are valid, though even more wasteful.)
    fn decode_idat_chunk(&mut self, chunk: &Vec<u8>) {
        // avoid accessing empty IDAT chunk
        if chunk.len() == 0 {
            self.has_idat = true;
            return;
        }
        if self.zlib_compression_method.is_none() && chunk.len() > 0 {
            self.zlib_compression_method = Some(chunk[0]);
        } else if self.additional_flag_after_compression_method.is_none() && chunk.len() > 0 {
            self.additional_flag_after_compression_method = Some(chunk[0]);
        }
        if self.additional_flag_after_compression_method.is_none() && chunk.len() > 1 {
            self.additional_flag_after_compression_method = Some(chunk[1]);
        }

        match self.zlib_decompress_stream.decompress(&chunk) {
            Ok(_) => (),
            Err(reason) => {
                self.create_error(PngDecodeErrorCode::_14(
                    reason,
                ));
            }
        };
        if !self.has_idat {
            self.has_idat = true
        }
    }

    #[allow(non_snake_case)]
    fn validate_tRNS_chunk(&mut self, chunk: &Vec<u8>) -> (ColorType, PixelType) {
        let color_type = match self.color_type {
            Some(ct) => ct,
            _ => {
                self.create_error(errors::PngDecodeErrorCode::_19);
                self.multi_errors_manager
                    .force_end(errors::ForceExitReason::Unrecoverable);
            }
        };
        let expected_chunk_length = chunk_helpers::colortype_to_alpha_byte_length(color_type);
        if chunk.len() != expected_chunk_length as usize && expected_chunk_length != 0 {
            self.create_error(errors::PngDecodeErrorCode::_21(color_type, chunk.len()));
        }

        let pixel_type = match self.pixel_type {
            Some(pt) => pt,
            _ => {
                self.create_error(errors::PngDecodeErrorCode::_23);
                self.multi_errors_manager
                    .force_end(errors::ForceExitReason::Unrecoverable);
            }
        };

        return (color_type, pixel_type);
    }

    #[allow(non_snake_case)]
    fn decode_tRNS_chunk(&mut self, chunk: &Vec<u8>) {
        let (color_type, pixel_type) = self.validate_tRNS_chunk(&chunk);

        match color_type {
            ColorType::Greyscale | ColorType::Truecolor | ColorType::IndexedColor => {
                self.transparency_chunk = TransparencyChunk::new(chunk.to_vec(), pixel_type);
            }
            _ => {
                self.create_error(errors::PngDecodeErrorCode::_20);
            }
        }
    }

    /// Validates CRC. Adds an error when there is a mismatch between
    /// calculated CRC and existing CRC in a PNG chunk.
    fn validate_crc(&mut self, actual_chunk_crc: u32, chunk_type_and_chunk_data: &[u8]) {
        let expected_chunk_crc = crc32fast::hash(chunk_type_and_chunk_data);

        if actual_chunk_crc != expected_chunk_crc {
            self.create_error(
                errors::PngDecodeErrorCode::_7(
                    expected_chunk_crc,
                    actual_chunk_crc,
                ),
            );
        }
    }

    fn decode_chunks(&mut self) {
        let _ = self.read_header();

        loop {
            let chunk_data_length = self.byte_reader.read_next_4bytes_num();
            let (chunk_type, chunk_type_bytes) = self.byte_reader.read_next_4bytes_str();
            let chunk_data = self.byte_reader.read_next_n_bytes(chunk_data_length.into());

            if !self.has_ihdr && chunk_type != chunk_types::ChunkTypes::IHDR {
                self.create_error(errors::PngDecodeErrorCode::_13(chunk_type.clone()));
            }

            let mut needs_break = false;
            match chunk_type.as_ref() {
                chunk_types::ChunkTypes::IHDR => self.decode_ihdr_chunk(&chunk_data),
                chunk_types::ChunkTypes::IDAT => self.decode_idat_chunk(&chunk_data),
                chunk_types::ChunkTypes::PLTE => self.decode_plte_chunk(&chunk_data),
                chunk_types::ChunkTypes::tRNS => self.decode_tRNS_chunk(&chunk_data),
                chunk_types::ChunkTypes::IEND => {
                    self.finalize_at_iend_chunk();
                    needs_break = true;
                }
                unknown_type => {
                    // println!(
                    //     "warning: unknown type {}",
                    //     unknown_type
                    // );
                }
            }
            let chunk_crc = self.byte_reader.read_next_4bytes_num();
            if self.decoder_options.validate_crc {
                // Consume them instead of referencing, cloning, or copying
                // because we are not going to use these values after this line
                let chunk_type_and_chunk_data: Vec<u8> = chunk_type_bytes
                    .into_iter()
                    .chain(chunk_data.into_iter())
                    .collect();
                self.validate_crc(
                    chunk_crc,
                    &chunk_type_and_chunk_data,
                );
            }
            if needs_break {
                break;
            }
        }
    }

    fn unfilter_interlaced_image(&mut self) -> Vec<ReducedImage> {
        let mut unknown_filter_type: Option<u8> = None;

        let height = self.height.expect("Height is None");
        let decompressed_data = self.zlib_decompress_stream.get_out_buffer();

        let reduced_images = deinterlace::create_reduced_images(
            self.width.expect("Width is None"),
            height,
            colortype_to_channel(self.color_type.expect("Color type is None")),
            self.bit_depth.expect("Bit depth is None"),
        );

        let mut decompressed_data_cursor: usize = 0;
        let mut unfiltered_output_cursor: usize = 0;
        let unfiltered_output_length: usize = reduced_images
            .iter()
            .map(|img| img.pixel_height as usize * img.bytes_per_line)
            .sum();
        self.unfiltered_output.resize_with(
            unfiltered_output_length,
            Default::default,
        );

        for (_, reduced_image) in reduced_images.iter().enumerate() {
            if reduced_image.pixel_height == 0 && reduced_image.pixel_width == 0 {
                // ignore empty scanline
                continue;
            }

            let current_scanline_length_without_filter_bytes =
                reduced_image.bytes_per_line * reduced_image.pixel_height as usize;

            let mut unfilter_processor = unfilter::UnfilterProcessor::new(
                reduced_image.pixel_height,
                reduced_image.bytes_per_pixel,
                reduced_image.bytes_per_line,
            );

            // for safety
            if reduced_image.bytes_per_line == 0 {
                continue;
            }

            // next_bytes_to_unfilter = current_scanline_length_without_filter_bytes + length of filter bytes in current scanline
            let next_bytes_to_unfilter =
                current_scanline_length_without_filter_bytes + reduced_image.pixel_height as usize;

            match unfilter_processor.unfilter(
                &decompressed_data
                    [decompressed_data_cursor..decompressed_data_cursor + next_bytes_to_unfilter],
                &mut self.unfiltered_output[unfiltered_output_cursor
                    ..unfiltered_output_cursor + current_scanline_length_without_filter_bytes],
            ) {
                Ok(_) => (),
                Err(filter_type) => {
                    unknown_filter_type = Some(filter_type);
                    break;
                }
            };
            decompressed_data_cursor = decompressed_data_cursor + next_bytes_to_unfilter;
            unfiltered_output_cursor =
                unfiltered_output_cursor + current_scanline_length_without_filter_bytes;
        }
        match unknown_filter_type {
            None => (),
            Some(filter_type) => {
                self.create_error(PngDecodeErrorCode::_17(
                    filter_type,
                ));
            }
        }

        return reduced_images.to_vec();
    }

    fn unfilter_non_interlaced_image(&mut self) -> Vec<ReducedImage> {
        let height = self.height.expect("Height is None");
        let decompressed_data = self.zlib_decompress_stream.get_out_buffer();

        let mut unfilter_processor = unfilter::UnfilterProcessor::new(
            height,
            self.bytes_per_pixel,
            self.bytes_per_line,
        );
        // expected full image size in bytes (does not include filter bytes)
        let maximum_possible_byte_width = (self.bytes_per_line * height as usize) as usize;
        self.unfiltered_output.resize_with(
            maximum_possible_byte_width,
            Default::default,
        );
        match unfilter_processor.unfilter(
            &decompressed_data,
            &mut self.unfiltered_output,
        ) {
            Ok(_) => (),
            Err(unknown_filter_type) => {
                self.create_error(PngDecodeErrorCode::_17(
                    unknown_filter_type,
                ));
            }
        };

        return vec![ReducedImage {
            pixel_width: self.width.expect("Width is None"),
            pixel_height: height,
            bytes_per_line: self.bytes_per_line,
            bytes_per_pixel: self.bytes_per_pixel,
        }];
    }
    
    /// outputs data in rgba (4 bytes) for each pixel
    fn to_rgba_vec(&self, reduced_images: Vec<ReducedImage>) -> Vec<u8> {
        let width = self.width.expect("Width is None");
        let height = self.height.expect("Height is None");
        let rgba_data_length = ((width * height) * 4) as usize;
        let mut rgba_data = vec![0u8; rgba_data_length];
        let pixel_type = self.pixel_type.expect("Pixel type is None");
        let interlace_method = self.interlace_method.expect("Interlace method is None");

        let trns = self.transparency_chunk.clone();

        let mut previous_reduced_image_offset: usize = 0;
        for (nth_pass, reduced_image) in reduced_images.iter().enumerate() {
            for row_index in 0..reduced_image.pixel_height {
                let current_scanline_offset = row_index as usize * reduced_image.bytes_per_line as usize;
                let current_scanline_start = previous_reduced_image_offset + current_scanline_offset;
                let current_scanline_end = current_scanline_start + reduced_image.bytes_per_line;
                let current_scanline = &self.unfiltered_output[current_scanline_start..current_scanline_end];

                for col_index in 0..reduced_image.pixel_width {
                    let rgba_data_start_index = match interlace_method {
                        InterlaceMethod::Adam7 => deinterlace::calc_interlaced_pixel_index(
                            col_index as usize,
                            row_index as usize,
                            (nth_pass + 1) as u8,
                            width,
                        ),
                        InterlaceMethod::None => {
                            ((row_index as u64 * width as u64 * 4) + (col_index as u64 * 4))
                                as usize
                        }
                    };

                    let (r, g, b, a) = bitmap::to_rgba_pixel_bytes(
                        pixel_type,
                        trns.as_ref(),
                        self.palette.as_ref(),
                        col_index as usize,
                        current_scanline,
                    );
                    if rgba_data_start_index > rgba_data_length {
                        break;
                    }

                    rgba_data[rgba_data_start_index] = r;
                    rgba_data[rgba_data_start_index + 1] = g;
                    rgba_data[rgba_data_start_index + 2] = b;
                    rgba_data[rgba_data_start_index + 3] = a;                    
                }
            }
            previous_reduced_image_offset +=
                (reduced_image.pixel_height as usize * reduced_image.bytes_per_line) as usize
        }

        return rgba_data;
    }

    /// returns RGBA vec
    pub fn run(&mut self) -> Vec<u8> {
        self.decode_chunks();

        // length is 1 or 7 based on interlace == 0 or 1
        let reduced_images = match self.interlace_method.expect("Interlace method is None") {
            chunk_helpers::InterlaceMethod::None => self.unfilter_non_interlaced_image(),
            chunk_helpers::InterlaceMethod::Adam7 => self.unfilter_interlaced_image(),
        };
        let rgba_vec = self.to_rgba_vec(reduced_images);
        self.multi_errors_manager.end(errors::ExitReason::JobDone);

        // println!("{:?}", (self.fil))
        // println!("{:?}", rgba_vec);
        return rgba_vec;
    }
}
