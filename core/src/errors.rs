#![allow(non_camel_case_types)]
//! There are so many ways a PNG decoding can go wrong.
//! Thus rigorous handling of errors is the first priority.

use std::{
    error::Error,
    fmt::{self, Debug},
};

use miniz_oxide::inflate::TINFLStatus;

use crate::chunk_helpers::{self, ColorType};

#[derive(Debug, Clone)]
pub enum PngDecodeErrorCode {
    /// Invalid Png Header
    _1([u8; 8]),
    /// Invalid chunk order: PLTE chunk appeared before IHDR chunk
    _2,
    /// Invalid data length of IHDR chunk
    _3(usize),
    /// Unsupported bit depth.
    _4(u8, Vec<u8>),
    /// Duplicate PLTE chunk
    _5,
    /// PLTE is forbidden for a given color type but has appeared. `(color_type)`
    _6(ColorType),
    /// CRC checksum mismatch `(expected, actual)`
    _7(u32, u32),
    /// Invalid chunk order: IHDR chunk appears after PLTE
    _8,
    /// Invalid data length of PLTE chunk
    _9(usize),
    /// Unsupported compression method
    _10(u8),
    /// Unsupported filter method
    _11(u8),
    /// Unsupported interlace method `(actual)`
    _12(u8),
    /// First chunk is not IHDR `(actual)`
    _13(String),
    /// Error while decompressing with zlib
    _14(TINFLStatus),
    /// IHDR chunk does not exist before IEND chunk
    _15,
    /// PLTE chunk must appear for certain colortype, but has not appeared until the end (IEND)
    _16(ColorType),
    /// unknown filter type encountered
    _17(u8),
    /// Unsupported color type
    _18(u8),
    /// IHDR chunk (color type) has not been encountered, but tRNS chunk is encountered
    _19,
    /// tRNS chunk must not appear for color types 4 and 6 but has appeared
    _20,
    /// tRNS chunk data length is incorrect for a given color type. `(colortype: ColorType, actual_chunk_length: usize)
    _21(ColorType, usize),
    /// Combination of such bit depth and color type is not permitted
    _22(ColorType, u8),
    /// Pixel type has not been defined yet. Probably tRNS chunk has been encountered before IHDR chunk.
    _23,
}

/// For specific errors that are not in line with the PNG specification and can't really be generalized, meaning that they probably have unique error messages.
#[derive(Debug, Clone)]
pub struct PngDecodeError {
    pub is_recoverable: bool,
    pub code: PngDecodeErrorCode,
    /// The location from the image from which the error probably happened.
    /// Might not be exact, since errors are raised independently of byte reading.
    /// But it will tend to be close to where the real error is, as long as errors are raised as soon as bytes are read and they are found. Better than nothing.
    approx_byte_location: usize,
}

fn recoverable_map(code: PngDecodeErrorCode) -> bool {
    match code {
        // invalid png header
        PngDecodeErrorCode::_1(_) => true,
        // Invalid chunk order: PLTE chunk appeared before IDAT chunk
        PngDecodeErrorCode::_2 => false,
        // Invalid data length of IHDR chunk
        PngDecodeErrorCode::_3(_) => false,
        // Unsupported bit depth
        PngDecodeErrorCode::_4(..) => false,
        // Duplicate PLTE chunk. False for safety
        PngDecodeErrorCode::_5 => false,
        // PLTE is forbidden but has appeared
        PngDecodeErrorCode::_6(_) => true,
        // CRC checksum mismatch
        PngDecodeErrorCode::_7(..) => true,
        // Invalid chunk order: IHDR chunk appears after PLTE.
        PngDecodeErrorCode::_8 => false,
        // Invalid PLTE chunk data length
        PngDecodeErrorCode::_9(_) => false,
        // Unsupported compression method. There is only one compression method anyway
        PngDecodeErrorCode::_10(_) => true,
        // Unsupported filter method. There is only one filter method anyway
        PngDecodeErrorCode::_11(_) => true,
        // Unsupported interlace method. There is only one interlace method anyway
        PngDecodeErrorCode::_12(_) => true,
        // First chunk is not IHDR
        PngDecodeErrorCode::_13(_) => true,
        // Error while decompressing with zlib
        PngDecodeErrorCode::_14(_) => false,
        // IHDR chunk does not appear before IEND chunk
        PngDecodeErrorCode::_15 => false,
        // PLTE chunk must appear for certain colortype, but has not appeared until the end (IEND)
        PngDecodeErrorCode::_16(..) => false,
        PngDecodeErrorCode::_17(..) => false,
        // unsupported color type
        PngDecodeErrorCode::_18(..) => false,
        // tRNS chunk appears before IHDR chunk
        PngDecodeErrorCode::_19 => false,
        // tRNS chunk must not appear for color types 4 and 6 but has appeared
        PngDecodeErrorCode::_20 => true,
        // tRNS chunk data length is incorrect for a given color type
        PngDecodeErrorCode::_21(..) => true,
        // Combination of such bit depth and color type is not permitted
        PngDecodeErrorCode::_22(ColorType, u8) => false,
        // Pixel type has not been defined yet. Probably tRNS chunk has been encountered before IHDR chunk.
        PngDecodeErrorCode::_23 => false,
    }
}

