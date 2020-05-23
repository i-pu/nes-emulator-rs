use std::ops::Index;

pub struct PPU {
    register: Register,
}
struct Register {
    /// - $2000
    /// - VPHB SINN
	/// - NMI enable (V), PPU master/slave (P), sprite height (H), background tile select (B), sprite tile select (S), increment mode (I), nametable select (NN)
    ppuctrl: u8,
    /// - $2001
    /// - BGRs bMmG
    /// - color emphasis (BGR), sprite enable (s), background enable (b), sprite left column enable (M), background left column enable (m), greyscale (G)
    ppumask: u8,
    /// - $2002
    /// - VSO- ----
    /// - vblank (V), sprite 0 hit (S), sprite overflow (O); read resets write pair for $2005/$2006
    ppustatus: u8,
    /// - $2003
    /// - aaaa aaaa
    /// - OAM read/write address
    oamaddr: u8,
    /// - $2004
    /// - dddd dddd
    /// - OAM data read/write
    oamdata	: u8,
    /// - $2005
    /// - xxxx xxxx
    /// - fine scroll position (two writes: X scroll, Y scroll)
    ppuscroll: u8,
    /// - $2006
    /// - aaaa aaaa
    /// - PPU read/write address (two writes: most significant byte, least significant byte)
    ppuaddr: u8,
    /// - $2007
    /// - dddd dddd
    /// - PPU data read/write
    ppudata: u8,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            register: Register{
                ppuctrl: 0,
                ppumask: 0,
                ppustatus: 0,
                oamaddr: 0,
                oamdata: 0,
                ppuscroll: 0,
                ppuaddr: 0,
                ppudata: 0,
            }
        }
    }
    // cyclesはppuが実行していいサイクル数
    pub fn run(&mut self, cycles: usize) {
        let mut cycles = cycles;
        unimplemented!("dont run");
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