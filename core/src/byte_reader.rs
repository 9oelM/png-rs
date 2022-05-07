use std::{fs::File, io::Read, path::Path};

/// Byte reading mode. Can either be "file" or "raw"
/// `raw` means raw encoded Vec<u8> is to be decoded
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum ByteReaderMode {
    /// A PNG file.
    FILE = 0,
    /// Bytes stored in `Vec<u8>` intead of an iamge file.
    RAW = 1,
}

pub struct ByteReader<'a> {
    /// Byte reading mode. Can either be "file" or "raw"
    mode: ByteReaderMode,
    /// path to the PNG file read into the parser
    file_path: Option<&'a str>,
    /// PNG file read into the parser
    file: Option<File>,
    /// If file is None, user chooses [ByteReaderMode::RAW] option
    raw_bytes: Vec<u8>,
    /// Current read position.
    current_byte_pos: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(
        file_path: Option<&'a str>,
        mode: ByteReaderMode,
        raw_bytes: Option<Vec<u8>>,
    ) -> ByteReader<'a> {
        ByteReader {
            file_path: file_path.or_else(|| None),
            mode,
            file: None,
            raw_bytes: raw_bytes.unwrap_or(vec![]),
            current_byte_pos: 0,
        }
    }

    pub fn read_next_n_bytes(&mut self, n: u64) -> Vec<u8> {
        self.current_byte_pos += n as usize;
        match (self.mode, self.file_path) {
            (ByteReaderMode::RAW, None) => {
                let drained: Vec<u8> = self.raw_bytes.drain(0..n as usize).collect();

                return drained;
            }
            (ByteReaderMode::FILE, Some(_)) => {
                let file = self
                    .file
                    .as_ref()
                    .expect("Image file must be provided before reading bytes");

                let mut buffer: Vec<u8> = vec![];
                match &file.take(n).read_to_end(&mut buffer) {
                    Err(reason) => {
                        panic!(
                            "could not read next {} bytes: {}",
                            n, reason
                        )
                    }
                    Ok(_) => (),
                };

                return buffer;
            }
            _ => panic!("Wrong mode"),
        }
    }

    pub fn read_image(&mut self) {
        if self.mode == ByteReaderMode::RAW && self.file_path.is_none() {
        } else if self.mode == ByteReaderMode::FILE && self.file_path.is_some() {
            let file_path = self.file_path.expect("File path must exist");
            let path = Path::new(&file_path);
            let display = path.display();

            let file = match File::open(&path) {
                Err(why) => panic!(
                    "couldn't open {}: {}",
                    display, why
                ),
                Ok(file) => file,
            };

            self.file = Some(file);
        } else {
            panic!("Failed to read image with either options.");
        }
    }

    pub fn read_next_u32_num(&self, four_bytes_chunk: &Vec<u8>) -> u32 {
        return ((four_bytes_chunk[0] as u32) << 24)
            | ((four_bytes_chunk[1] as u32) << 16)
            | ((four_bytes_chunk[2] as u32) << 8)
            | four_bytes_chunk[3] as u32;
    }

    pub fn read_next_4bytes(&mut self) -> Vec<u8> {
        return self.read_next_n_bytes(4);
    }

    pub fn read_next_4bytes_num(&mut self) -> u32 {
        let next_4bytes = self.read_next_4bytes();
        return self.read_next_u32_num(&next_4bytes);
    }

    pub fn read_next_4bytes_str(&mut self) -> (String, Vec<u8>) {
        let buffer = self.read_next_4bytes();
        let str_from_buffer = match std::str::from_utf8(&buffer) {
            Err(reason) => {
                panic!(
                    "error converting next 4 bytes to a string: {}",
                    reason
                )
            }
            Ok(str_without_error) => (str_without_error),
        };

        return (
            String::from(str_from_buffer),
            buffer,
        );
    }

    pub fn get_current_byte_pos(&self) -> usize {
        self.current_byte_pos
    }
}
