use std::io::prelude::*;
use std::fs::File;
pub struct NES {
}

impl NES {
    pub fn new(file: &str) ->  Result<Self, Box<dyn std::error::Error>> {
        let mut f = File::open(file)?;
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)?;
        assert_eq!(&buffer[0..4], &[0x4e, 0x45, 0x53, 0x1a]);
        Ok(NES {})
    }

    pub fn run(self) {
    }

    fn parse_header(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {

}