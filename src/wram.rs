use std::ops::{Index, IndexMut};

//
// Memory map
/*
| addr           |  description               |   mirror       |
+----------------+----------------------------+----------------+
| 0x0000-0x07FF  |  RAM                       |                |
| 0x0800-0x1FFF  |  reserve                   | 0x0000-0x07FF  |
| 0x2000-0x2007  |  I/O(PPU)                  |                |
| 0x2008-0x3FFF  |  reserve                   | 0x2000-0x2007  |
| 0x4000-0x401F  |  I/O(APU, etc)             |                |
| 0x4020-0x5FFF  |  ex RAM                    |                |
| 0x6000-0x7FFF  |  battery backup RAM        |                |
| 0x8000-0xBFFF  |  program ROM LOW           |                |
| 0xC000-0xFFFF  |  program ROM HIGH          |                |
*/

/// WRAM_SIZE = 2kib
const WRAM_SIZE: usize = 0x0800;
const WRAM_MIRROR_SIZE: usize = 0x1800;

/// # Example
/// ```
/// let wram = WRAM::new();
/// wram[0];
/// panic!("woo");
/// ```
/// wram[0x8000]
/// $0100～$01FFがスタックに相当する
/// FIXME: 今の所memoryにしかアクセスできない
pub struct WRAM {
    /// 0x0000~0x07FF
    memory: [u8; WRAM_SIZE],
    /// 0x0800~0x1FFF
    mirror_memory: [u8; WRAM_MIRROR_SIZE],
}

impl Index<usize> for WRAM {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[index]
    }
}

impl IndexMut<usize> for WRAM {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.memory[index]
    }
}

impl WRAM {
    pub fn new() -> Self {
        WRAM {
            memory: [0; WRAM_SIZE],
            mirror_memory: [0; WRAM_MIRROR_SIZE],
        }
    }
}

pub struct WRAMMirror {
    memory: [u8; WRAM_MIRROR_SIZE],
}
impl WRAMMirror {
    pub fn new() -> Self {
        WRAMMirror {
            memory: [0; WRAM_MIRROR_SIZE],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
