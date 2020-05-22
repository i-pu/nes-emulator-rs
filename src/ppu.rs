use std::ops::Index;

pub struct PPU {
    register: Register,
}
struct Register {
    /// - $2000
    /// - VPHB SINN
	/// - NMI enable (V), PPU master/slave (P), sprite height (H), background tile select (B), sprite tile select (S), increment mode (I), nametable select (NN)
    PPUCTRL: u8,
    /// - $2001
    /// - BGRs bMmG
    /// - color emphasis (BGR), sprite enable (s), background enable (b), sprite left column enable (M), background left column enable (m), greyscale (G)
    PPUMASK: u8,
    /// - $2002
    /// - VSO- ----
    /// - vblank (V), sprite 0 hit (S), sprite overflow (O); read resets write pair for $2005/$2006
    PPUSTATUS: u8,
    /// - $2003
    /// - aaaa aaaa
    /// - OAM read/write address
    OAMADDR: u8,
    /// - $2004
    /// - dddd dddd
    /// - OAM data read/write
    OAMDATA	: u8,
    /// - $2005
    /// - xxxx xxxx
    /// - fine scroll position (two writes: X scroll, Y scroll)
    PPUSCROLL: u8,
    /// - $2006
    /// - aaaa aaaa
    /// - PPU read/write address (two writes: most significant byte, least significant byte)
    PPUADDR: u8,
    /// - $2007
    /// - dddd dddd
    /// - PPU data read/write
    PPUDATA: u8,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            register: Register{
                PPUCTRL: 0,
                PPUMASK: 0,
                PPUSTATUS: 0,
                OAMADDR: 0,
                OAMDATA: 0,
                PPUSCROLL: 0,
                PPUADDR: 0,
                PPUDATA: 0,
            }
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