use std::ops::Index;

/// WRAM_SIZE = 32kib
const WRAM_SIZE: usize = 0xFFFF;

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
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
