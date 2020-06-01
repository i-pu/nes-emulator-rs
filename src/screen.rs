
/// screen size(width, height)
pub const SCREEN_SIZE: (usize, usize) = (256, 240);
pub const INTERNAL_SIZE: (usize, usize) = (341, 262);

pub struct Screen {
    /// 画面
    pub screen: Vec<u8>
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            screen: vec![0u8; SCREEN_SIZE.0 * SCREEN_SIZE.1 * 4],
        }
    }

    /// draw screen
    /// u6すなわち64個のうちのどれかの色を指定する
    pub fn draw(&mut self, pixels: Vec<Vec<u8>>) {
        print!("draw completed");
        self.screen = self.convert_screen_to_image(pixels);
    }

    pub fn convert_screen_to_image(&self, pixels: Vec<Vec<u8>>) -> Vec<u8> {
        assert!(pixels.len() == SCREEN_SIZE.1 && pixels[0].len() == SCREEN_SIZE.0);

        let mut data = Vec::new();

        // TODO: color mapping
        for y in 0..SCREEN_SIZE.1 {
            for x in 0..SCREEN_SIZE.0 {
                // R
                data.push(pixels[y][x] & 0b00110000);
                // G
                data.push(pixels[y][x] & 0b00001100);
                // B
                data.push(pixels[y][x] & 0b00000011);
                // A
                data.push(255);
            }
        }

        data
    }
}