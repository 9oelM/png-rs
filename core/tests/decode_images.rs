// use core::{self};
// use std::{
//     fs::{self, DirEntry},
//     path::PathBuf,
// };

// fn get_test_image_paths(is_corrupt: bool) -> Vec<String> {
//     let all_files =
//         fs::read_dir("../test/png/official").expect("Test image files directory must be readable");

//     let png_paths: Vec<String> = all_files
//         .filter_map(|maybe_png| {
//             let dir_entry: DirEntry = maybe_png.ok()?;
//             let path_buf: PathBuf = dir_entry.path();
//             let file_path = path_buf
//                 .clone()
//                 .into_os_string()
//                 .into_string()
//                 .expect("OS string is not ok");
//             let file_name: &str = path_buf.file_name()?.to_str()?;

//             if file_path.ends_with(".png")
//                 && ((is_corrupt && file_name.starts_with("x"))
//                     || (!is_corrupt && !file_name.starts_with("x")))
//             {
//                 Some(file_path)
//             } else {
//                 None
//             }
//         })
//         .collect();

//     return png_paths;
// }

// Anyone know a better way to iterate?? Like .each in jest...
#[cfg(test)]
mod tests {
    use core::byte_reader;
    use image::EncodableLayout;
    use test_case::test_case;

    #[test_case("../test/png/official/s39i3p04.png";"Decoding ../test/png/official/s39i3p04.png) should work")]
    #[test_case("../test/png/official/s38i3p04.png";"Decoding ../test/png/official/s38i3p04.png) should work")]
    #[test_case("../test/png/official/s05n3p02.png";"Decoding ../test/png/official/s05n3p02.png) should work")]
    #[test_case("../test/png/official/cs5n2c08.png";"Decoding ../test/png/official/cs5n2c08.png) should work")]
    #[test_case("../test/png/official/bgwn6a08.png";"Decoding ../test/png/official/bgwn6a08.png) should work")]
    #[test_case("../test/png/official/basn3p02.png";"Decoding ../test/png/official/basn3p02.png) should work")]
    #[test_case("../test/png/official/bggn4a16.png";"Decoding ../test/png/official/bggn4a16.png) should work")]
    #[test_case("../test/png/official/tbgn3p08.png";"Decoding ../test/png/official/tbgn3p08.png) should work")]
    #[test_case("../test/png/official/basi0g08.png";"Decoding ../test/png/official/basi0g08.png) should work")]
    #[test_case("../test/png/official/bgbn4a08.png";"Decoding ../test/png/official/bgbn4a08.png) should work")]
    #[test_case("../test/png/official/basn4a16.png";"Decoding ../test/png/official/basn4a16.png) should work")]
    #[test_case("../test/png/official/s04n3p01.png";"Decoding ../test/png/official/s04n3p01.png) should work")]
    #[test_case("../test/png/official/s33i3p04.png";"Decoding ../test/png/official/s33i3p04.png) should work")]
    #[test_case("../test/png/official/s32i3p04.png";"Decoding ../test/png/official/s32i3p04.png) should work")]
    #[test_case("../test/png/official/cm0n0g04.png";"Decoding ../test/png/official/cm0n0g04.png) should work")]
    #[test_case("../test/png/official/basn3p01.png";"Decoding ../test/png/official/basn3p01.png) should work")]
    #[test_case("../test/png/official/basi2c08.png";"Decoding ../test/png/official/basi2c08.png) should work")]
    #[test_case("../test/png/official/bgyn6a16.png";"Decoding ../test/png/official/bgyn6a16.png) should work")]
    #[test_case("../test/png/official/ps1n2c16.png";"Decoding ../test/png/official/ps1n2c16.png) should work")]
    #[test_case("../test/png/official/g03n3p04.png";"Decoding ../test/png/official/g03n3p04.png) should work")]
    #[test_case("../test/png/official/basn6a16.png";"Decoding ../test/png/official/basn6a16.png) should work")]
    #[test_case("../test/png/official/ccwn3p08.png";"Decoding ../test/png/official/ccwn3p08.png) should work")]
    #[test_case("../test/png/official/oi1n2c16.png";"Decoding ../test/png/official/oi1n2c16.png) should work")]
    #[test_case("../test/png/official/f03n0g08.png";"Decoding ../test/png/official/f03n0g08.png) should work")]
    #[test_case("../test/png/official/f02n0g08.png";"Decoding ../test/png/official/f02n0g08.png) should work")]
    #[test_case("../test/png/official/oi1n0g16.png";"Decoding ../test/png/official/oi1n0g16.png) should work")]
    #[test_case("../test/png/official/s06i3p02.png";"Decoding ../test/png/official/s06i3p02.png) should work")]
    #[test_case("../test/png/official/tbyn3p08.png";"Decoding ../test/png/official/tbyn3p08.png) should work")]
    #[test_case("../test/png/official/s07i3p02.png";"Decoding ../test/png/official/s07i3p02.png) should work")]
    #[test_case("../test/png/official/cdhn2c08.png";"Decoding ../test/png/official/cdhn2c08.png) should work")]
    #[test_case("../test/png/official/f03n2c08.png";"Decoding ../test/png/official/f03n2c08.png) should work")]
    #[test_case("../test/png/official/f02n2c08.png";"Decoding ../test/png/official/f02n2c08.png) should work")]
    #[test_case("../test/png/official/cs8n3p08.png";"Decoding ../test/png/official/cs8n3p08.png) should work")]
    #[test_case("../test/png/official/basn3p04.png";"Decoding ../test/png/official/basn3p04.png) should work")]
    #[test_case("../test/png/official/g04n0g16.png";"Decoding ../test/png/official/g04n0g16.png) should work")]
    #[test_case("../test/png/official/g05n0g16.png";"Decoding ../test/png/official/g05n0g16.png) should work")]
    #[test_case("../test/png/official/cthn0g04.png";"Decoding ../test/png/official/cthn0g04.png) should work")]
    #[test_case("../test/png/official/cs3n3p08.png";"Decoding ../test/png/official/cs3n3p08.png) should work")]
    #[test_case("../test/png/official/bgai4a08.png";"Decoding ../test/png/official/bgai4a08.png) should work")]
    #[test_case("../test/png/official/basi3p08.png";"Decoding ../test/png/official/basi3p08.png) should work")]
    #[test_case("../test/png/official/g05n3p04.png";"Decoding ../test/png/official/g05n3p04.png) should work")]
    #[test_case("../test/png/official/g04n3p04.png";"Decoding ../test/png/official/g04n3p04.png) should work")]
    #[test_case("../test/png/official/bgan6a16.png";"Decoding ../test/png/official/bgan6a16.png) should work")]
    #[test_case("../test/png/official/f04n0g08.png";"Decoding ../test/png/official/f04n0g08.png) should work")]
    #[test_case("../test/png/official/ct0n0g04.png";"Decoding ../test/png/official/ct0n0g04.png) should work")]
    #[test_case("../test/png/official/ct1n0g04.png";"Decoding ../test/png/official/ct1n0g04.png) should work")]
    #[test_case("../test/png/official/f04n2c08.png";"Decoding ../test/png/official/f04n2c08.png) should work")]
    #[test_case("../test/png/official/tbbn2c16.png";"Decoding ../test/png/official/tbbn2c16.png) should work")]
    #[test_case("../test/png/official/cten0g04.png";"Decoding ../test/png/official/cten0g04.png) should work")]
    #[test_case("../test/png/official/basn0g01.png";"Decoding ../test/png/official/basn0g01.png) should work")]
    #[test_case("../test/png/official/tm3n3p02.png";"Decoding ../test/png/official/tm3n3p02.png) should work")]
    #[test_case("../test/png/official/g03n0g16.png";"Decoding ../test/png/official/g03n0g16.png) should work")]
    #[test_case("../test/png/official/basn2c16.png";"Decoding ../test/png/official/basn2c16.png) should work")]
    #[test_case("../test/png/official/s40n3p04.png";"Decoding ../test/png/official/s40n3p04.png) should work")]
    #[test_case("../test/png/official/basi4a08.png";"Decoding ../test/png/official/basi4a08.png) should work")]
    #[test_case("../test/png/official/cs5n3p08.png";"Decoding ../test/png/official/cs5n3p08.png) should work")]
    #[test_case("../test/png/official/s37n3p04.png";"Decoding ../test/png/official/s37n3p04.png) should work")]
    #[test_case("../test/png/official/s36n3p04.png";"Decoding ../test/png/official/s36n3p04.png) should work")]
    #[test_case("../test/png/official/s01i3p01.png";"Decoding ../test/png/official/s01i3p01.png) should work")]
    #[test_case("../test/png/official/g07n2c08.png";"Decoding ../test/png/official/g07n2c08.png) should work")]
    #[test_case("../test/png/official/basn0g02.png";"Decoding ../test/png/official/basn0g02.png) should work")]
    #[test_case("../test/png/official/basn0g16.png";"Decoding ../test/png/official/basn0g16.png) should work")]
    #[test_case("../test/png/official/tbrn2c08.png";"Decoding ../test/png/official/tbrn2c08.png) should work")]
    #[test_case("../test/png/official/tbbn0g04.png";"Decoding ../test/png/official/tbbn0g04.png) should work")]
    #[test_case("../test/png/official/g25n2c08.png";"Decoding ../test/png/official/g25n2c08.png) should work")]
    #[test_case("../test/png/official/PngSuite.png";"Decoding ../test/png/official/PngSuite.png) should work")]
    #[test_case("../test/png/official/exif2c08.png";"Decoding ../test/png/official/exif2c08.png) should work")]
    #[test_case("../test/png/official/basi6a08.png";"Decoding ../test/png/official/basi6a08.png) should work")]
    #[test_case("../test/png/official/tbwn0g16.png";"Decoding ../test/png/official/tbwn0g16.png) should work")]
    #[test_case("../test/png/official/z03n2c08.png";"Decoding ../test/png/official/z03n2c08.png) should work")]
    #[test_case("../test/png/official/s08n3p02.png";"Decoding ../test/png/official/s08n3p02.png) should work")]
    #[test_case("../test/png/official/s09n3p02.png";"Decoding ../test/png/official/s09n3p02.png) should work")]
    #[test_case("../test/png/official/pp0n6a08.png";"Decoding ../test/png/official/pp0n6a08.png) should work")]
    #[test_case("../test/png/official/cs8n2c08.png";"Decoding ../test/png/official/cs8n2c08.png) should work")]
    #[test_case("../test/png/official/f99n0g04.png";"Decoding ../test/png/official/f99n0g04.png) should work")]
    #[test_case("../test/png/official/ps2n0g08.png";"Decoding ../test/png/official/ps2n0g08.png) should work")]
    #[test_case("../test/png/official/s34i3p04.png";"Decoding ../test/png/official/s34i3p04.png) should work")]
    #[test_case("../test/png/official/s35i3p04.png";"Decoding ../test/png/official/s35i3p04.png) should work")]
    #[test_case("../test/png/official/ctzn0g04.png";"Decoding ../test/png/official/ctzn0g04.png) should work")]
    #[test_case("../test/png/official/s03n3p01.png";"Decoding ../test/png/official/s03n3p01.png) should work")]
    #[test_case("../test/png/official/s02n3p01.png";"Decoding ../test/png/official/s02n3p01.png) should work")]
    #[test_case("../test/png/official/z09n2c08.png";"Decoding ../test/png/official/z09n2c08.png) should work")]
    #[test_case("../test/png/official/g10n2c08.png";"Decoding ../test/png/official/g10n2c08.png) should work")]
    #[test_case("../test/png/official/cm7n0g04.png";"Decoding ../test/png/official/cm7n0g04.png) should work")]
    #[test_case("../test/png/official/ccwn2c08.png";"Decoding ../test/png/official/ccwn2c08.png) should work")]
    #[test_case("../test/png/official/basn0g04.png";"Decoding ../test/png/official/basn0g04.png) should work")]
    #[test_case("../test/png/official/g07n0g16.png";"Decoding ../test/png/official/g07n0g16.png) should work")]
    #[test_case("../test/png/official/basi3p01.png";"Decoding ../test/png/official/basi3p01.png) should work")]
    #[test_case("../test/png/official/basn2c08.png";"Decoding ../test/png/official/basn2c08.png) should work")]
    #[test_case("../test/png/official/oi9n2c16.png";"Decoding ../test/png/official/oi9n2c16.png) should work")]
    #[test_case("../test/png/official/basi4a16.png";"Decoding ../test/png/official/basi4a16.png) should work")]
    #[test_case("../test/png/official/s04i3p01.png";"Decoding ../test/png/official/s04i3p01.png) should work")]
    #[test_case("../test/png/official/ctjn0g04.png";"Decoding ../test/png/official/ctjn0g04.png) should work")]
    #[test_case("../test/png/official/s33n3p04.png";"Decoding ../test/png/official/s33n3p04.png) should work")]
    #[test_case("../test/png/official/s32n3p04.png";"Decoding ../test/png/official/s32n3p04.png) should work")]
    #[test_case("../test/png/official/oi9n0g16.png";"Decoding ../test/png/official/oi9n0g16.png) should work")]
    #[test_case("../test/png/official/g03n2c08.png";"Decoding ../test/png/official/g03n2c08.png) should work")]
    #[test_case("../test/png/official/basn0g08.png";"Decoding ../test/png/official/basn0g08.png) should work")]
    #[test_case("../test/png/official/basi3p02.png";"Decoding ../test/png/official/basi3p02.png) should work")]
    #[test_case("../test/png/official/tp0n2c08.png";"Decoding ../test/png/official/tp0n2c08.png) should work")]
    #[test_case("../test/png/official/bgan6a08.png";"Decoding ../test/png/official/bgan6a08.png) should work")]
    #[test_case("../test/png/official/oi2n2c16.png";"Decoding ../test/png/official/oi2n2c16.png) should work")]
    #[test_case("../test/png/official/f00n0g08.png";"Decoding ../test/png/official/f00n0g08.png) should work")]
    #[test_case("../test/png/official/f01n0g08.png";"Decoding ../test/png/official/f01n0g08.png) should work")]
    #[test_case("../test/png/official/s05i3p02.png";"Decoding ../test/png/official/s05i3p02.png) should work")]
    #[test_case("../test/png/official/oi2n0g16.png";"Decoding ../test/png/official/oi2n0g16.png) should work")]
    #[test_case("../test/png/official/f00n2c08.png";"Decoding ../test/png/official/f00n2c08.png) should work")]
    #[test_case("../test/png/official/f01n2c08.png";"Decoding ../test/png/official/f01n2c08.png) should work")]
    #[test_case("../test/png/official/s39n3p04.png";"Decoding ../test/png/official/s39n3p04.png) should work")]
    #[test_case("../test/png/official/s38n3p04.png";"Decoding ../test/png/official/s38n3p04.png) should work")]
    #[test_case("../test/png/official/tp0n0g08.png";"Decoding ../test/png/official/tp0n0g08.png) should work")]
    #[test_case("../test/png/official/tbgn2c16.png";"Decoding ../test/png/official/tbgn2c16.png) should work")]
    #[test_case("../test/png/official/cdun2c08.png";"Decoding ../test/png/official/cdun2c08.png) should work")]
    #[test_case("../test/png/official/g10n0g16.png";"Decoding ../test/png/official/g10n0g16.png) should work")]
    #[test_case("../test/png/official/ch1n3p04.png";"Decoding ../test/png/official/ch1n3p04.png) should work")]
    #[test_case("../test/png/official/ps2n2c16.png";"Decoding ../test/png/official/ps2n2c16.png) should work")]
    #[test_case("../test/png/official/basi3p04.png";"Decoding ../test/png/official/basi3p04.png) should work")]
    #[test_case("../test/png/official/cs3n2c16.png";"Decoding ../test/png/official/cs3n2c16.png) should work")]
    #[test_case("../test/png/official/s06n3p02.png";"Decoding ../test/png/official/s06n3p02.png) should work")]
    #[test_case("../test/png/official/s07n3p02.png";"Decoding ../test/png/official/s07n3p02.png) should work")]
    #[test_case("../test/png/official/g25n0g16.png";"Decoding ../test/png/official/g25n0g16.png) should work")]
    #[test_case("../test/png/official/z06n2c08.png";"Decoding ../test/png/official/z06n2c08.png) should work")]
    #[test_case("../test/png/official/basi6a16.png";"Decoding ../test/png/official/basi6a16.png) should work")]
    #[test_case("../test/png/official/cm9n0g04.png";"Decoding ../test/png/official/cm9n0g04.png) should work")]
    #[test_case("../test/png/official/cdsn2c08.png";"Decoding ../test/png/official/cdsn2c08.png) should work")]
    #[test_case("../test/png/official/tbbn3p08.png";"Decoding ../test/png/official/tbbn3p08.png) should work")]
    #[test_case("../test/png/official/ps1n0g08.png";"Decoding ../test/png/official/ps1n0g08.png) should work")]
    #[test_case("../test/png/official/basi0g02.png";"Decoding ../test/png/official/basi0g02.png) should work")]
    #[test_case("../test/png/official/basi0g16.png";"Decoding ../test/png/official/basi0g16.png) should work")]
    #[test_case("../test/png/official/basn4a08.png";"Decoding ../test/png/official/basn4a08.png) should work")]
    #[test_case("../test/png/official/s37i3p04.png";"Decoding ../test/png/official/s37i3p04.png) should work")]
    #[test_case("../test/png/official/s36i3p04.png";"Decoding ../test/png/official/s36i3p04.png) should work")]
    #[test_case("../test/png/official/s01n3p01.png";"Decoding ../test/png/official/s01n3p01.png) should work")]
    #[test_case("../test/png/official/s40i3p04.png";"Decoding ../test/png/official/s40i3p04.png) should work")]
    #[test_case("../test/png/official/g25n3p04.png";"Decoding ../test/png/official/g25n3p04.png) should work")]
    #[test_case("../test/png/official/tp1n3p08.png";"Decoding ../test/png/official/tp1n3p08.png) should work")]
    #[test_case("../test/png/official/tp0n3p08.png";"Decoding ../test/png/official/tp0n3p08.png) should work")]
    #[test_case("../test/png/official/basi2c16.png";"Decoding ../test/png/official/basi2c16.png) should work")]
    #[test_case("../test/png/official/pp0n2c16.png";"Decoding ../test/png/official/pp0n2c16.png) should work")]
    #[test_case("../test/png/official/basi0g01.png";"Decoding ../test/png/official/basi0g01.png) should work")]
    #[test_case("../test/png/official/g10n3p04.png";"Decoding ../test/png/official/g10n3p04.png) should work")]
    #[test_case("../test/png/official/basn3p08.png";"Decoding ../test/png/official/basn3p08.png) should work")]
    #[test_case("../test/png/official/z00n2c08.png";"Decoding ../test/png/official/z00n2c08.png) should work")]
    #[test_case("../test/png/official/basi0g04.png";"Decoding ../test/png/official/basi0g04.png) should work")]
    #[test_case("../test/png/official/s34n3p04.png";"Decoding ../test/png/official/s34n3p04.png) should work")]
    #[test_case("../test/png/official/s35n3p04.png";"Decoding ../test/png/official/s35n3p04.png) should work")]
    #[test_case("../test/png/official/s03i3p01.png";"Decoding ../test/png/official/s03i3p01.png) should work")]
    #[test_case("../test/png/official/s02i3p01.png";"Decoding ../test/png/official/s02i3p01.png) should work")]
    #[test_case("../test/png/official/s08i3p02.png";"Decoding ../test/png/official/s08i3p02.png) should work")]
    #[test_case("../test/png/official/s09i3p02.png";"Decoding ../test/png/official/s09i3p02.png) should work")]
    #[test_case("../test/png/official/tbwn3p08.png";"Decoding ../test/png/official/tbwn3p08.png) should work")]
    #[test_case("../test/png/official/g05n2c08.png";"Decoding ../test/png/official/g05n2c08.png) should work")]
    #[test_case("../test/png/official/g04n2c08.png";"Decoding ../test/png/official/g04n2c08.png) should work")]
    #[test_case("../test/png/official/ch2n3p08.png";"Decoding ../test/png/official/ch2n3p08.png) should work")]
    #[test_case("../test/png/official/bgai4a16.png";"Decoding ../test/png/official/bgai4a16.png) should work")]
    #[test_case("../test/png/official/cdfn2c08.png";"Decoding ../test/png/official/cdfn2c08.png) should work")]
    #[test_case("../test/png/official/basn6a08.png";"Decoding ../test/png/official/basn6a08.png) should work")]
    #[test_case("../test/png/official/g07n3p04.png";"Decoding ../test/png/official/g07n3p04.png) should work")]
    #[test_case("../test/png/official/oi4n2c16.png";"Decoding ../test/png/official/oi4n2c16.png) should work")]
    #[test_case("../test/png/official/oi4n0g16.png";"Decoding ../test/png/official/oi4n0g16.png) should work")]
    #[test_case("../test/png/official/ctfn0g04.png";"Decoding ../test/png/official/ctfn0g04.png) should work")]
    #[test_case("../test/png/official/ctgn0g04.png";"Decoding ../test/png/official/ctgn0g04.png) should work")]
    fn decoding_test(png_path: &str) {
        let byte_reader = &mut byte_reader::ByteReader::new(
            Some(&png_path),
            byte_reader::ByteReaderMode::FILE,
            None,
        );
        byte_reader.read_image();
        let mut decoder = core::decoder::PngDecoder::new(
            byte_reader,
            &core::decoder::PngDecoderOptions {
                fail_fast: false,
                validate_crc: true,
            },
        );

        let decoder_result = decoder.run().unwrap();

        let image_rs_output = image::open(png_path).unwrap();
        let image_rs_rgba8 = image_rs_output.to_rgba8();
        let image_rs_bytes = image_rs_rgba8.as_bytes();

        assert_eq!(decoder_result.len(), image_rs_bytes.len());
        assert_eq!(&decoder_result[..], &image_rs_bytes[..]);
    }