impl PngDecodeError {
    pub fn new(code: PngDecodeErrorCode, approx_byte_location: usize) -> Self {
        Self {
            is_recoverable: recoverable_map(code.clone()),
            code,
            approx_byte_location,
        }
    }
}

impl Error for PngDecodeError {}

impl fmt::Display for PngDecodeError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.code {
      PngDecodeErrorCode::_1(actual) => write!(fmt, "This file does not have a valid png header. Expected header of [137 80 78 71 13 10 26 10] but found {:?}. To fix this error, simply change the header to the expected one.", actual),
      PngDecodeErrorCode::_2 => write!(fmt, "PLTE chunk appeared before IHDR chunk. To fix this error, make PLTE chunk appear after IHDR chunk."),
      PngDecodeErrorCode::_3(data_length) => write!(fmt, "IHDR chunk's data length must be 13 bits, but found {} bits. To fix this error, adjust the length to 13 bits.", data_length),
      PngDecodeErrorCode::_4(actual, expected_one_of) => write!(fmt, "Bit depth of {:?} is not a permitted value. It must be one of {:?}.", actual, expected_one_of),
      PngDecodeErrorCode::_5 => write!(fmt, "More than one PLTE chunk is present. To fix this error, remove duplicate PLTE chunks."),
      PngDecodeErrorCode::_6(color_type) => write!(fmt, "When color type is {:?}, there must not be a PLTE chunk, but it has been encountered", color_type),
      PngDecodeErrorCode::_7(expected, actual ) => write!(fmt, "There is a CRC Checksum mismatch. Expected: {:?}, Actual: {:?}. To fix this error, replace the value with the correct CRC checksum based on the chunk type and chunk data.", expected, actual),
      PngDecodeErrorCode::_8 => write!(fmt, "IHDR chunk appeared after PLTE chunk. To fix this error, make IHDR chunk appear before PLTE chunk."),
      PngDecodeErrorCode::_9(actual) => write!(fmt, "PLTE chunk's data length must be divisible by 3, but found {} which is not divisible by 3.", actual),
      PngDecodeErrorCode::_10(actual) => write!(fmt, "IHDR chunk contains unsupported compression method: {}. To fix this error, change it to 0.", actual),
      PngDecodeErrorCode::_11(actual) => write!(fmt, "IHDR chunk contains unsupported filter method: {}. To fix this error, change to 0.", actual),
      PngDecodeErrorCode::_12(actual) => write!(fmt, "Interlace method must be 0 or 1, but found {}. To fix this error, first find out if your image is interlaced. If so, change the number of the corresponding byte to 1. Otherwise, 0.", actual),
      PngDecodeErrorCode::_13(chunk_type) => write!(fmt, "The first chunk type must be IHDR, but found {} instead", chunk_type),
      PngDecodeErrorCode::_14(reason) => write!(fmt, "Failed to decompress IDAT chunk data. Reason: {:?}. It is likely that the data in the IDAT chunk is corrupt", reason),
      PngDecodeErrorCode::_15 => write!(fmt, "IHDR chunk has not appeared before IEND chunk. To fix this error, make IHDR chunk appear before it."),
      PngDecodeErrorCode::_16(color_type) => write!(fmt, "Color type of the image is {:?}, for which PLTE chunk must appear. However it has not been found until IEND chunk was reached. Please insert a PLTE chunk.", color_type),
      PngDecodeErrorCode::_17(unsupported_filter_type) => write!(fmt, "Unknown filter type of {} encountered. It needs to be an integer between 0 and 4.", unsupported_filter_type),
      PngDecodeErrorCode::_18(unsupported_color_type) => write!(fmt, "Unsupported color type of {} encountered. It needs to be one of {:?}", unsupported_color_type, [ColorType::Greyscale, ColorType::Truecolor, ColorType::IndexedColor,  ColorType::GreyscaleAlpha, ColorType::TruecolorAlpha]),
      PngDecodeErrorCode::_19 => write!(fmt, "tRNS chunk has appeared before IHDR chunk. To fix this error, make IHDR chunk containing a correct color type appear before tRNS chunk."),
      PngDecodeErrorCode::_20 => write!(fmt, "tRNS chunk must not appear for color type 4 or 6, but it has appeared. To fix this error, change the color type to 0, 2, 3 or delete tRNS chunk."),
      PngDecodeErrorCode::_21(color_type, actual_chunk_length) => write!(fmt, "tRNS chunk length must be {} for color type of {:?}, but the actual chunk length is found to be {}. To fix this error, set a correct chunk length for tRNS chunk.", chunk_helpers::colortype_to_alpha_byte_length(*color_type), *color_type, actual_chunk_length),
      PngDecodeErrorCode::_22(color_type, bit_depth) => write!(fmt, "Combination of color type of {:?} and bit depth of {} is not permitted.", color_type, bit_depth),
      PngDecodeErrorCode::_23 => write!(fmt, "Pixel type has not been defined yet. Probably tRNS chunk has been encountered before IHDR chunk."),
    }
    }
}

