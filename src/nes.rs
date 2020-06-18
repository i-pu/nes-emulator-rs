use crate::cpu;
use crate::cpu_bus;
use crate::ppu;
use crate::screen;
use crate::wram;
use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::rc::{Rc, Weak};

const NES_HEADER_SIZE: usize = 0x0010;
const PROGRAM_ROM_SIZE: usize = 0x4000;
const CHARACTER_ROM_SIZE: usize = 0x2000;

use web_sys;

pub struct NES {
    cpu: Rc<RefCell<cpu::Cpu>>,
    pub ppu: Rc<RefCell<ppu::Ppu>>,
}

/// CPUのクロック数の管理やppuのクロック数の管理をする
impl NES {
    pub fn new(file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut f = File::open(file)?;
        let mut program: Vec<u8> = Vec::new();
        f.read_to_end(&mut program)?;
        let (prog, _) = NES::parse(program).unwrap();

        dbg!(&prog[..20]);

        // wramの初期化
        let wram = wram::WRAM::new();
        // ppuの初期化
        let screen = screen::Screen::new();
        let ppu = Rc::new(RefCell::new(ppu::Ppu::new(screen, Weak::new())));

        let cpu_bus = cpu_bus::CpuBus::new(wram, ppu.clone(), prog);
        let mut cpu = Rc::new(RefCell::new(cpu::Cpu::new(cpu_bus)));
        ppu.borrow_mut().add_cpu(cpu.clone());

        Ok(NES { cpu, ppu })
    }

    pub fn load(program: Vec<u8>) -> Self {
        let (prog, chrs) = NES::parse(program).unwrap();

        dbg!(&prog[..20]);

        // wramの初期化
        let wram = wram::WRAM::new();
        // ppuの初期化
        let screen = screen::Screen::new();
        let ppu = Rc::new(RefCell::new(ppu::Ppu::new(screen, Weak::new())));

        let cpu_bus = cpu_bus::CpuBus::new(wram, ppu.clone(), prog);
        let mut cpu = Rc::new(RefCell::new(cpu::Cpu::new(cpu_bus)));
        ppu.borrow_mut().add_cpu(cpu.clone());
        ppu.borrow_mut().load_pattern_table(chrs);

        NES { cpu, ppu }
    }

    /// # next
    /// nesをcpuの1命令ごとにすすめる
    /// # Return
    /// cpuが何サイクル使ったか
    pub fn next(&mut self) -> usize {
        // cycles: cpuが何サイクル回ったか
        let mut cycles: usize = 0;
        cycles += self.cpu.borrow_mut().run() as usize;
        self.ppu.borrow_mut().run(3 * cycles);
        return cycles;
    }

    pub fn run(mut self) {
        // 60fps
        // let hz = 1_790_000u32;
        // 1fps
        let hz = 179_000u32 / 6;
        // let hz = 30u32; // ヘルツ
        loop {
            // cycles: cpuが何サイクル回ったか
            let mut cycles: usize = 0;

            // cpu実行
            cycles += self.cpu.borrow_mut().run() as usize;
            // ppu 実行
            self.ppu.borrow_mut().run(3 * cycles);
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
        // TODO: ちゃんとぱーす考える
        //   log.info('prom pages =', programROMPages);
        let character_rom_page = binary[5];
        //   log.info('crom pages =', characterROMPages);
        //   const isHorizontalMirror = !(nes[6] & 0x01);
        //   const mapper = (((nes[6] & 0xF0) >> 4) | nes[7] & 0xF0);
        //   log.info('mapper', mapper);
        let character_rom_start = NES_HEADER_SIZE + program_rom_page as usize * PROGRAM_ROM_SIZE;
        let character_rom_end = character_rom_start + character_rom_page as usize * CHARACTER_ROM_SIZE;
        let prog = binary[NES_HEADER_SIZE..character_rom_start].to_vec();
        // TODO: progの長さが0x8000以下だと割り込みベクタがかけることになるのでおかしくなるかも
        Ok((
            prog,
            binary[character_rom_start..character_rom_end].to_vec(),
        ))
    }
}

#[cfg(test)]
mod tests {}