    fn decode_corrupt_image(corrupt_png_path: &str) {
        let byte_reader = &mut byte_reader::ByteReader::new(
            Some(&corrupt_png_path),
            byte_reader::ByteReaderMode::FILE,
            None,
        );
        byte_reader.read_image();
        let mut decoder = core::decoder::PngDecoder::new(
            byte_reader,
            &core::decoder::PngDecoderOptions {
                fail_fast: false,
                validate_crc: true,
            },
        );
        decoder.run();
    }

    // recoverable errors
    // signature byte 1 MSBit reset to zero
    #[test_case("../test/png/official/xs1n0g01.png";"Decoding ../test/png/official/xs1n0g01.png) should report error")]
    // signature byte 2 is a 'Q'
    #[test_case("../test/png/official/xs2n0g01.png";"Decoding ../test/png/official/xs2n0g01.png) should report error")]
    // signature byte 4 lowercase
    #[test_case("../test/png/official/xs4n0g01.png";"Decoding ../test/png/official/xs4n0g01.png) should report error")]
    // 7th byte a space instead of control-Z
    #[test_case("../test/png/official/xs7n0g01.png";"Decoding ../test/png/official/xs7n0g01.png) should report error")]
    // incorrect IHDR checksum
    #[test_case("../test/png/official/xhdn0g08.png";"Decoding ../test/png/official/xhdn0g08.png) should report error")]
    // incorrect IDAT checksum
    #[test_case("../test/png/official/xcsn0g01.png";"Decoding ../test/png/official/xcsn0g01.png) should report error")]
    // added cr bytes
    #[test_case("../test/png/official/xcrn0g04.png";"Decoding ../test/png/official/xcrn0g04.png) should report error")]
    fn decode_recoverable_corrupt_image(path: &str) {
        decode_corrupt_image(path)
    }

    // unrecoverable errors
    // color type 1
    #[test_case("../test/png/official/xc1n0g08.png";"Decoding ../test/png/official/xc1n0g08.png) should report error")]
    // color type 9
    #[test_case("../test/png/official/xc9n2c08.png";"Decoding ../test/png/official/xc9n2c08.png) should report error")]
    // bit depth 0
    #[test_case("../test/png/official/xd0n2c08.png";"Decoding ../test/png/official/xd0n2c08.png) should report error")]
    // bit depth 3
    #[test_case("../test/png/official/xd3n2c08.png";"Decoding ../test/png/official/xd3n2c08.png) should report error")]
    // bit depth 99
    #[test_case("../test/png/official/xd9n2c08.png";"Decoding ../test/png/official/xd9n2c08.png) should report error")]
    // missing IDAT/IHDR chunk
    #[test_case("../test/png/official/xdtn0g01.png";"Decoding ../test/png/official/xdtn0g01.png) should report error")]
    // added lf bytes (incorrect chunk length)
    #[test_case("../test/png/official/xlfn0g04.png";"Decoding ../test/png/official/xlfn0g04.png) should report error")]
    fn decode_unrecoverable_corrupt_image(path: &str) {
        decode_corrupt_image(path)
    }
}
