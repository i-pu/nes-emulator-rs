use crate::ppu;
use crate::wram;

/// CpuBus は cpuから他のデバイスにアクセスするためのもの
pub struct CpuBus {
    // TODO:それぞれの変数はu8ではなく構造体
    wram: wram::WRAM,
    // extend_ram
    // extend_rom
    // prog_rom1
    // prog_rom2
    // pro: u8,
    pub ppu: ppu::PPU,
    // apu: u8,
    // keypad: u8,
    // dma: u8,
}

impl CpuBus {
    pub fn new(wram: wram::WRAM, ppu: ppu::PPU) -> Self {
        CpuBus {
            wram,
            // pro,
            ppu,
            // apu, keypad, dma
        }
    }
    /// cpuのメモリマップから値を読み込む
    pub fn read(&self, addr: u16) -> u8 {
        unimplemented!();
        //if addr < 0x0800 {
        //    return self.ram.read(addr);
        //} else if addr < 0x2000 {
        //    // mirror
        //    return self.ram.read(addr - 0x0800);
        //} else if addr < 0x4000 {
        //    // mirror
        //    const data = self.ppu.read((addr - 0x2000) % 8);
        //    return data;
        //} else if addr == 0x4016 {
        //    // TODO Add 2P
        //    return +this.keypad.read();
        //} else if addr >= 0xC000 {
        //    // Mirror, if prom block number equals 1
        //    if this.programROM.size <= 0x4000 {
        //        return this.programROM.read(addr - 0xC000);
        //    }
        //    return this.programROM.read(addr - 0x8000);
        //} else if addr >= 0x8000 {
        //    // ROM
        //    return this.programROM.read(addr - 0x8000);
        //} else {
        //    return 0;
        //}
    }

    /// write_by_cpuはWRAMにデータを書き込む
    /// # Return
    /// * `書き込んだ結果の値`
    pub fn write(&mut self, addr: u16, data: u8) -> u8 {

        // // log.debug(`cpu:write addr = ${addr}`, data);
        // if (addr < 0x0800) {
        // // RAM
        //     this.ram.write(addr, data);
        // } else if (addr < 0x2000) {
        //     // mirror
        //     this.ram.write(addr - 0x0800, data);
        // } else if (addr < 0x2008) {
        //     // PPU
        //     this.ppu.write(addr - 0x2000, data);
        // } else if (addr >= 0x4000 && addr < 0x4020) {
        //     if (addr === 0x4014) {
        //         this.dma.write(data);
        //     } else if (addr === 0x4016) {
        //         // TODO Add 2P
        //         this.keypad.write(data);
        //     } else {
        //         // APU
        //         this.apu.write(addr - 0x4000, data);
        //     }
        // }
        unimplemented!();
    }
}

#[test]
fn it_works() {
}