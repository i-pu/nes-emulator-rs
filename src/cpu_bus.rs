use crate::ppu;
use crate::wram;
use std::cell::RefCell;
use std::rc::Rc;

/// CpuBus は cpuから他のデバイスにアクセスするためのもの
pub struct CpuBus {
    wram: wram::WRAM,
    pub ppu: Rc<RefCell<ppu::Ppu>>,
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
    pub fn new(wram: wram::WRAM, ppu: Rc<RefCell<ppu::Ppu>>, prog: Vec<u8>) -> Self {
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
            0x0000..=0x07ff => self.wram[addr as usize],
            // WRAM mirror
            0x0800..=0x1fff => {
                self.wram[(addr % 0x0800) as usize]
            }
            // I/O port Ppu
            addr @ 0x2000..=0x3fff => {
                let mut ppu = self.ppu.borrow_mut();

                ppu.read_register((addr % 8) + 0x2000)
            }
            // I/O port APU, etc
            0x4000..=0x401f => unimplemented!("I/O port APU, etc"),
            // extended RAM
            0x4020..=0x5fff => unimplemented!("extended RAM"),
            // battely backup RAM
            0x6000..=0x7fff => unimplemented!("battely backup RAM"),
            // PRG ROM LOW & HIGH
            0x8000..=0xffff => self.prog_rom1[(addr - 0x8000) as usize],
            // FIXME: ホントはHIGHとLOWに別れてるので変かもしれない
        }
    }

    /// write_by_cpuはWRAMにデータを書き込む
    /// # Return
    /// * `書き込んだ結果の値`
    pub fn write(&mut self, addr: u16, data: u8) -> u8 {
        // println!("cpu:write addr: {:x}, data: {:x}", addr, data);

        match addr {
            // WRAM
            0x0000..=0x07ff => {
                self.wram[addr as usize] = data;
                data
            }
            // WRAM mirror
            0x0800..=0x1fff => {
                self.wram[(addr % 0x800) as usize] = data;
                data
            }
            // I/O port Ppu
            addr@0x2000..=0x3fff => {
                let mut ppu = self.ppu.borrow_mut();
                ppu.write_register((addr % 8) + 0x2000, data)
            }
            // I/O port APU, etc
            0x4000..=0x401f => unimplemented!("I/O port APU, etc"),
            // extended RAM
            0x4020..=0x5fff => unimplemented!("extended RAM"),
            // battely backup RAM
            0x6000..=0x7fff => unimplemented!("battely backup RAM"),
            // PRG ROM LOW
            0x8000..=0xbfff => panic!("ROM is readonly"),
            // PRG ROM HIGH
            0xc000..=0xffff => panic!("ROM is readonly"),
        }
    }
}

#[test]
fn it_works() {}
