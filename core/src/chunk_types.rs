#[derive(Debug, Clone)]
pub struct ChunkTypes();

impl<'a> ChunkTypes {
    pub const IHDR: &'a str = "IHDR";
    pub const IDAT: &'a str = "IDAT";
    pub const IEND: &'a str = "IEND";
    pub const PLTE: &'a str = "PLTE";
    pub const tRNS: &'a str = "tRNS";
}