pub enum ForceExitReason {
    /// The user wants to fail fast when the first error is encountered.
    FailFast,
    /// The program has encountered an unrecoverable error.
    Unrecoverable,
}

pub enum ExitReason {
    /// The program has successfully finished the job through a normal course.
    JobDone,
}

/// Because the program can have multiple errors and still not quit,
/// There needs to be a way to store multiple errors
pub struct MultiErrorsManager {
    errors: Vec<PngDecodeError>,
    fail_fast: bool,
}

impl MultiErrorsManager {
    pub fn new(fail_fast: bool) -> Self {
        MultiErrorsManager {
            errors: vec![],
            fail_fast,
        }
    }

    fn print_all_errors(&self) {
        let error_len = self.errors.len();
        if error_len > 0 {
            println!(
                "\x1b[93mTotal {} errors found:\x1b[0m",
                self.errors.len()
            );
            for (pos, e) in self.errors.iter().enumerate() {
                eprint!(
                    "\x1b[93m[Error #{}]\x1b[0m: {}",
                    pos, e
                );
                println!(
                    " [Approx. byte location of error]: {}",
                    e.approx_byte_location
                );
            }
        } else {
            println!("Validating & decoding PNG completed with no errors.")
        }
    }

    // Add an error.
    pub fn handle_err(&mut self, err: PngDecodeError) -> PngDecodeError {
        self.errors.push(err.clone());
        self.print_all_errors();
        if !err.is_recoverable {
            self.force_end(ForceExitReason::Unrecoverable);
        }
        if self.fail_fast {
            self.force_end(ForceExitReason::FailFast);
        }
        return err;
    }

    /// the program has panicked
    pub fn force_end(&self, end_reason: ForceExitReason) -> ! {
        match end_reason {
            ForceExitReason::FailFast => panic!(
                "Ending program because fail_fast is set to true and first error is encountered."
            ),
            ForceExitReason::Unrecoverable => {
                panic!("Ending program because the program has encountered an unrecoverable error.")
            }
        }
    }
    pub fn end(&self, end_reason: ExitReason) {
        match end_reason {
            ExitReason::JobDone => {
                if self.errors.len() > 0 {
                    println!("✔ Validated and decoded png with some recoverable errors.")
                } else {
                    println!("✔ Validated and decoded png without any errors.")
                }
            }
        }
    }
}
