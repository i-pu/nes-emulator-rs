#[test]
fn it_works() {
    use super::*;
    use crate::{cpu_bus, wram, ppu};
    let wram = wram::WRAM::new();
    let ppu = ppu::Ppu::new();
    let mut cpu_bus = cpu_bus::CpuBus::new(wram, ppu, vec![]);
    let mut cpu = Cpu::new();
    cpu.run(&mut cpu_bus);
}