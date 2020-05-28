
// screen size(width, height)
pub const SCREEN_SIZE: (usize, usize) = (256, 240);
pub const INTERNAL_SIZE: (usize, usize) = (341, 262);

pub struct Screen {
    /// 画面
    screen: Vec<Vec<u8>>
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            screen: vec![vec![0u8; SCREEN_SIZE.0]; SCREEN_SIZE.1],
        }
    }

    /// draw screen
    /// u6すなわち64個のうちのどれかの色を指定する
    pub fn draw(&mut self, pixels: Vec<Vec<u8>>) {
        print!("draw completed");
        self.screen = pixels;
    }
}