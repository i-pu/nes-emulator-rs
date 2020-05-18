use std::io::prelude::*;
use std::fs::File;
use crate::cpu;
use crate::wram;
use crate::cpu_bus;
use crate::ppu;

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
        assert_eq!(&program[0..4], &[0x4e, 0x45, 0x53, 0x1a]);


        let mut cpu_bus = {
            // すべてのデバイスの初期化

            // wramの初期化
            let mut wram = wram::WRAM::new();

            // 間違い // wram.load_program(program);

            // ppuの初期化
            let mut ppu = ppu::PPU::new();

            cpu_bus::CpuBus::new(wram, ppu)
        };

        Ok(NES {
            cpu: cpu::Cpu::new(),
            cpu_bus: cpu_bus,
        })
    }

    pub fn run(mut self) {
        loop {
            // cycles: cpuが何サイクル回ったか
            let mut cycles: usize = 0;

            // cpu実行
            cycles += self.cpu.run(&mut self.cpu_bus) as usize;
            self.cpu_bus.ppu.run(cycles * 3);
            break;
        }
    }

    fn parse_header(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {

}