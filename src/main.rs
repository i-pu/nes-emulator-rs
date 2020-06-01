use nes_emulator_rs::{nes};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cassette = "./sample1/sample1.nes";
    let mut nes: nes::NES = nes::NES::new(cassette)?;
    // nes.run();
    loop {
      nes.next();
    }
    Ok(())
}