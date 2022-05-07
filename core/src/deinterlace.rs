use crate::common;

#[derive(Default, Clone, Copy, Debug)]
pub struct ReducedImage {
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub bytes_per_line: usize,
    pub bytes_per_pixel: usize,
}
/// Replicate this pattern over the original image.
/// Each number represents the pixel belonging to nth pass.
/// ```ignore
/// 1 6 4 6 2 6 4 6 ... (replicated on and on) ...
/// 7 7 7 7 7 7 7 7 ... (replicated on and on) ...
/// 5 6 5 6 5 6 5 6 ... (replicated on and on) ...
/// 7 7 7 7 7 7 7 7 ... (replicated on and on) ...
/// 3 6 4 6 3 6 4 6 ... (replicated on and on) ...
/// 7 7 7 7 7 7 7 7 ... (replicated on and on) ...
/// 5 6 5 6 5 6 5 6 ... (replicated on and on) ...
/// 7 7 7 7 7 7 7 7 ... (replicated on and on) ...
/// ... (replicated on and on) ... ... (replicated on and on) ...
/// ```
///
/// Let's give an example of a 1024 by 768 image (w: 1024, h: 768).
/// The first pass only includes 1 at the top left, and the entire patter is 8 pixels wide.
/// Then the first pass must have a width of 1024 / 8 = 128 pixels, meaning that there are 128 of '1's
/// in the interlaced image horizontally, in each row. It will have a height of 768 / 8 = 96 pixels, meaning
/// there are 96 of '1's in each column. So that is going to be our first reduced image (= pass).
/// in the code below, `x / 2^k` is represented as `x >> k`, and `x * 2^k` as `x << k`, and `x & 2^k` as `x & (2^k - 1)` to increase the chance of optimization (at least it won't make CPU work slower because bitwise ops are done really fast)
///
/// Note that the original image is encoded in a way that the first reduced image's information is stored first,
/// and then the second, and third and so on, in a linear manner. The order of storing the info is NOT interlaced.
/// You just need to read it sequentially with correct length of reduced image each time.
///
/// Each reduced image can have different `bytes_per_pixel` and `bytes_per_line`. The calculation logic
/// resides in [common::calc_bytes_per_pixel_and_line]. It will affect how many bytes we read for a reduced image,
/// because we only knew about the pixel width and pixel height of the reduced image, and what we need to actually
/// access is at a byte level, not a pixel.
///
/// Continuing with the previous example, if a complete reduced image were
/// `ReducedImage { pixel_width: 128, pixel_height: 96, bytes_per_line: 384, bytes_per_pixel: 3 }`,
/// We will know that we must read `384 * 96 + 96` bytes for this reduced image, because `384 * 96` bytes will be
/// the image data belonging to the first reduced image, and the remaining 96 bytes will account for
/// the filter method bytes (one byte each for filter method).
///
/// After reading the first `384 * 96 + 96` bytes from the data decompresssed by zlib, you will need
/// to read the next reduced image, and so on, until 7th reduced image.
pub fn create_reduced_images(
    pixel_width: u32,
    pixel_height: u32,
    channel: u8,
    bit_depth: u8,
) -> [ReducedImage; 7] {
    let mut reduced_images: [ReducedImage; 7] = [ReducedImage::default(); 7];

    for pass in 1..=7 {
        let (pixel_width, pixel_height) = match pass {
            1 => {
                let pass_width = (pixel_width + 7) >> 3;
                let pass_height = (pixel_height + 7) >> 3;
                (pass_width, pass_height)
            }
            2 => {
                let pass_width = (pixel_width >> 3) + ((pixel_width & 7) / 5);
                let pass_height = (pixel_height + 7) >> 3;
                (pass_width, pass_height)
            }
            3 => {
                let pass_width = ((pixel_width >> 3) << 1) + (((pixel_width & 7) + 3) >> 2);
                let pass_height = (pixel_height >> 3) + ((pixel_height & 7) / 5);
                (pass_width, pass_height)
            }
            4 => {
                let pass_width = ((pixel_width >> 3) << 1) + (((pixel_width & 7) + 1) >> 2);
                let pass_height = (pixel_height + 3) >> 2;
                (pass_width, pass_height)
            }
            5 => {
                let pass_width = (pixel_width >> 1) + (pixel_width & 1);
                let pass_height = ((pixel_height >> 3) << 1) + (((pixel_height & 7) + 1) >> 2);
                (pass_width, pass_height)
            }
            6 => {
                let pass_width = pixel_width >> 1;
                let pass_height = (pixel_height >> 1) + (pixel_height & 1);
                (pass_width, pass_height)
            }
            7 => {
                let pass_width = pixel_width;
                let pass_height = pixel_height >> 1;
                (pass_width, pass_height)
            }
            _ => (0, 0),
        };
        let (bytes_per_pixel, bytes_per_line) = common::calc_bytes_per_pixel_and_line(
            channel,
            bit_depth,
            pixel_width,
        );
        reduced_images[pass - 1] = ReducedImage {
            pixel_width,
            pixel_height,
            bytes_per_pixel,
            bytes_per_line,
        };
    }

    return reduced_images;
}

/// Calculates the pixel index to which 4-bytes long RGBA data corresponding to a single pixel
/// will be started to be inserted from, given that the original image is interlaced.
/// 
/// Do not use this for non-interlaced images. You can't.
/// 
/// For example, if the return value is 20, 
/// `r` will be inserted to index 20,
/// `g` will be inserted to index 21,
/// `b` will be inserted to index 22,
///  and `a` will be inserted to index 23.
/// 
/// Here's the logic behind the calculation, with an example.
/// Let `original_image_pixel_width = 32`.
/// If it is the first reduced image, it `nth_pass` must be 1.
/// And say, you are at `nth_col = 1` and `nth_row = 2`.
/// Then `x = nth_col * 8 = 1 * 8 = 8`, and `y = nth_row * 8 = 2 * 8 = 16`.
/// 
/// Then `output_index = 16 * 32 * 4 + 8 * 4 = 2080`. So your RGBA data will accommodate 2080th, 2081th, 2082th, and 2083th indices in the output vector.
/// 
/// We multiply `4` at the end because each pixel will account for 4 bytes in all cases.
/// 
/// Essentially, this is undoing the work from `create_reduced_images`. As long as you are iterating through all reduced images correctly in the pass order of 1 to 7, you will be able to fill all parts of the output vector without gaps in between.
/// 
/// Also, remember to ignore index (output from this function) that is bigger than the length of output vector. This can happen when there are multiple pixels per byte, where some low-order bits of the last byte of a scanline may go unused (The contents of these unused bits are not specified)
/// 
pub fn calc_interlaced_pixel_index(
    nth_col: usize,
    nth_row: usize,
    nth_pass: u8,
    original_image_pixel_width: u32,
) -> usize {
    let (x, y) = match nth_pass {
        1 => (nth_col * 8, nth_row * 8),
        2 => (nth_col * 8 + 4, nth_row * 8),
        3 => (nth_col * 4, nth_row * 8 + 4),
        4 => (nth_col * 4 + 2, nth_row * 4),
        5 => (nth_col * 2, nth_row * 4 + 2),
        6 => (nth_col * 2 + 1, nth_row * 2),
        7 => (nth_col, nth_row * 2 + 1),
        _ => panic!("nth_pass must be between 1 and 7"),
    };

    let output_index = (y as u64 * (original_image_pixel_width as u64) << 2) + ((x as u64) << 2);
    let output_index: usize = output_index
        .try_into()
        .expect("Output index does not fit in usize");

    return output_index;
}
