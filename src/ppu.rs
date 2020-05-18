use std::ops::Index;

pub struct PPU {
    register: Register,
}
struct Register {

}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            register: Register{ }
        }
    }
    // cyclesはppuが実行していいサイクル数
    pub fn run(&mut self, cycles: usize) {
        unimplemented!()
    }
}

/// VRAM_SIZE = 0x1fff
/// 単位はbyte
const VRAM_SIZE:usize = 0x1000;
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