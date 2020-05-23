use crate::ppu;
use crate::wram;

/// CpuBus は cpuから他のデバイスにアクセスするためのもの
pub struct CpuBus {
    wram: wram::WRAM,
    pub ppu: ppu::PPU,
    // extend_ram
    // extend_rom
    prog_rom1: Vec<u8>,
    // prog_rom2
    // pro: u8,
    // apu: u8,
    // keypad: u8,
    // dma: u8,
}

impl CpuBus {
    pub fn new(wram: wram::WRAM, ppu: ppu::PPU, prog: Vec<u8>) -> Self {
        CpuBus {
            wram,
            // pro,
            ppu,
            // apu, keypad, dma
            prog_rom1: prog,
        }
    }
    /// cpuのメモリマップから値を読み込む
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // WRAM
            0x0000..=0x07ff => {
                self.wram[addr as usize]
            }
            // unused
            0x0800..=0x1fff => {
                unimplemented!("Wram mirror")
            }
            // I/O port PPU
            0x2000..=0x2007 => {
                unimplemented!("PPU I/O port")
            }
            // unused
            0x2008..=0x3fff => {
                panic!("Unused Area")
            }
            // I/O port APU, etc
            0x4000..=0x401f => {
                unimplemented!("I/O port APU, etc")
            }
            // extended RAM
            0x4020..=0x5fff => {
                unimplemented!("extended RAM")
            }
            // battely backup RAM
            0x6000..=0x7fff => {
                unimplemented!("battely backup RAM")
            }
            // PRG ROM LOW
            0x8000..=0xbfff => {
                self.prog_rom1[(addr - 0x8000) as usize]
            }
            // PRG ROM HIGH
            0xc000..=0xffff => {
                self.prog_rom1[(addr - 0xc000) as usize]
            }
        }
    }

    /// write_by_cpuはWRAMにデータを書き込む
    /// # Return
    /// * `書き込んだ結果の値`
    pub fn write(&mut self, addr: u16, data: u8) -> u8 {
        println!("cpu:write addr: {:x}, data: {:x}", addr, data);

        match addr {
            // WRAM
            0x0000..=0x07ff => {
                self.wram[addr as usize] = data;
                data
            }
            // unused
            0x0800..=0x1fff => {
                unimplemented!("Wram mirror")
            }
            // I/O port PPU
            0x2000..=0x2007 => {
                unimplemented!("PPU I/O port")
            }
            // unused
            0x2008..=0x3fff => {
                panic!("Unused Area")
            }
            // I/O port APU, etc
            0x4000..=0x401f => {
                unimplemented!("I/O port APU, etc")
            }
            // extended RAM
            0x4020..=0x5fff => {
                unimplemented!("extended RAM")
            }
            // battely backup RAM
            0x6000..=0x7fff => {
                unimplemented!("battely backup RAM")
            }
            // PRG ROM LOW
            0x8000..=0xbfff => {
                panic!("ROM is readonly")
            }
            // PRG ROM HIGH
            0xc000..=0xffff => {
                panic!("ROM is readonly")
            }
        }
    }
}

#[test]
fn it_works() {
}