use std::ops::Index;

/// VRAM_SIZE = 0x3fff
/// 単位はbyte
const VRAM_SIZE:usize = 0x3fff;
struct VRAM {
    memory: [u8; VRAM_SIZE],
}

impl Index<usize> for VRAM {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[index]
    }
}
mod tests {
    #[test]
    fn hjoge() {

    }
}