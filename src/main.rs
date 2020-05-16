mod nes;
mod cpu;
mod cpu_bus;
mod ppu;
mod wram;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO:
    // 0-3: 4e45531a
    // 4: PRG = program ROM units 16KB
    // 5: CHR = character ROM units 8KB
    //    - sprite images
    // TODO: slice PRG and CHR
    // register: 8bit, PC: 16bit
    // let nes_header = 0x4e45531a;

    let cassette = "./sample1/sample1.nes";
    let nes: nes::NES = nes::NES::new(cassette)?;
    nes.run();
    Ok(())
}

// (16 * 16) * 8
// 0x800
// 2048 == 2 ** 11 == 2kib
// 1kib == 2 ** 10
// 2kib == 2 * 2 ** 10
// 1Mib == 2 ** 20
// 2Mib == 2 ** 21


// [0x01] 01110010
// [0x02] 01100101

// 0xA5 -> LDA, zpg
//   LDA: load to register
//   zpg: PC & 0x0011

// === CPU cycles ===
// 1. fetch opcode from PC
// 2. check op and adressing mode
// 3. fetch operands
// 4. calc address
// 5. execute op