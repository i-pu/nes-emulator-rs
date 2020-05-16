use crate::wram;
use crate::ppu;

pub struct CpuBus {
    wram: wram::WRAM,

    // ram: u8,
    // pro: u8,
    // ppu: u8,
    // apu: u8,
    // keypad: u8,
    // dma: u8,
}

impl CpuBus {
    fn new(&mut self, ram: u8, pro: u8, ppu: u8, apu: u8, keypad: u8, dma: u8) -> Self {
        CpuBus {
            wram: wram::WRAM::new(),
            // ram, pro, ppu, apu, keypad, dma
        }
    }
    pub fn read(&self, addr: u16) -> u8 {
        0x01
    }

    pub fn write(&mut self, data: u8) {}
}