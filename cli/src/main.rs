mod cli;

use core::{byte_reader, decoder};

fn main() {
    let mut cli = cli::Cli::new();
    cli.init();
    let input_file_path = cli.get_input_file_path();
    let mut byte_reader = byte_reader::ByteReader::new(
        Some(&input_file_path),
        byte_reader::ByteReaderMode::FILE,
        None,
    );
    byte_reader.read_image();
    let decode_options = decoder::PngDecoderOptions {
        fail_fast: cli.fail_fast,
        validate_crc: cli.validate_crc,
    };
    let mut decoder = decoder::PngDecoder::new(
        &mut byte_reader,
        &decode_options,
    );
    let decoded_bytes = decoder.run();
    // println!("{:?}", decoded_bytes);
}
