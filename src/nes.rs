use std::io::prelude::*;
use std::fs::File;
use crate::cpu;
use crate::wram;
use crate::cpu_bus;
use crate::ppu;

const NES_HEADER_SIZE: usize = 0x0010;
const PROGRAM_ROM_SIZE: usize = 0x4000;
const CHARACTER_ROM_SIZE: usize = 0x2000;

pub struct NES {
    cpu: cpu::Cpu,
    cpu_bus: cpu_bus::CpuBus,
}

/// CPUのクロック数の管理やppuのクロック数の管理をする
impl NES {
    pub fn new(file: &str) ->  Result<Self, Box<dyn std::error::Error>> {


        let mut f = File::open(file)?;
        let mut program: Vec<u8> = Vec::new();
        f.read_to_end(&mut program)?;
        let (prog, _) = NES::parse(program).unwrap();

        dbg!(&prog[..20]);


        let cpu_bus = {
            // すべてのデバイスの初期化

            // wramの初期化
            let wram = wram::WRAM::new();

            // ppuの初期化
            let ppu = ppu::PPU::new();

            cpu_bus::CpuBus::new(wram, ppu, prog)
        };

        Ok(NES {
            cpu: cpu::Cpu::new(),
            cpu_bus: cpu_bus,
        })
    }

    pub fn run(mut self) {
        let hz = 10u32; // ヘルツ
        loop {
            // cycles: cpuが何サイクル回ったか
            let mut cycles: usize = 0;

            // cpu実行
            cycles += self.cpu.run(&mut self.cpu_bus) as usize;
            // NOT IMPLEMENTED
            // self.cpu_bus.ppu.run(cycles * 3);
            // 1ナノ秒 = 0.000 000 001 秒
            std::thread::sleep(std::time::Duration::new(0, 1_000_000_000 / hz));
        }
    }

    /// nesのバイナリをprogramROMとcharactorROMにパースする
    fn parse(binary: Vec<u8>) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
        if !(&binary[0..4] == &[0x4e, 0x45, 0x53, 0x1a]) {
            // FIXME: errorを返すようにする
            panic!("not nes file");
        }
        let program_rom_page = binary[4];
        // TODO: logの方法を考える
        // TODO: debugの仕方を考える
        // TODO: ちゃんとぱーす考える
        //   log.info('prom pages =', programROMPages);
        let character_rom_page = binary[5];
        //   log.info('crom pages =', characterROMPages);
        //   const isHorizontalMirror = !(nes[6] & 0x01);
        //   const mapper = (((nes[6] & 0xF0) >> 4) | nes[7] & 0xF0);
        //   log.info('mapper', mapper);
        let character_rom_start = NES_HEADER_SIZE + program_rom_page as usize * PROGRAM_ROM_SIZE;
        let  character_rom_end = character_rom_start + character_rom_page as usize * CHARACTER_ROM_SIZE;
        Ok((binary[NES_HEADER_SIZE..character_rom_start].to_vec(),  binary[character_rom_start..character_rom_end].to_vec()))
    }
}

#[cfg(test)]
mod tests {

}