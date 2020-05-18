use std::ops::Index;

/// WRAM_SIZE = 2kib
const WRAM_SIZE: usize = 0x1000;
// $0100～$01FFがスタックに相当する

/// # Example
/// ```
/// let wram = WRAM::new();
/// wram[0];
/// panic!("woo");
/// ```
/// wram[0x8000]
pub struct WRAM {
    memory: [u8; WRAM_SIZE],
}

impl Index<usize> for WRAM {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[index]
    }
}

impl WRAM {
    pub fn new() -> Self {
        WRAM {
            memory: [0;WRAM_SIZE],
        }
    }
    pub fn load_program(&mut self, program: Vec<u8>) {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
