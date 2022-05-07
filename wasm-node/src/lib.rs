use core::{byte_reader, decoder};
use js_sys::Uint8Array;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn decode_raw_bytes(
    raw_bytes: Vec<u8>,
    decoder_options: &decoder::PngDecoderOptions,
) -> Uint8Array {
    let mut byte_reader = byte_reader::ByteReader::new(
        None,
        byte_reader::ByteReaderMode::RAW,
        Some(raw_bytes),
    );
    let mut decoder = decoder::PngDecoder::new(
        &mut byte_reader,
        decoder_options,
    );
    return decoder.run().as_slice().into();
}
