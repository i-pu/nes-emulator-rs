use crate::cpu;
use crate::screen;
use crate::screen::{INTERNAL_SIZE, SCREEN_SIZE};
use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::ops::Index;
use std::rc::{Rc, Weak};
use web_sys::console::log_1;
use itertools::izip;

pub struct PpuBus {
    pub screen: screen::Screen,
    cpu: Weak<RefCell<cpu::Cpu>>,
}

impl PpuBus {
    pub fn new(screen: screen::Screen, cpu: Weak<RefCell<cpu::Cpu>>) -> Self {
        PpuBus { screen, cpu }
    }
}

pub struct Ppu {
    pub register: Register,

    /// buffer
    /// 2回書き込見済みであればtrue
    /// 1回しか書いてないならfalse
    buffer_2006: (u16, bool),

    /// ppu bus
    pub ppu_bus: PpuBus,
    /// パレットにはNESの色ID
    /// [00VVHHHH]: u8
    /// V: 明度
    /// H: 色相
    vram: Vec<u8>,
    /// cpuの341サイクルごとに1周する
    cycles: usize,
    /// 現在何行目か
    lines: usize,
    /// フレーム数
    pub frames: usize,
}

pub struct Register {
    /// - $2000: Ppu_CTRL
    /// - VPHB SINN
    /// - NMI enable (V), Ppu master/slave (P), sprite height (H), background tile select (B), sprite tile select (S), increment mode (I), nametable select (NN)
    /// [ VPHBSINN ]: u8
    /// V: VBLANK 開始時に NMI 割り込みを発生 (0:off, 1:on)
    /// P: Ppu マスター/スレーブ (コードを読む分には気にする必要なし)
    /// H: スプライトサイズ (0:8*8, 1:8*16)
    /// B: BG パターンテーブル (0:$0000, 1:$1000)
    /// S: スプライトパターンテーブル (0:$0000, 1:$1000)
    /// I: Ppu アドレスインクリメント (0:+1, 1:+32) - VRAM 上で +1 は横方向、+32 は縦方向
    /// N: ネームテーブル (0:$2000, 1:$2400, 2:$2800, 3:$2C00)
    pub ppuctrl: u8,
    /// - $2001: Ppu_MASK
    /// - BGRs bMmG
    /// - color emphasis (BGR), sprite enable (s), background enable (b), sprite left column enable (M), background left column enable (m), greyscale (G)
    /// [ BGRsbMmG ]: u8
    /// B: 色強調(青) - 表示のみに影響すると思います。詳しくは知りません^^;
    /// G: 色強調(緑) - 同上
    /// R: 色強調(赤) - 同上
    /// s: スプライト描画 (0:off, 1:on)
    /// b: BG 描画 (0:off, 1:on)
    /// M: 画面左端 8px でスプライトクリッピング (0:有効, 1:無効)
    /// m: 画面左端 8px で BG クリッピング (0:有効, 1:無効)
    /// G: 0:カラー, 1:モノクロ
    pub ppumask: u8,
    /// - $2002: Ppu_STATUS
    /// - [ VSO..... ]
    /// - vblank (V), sprite 0 hit (S), sprite overflow (O); read resets write pair for $2005/$2006
    /// V: VBLANK フラグ
    /// S: Sprite 0 hit
    /// O: スプライトオーバーフローフラグだがバグがある。気にする必要なし
    pub ppustatus: u8,
    /// - $2003: Ppu_STATUS
    /// - aaaa aaaa
    /// - OAM read/write address
    pub oamaddr: u8,
    /// - $2004
    /// - dddd dddd
    /// - OAM data read/write
    pub oamdata: u8,
    /// - $2005
    /// - xxxx xxxx
    /// - fine scroll position (two writes: X scroll, Y scroll)
    pub ppuscroll: u8,
    /// - $2006
    /// - aaaa aaaa
    /// - Ppu read/write address (two writes: most significant byte, least significant byte)
    pub ppuaddr: u8,
    /// - $2007
    /// - dddd dddd
    /// - Ppu data read/write
    pub ppudata: u8,
}

impl Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"
        Register
        ppuctrl: {:08b}
        ppumask: {:08b}
        ppustatus: {:08b}
        oamaddr: {:08b}
        oamdata: {:08b}
        ppuscroll: {:08b}
        ppuaddr: {:08b}
        ppudata: {:08b}
        "#,
            self.ppuctrl,
            self.ppumask,
            self.ppustatus,
            self.oamaddr,
            self.oamdata,
            self.ppuscroll,
            self.ppuaddr,
            self.ppudata,
        )
    }
}

/// スプライトはSprite構造体64個の配列
pub struct Sprite {
    /// y座標, +1されて表示, 0 < y < 240
    y: u8,
    /// タイルID
    tile: u8,
    /// [VHP...CC]: u8
    /// V: 垂直反転
    /// H: 水平反転
    /// P: 優先度 (0:前面, 1:背面)
    /// CC: パレット
    attr: u8,
    /// x座標
    x: u8,
}

/// Ppuの描画要素
/// - BG: 8x8のタイルを敷き詰めた画像、スクロールは出来るが制約有り
///
/// - スプライト: 8x8 or 8x16 で最大64個
impl Ppu {
    pub fn new(screen: screen::Screen, cpu: Weak<RefCell<cpu::Cpu>>) -> Ppu {
        Ppu {
            register: Register {
                ppuctrl: 0,
                ppumask: 0,
                ppustatus: 0,
                oamaddr: 0,
                oamdata: 0,
                ppuscroll: 0,
                ppuaddr: 0,
                ppudata: 0,
            },
            ppu_bus: PpuBus::new(screen, cpu),
            vram: vec![0; 0x4000],
            cycles: 0,
            lines: 0,
            frames: 0,
            buffer_2006: (0, true),
        }
    }

    /// パターンテーブルにキャラクタROMをしまう
    pub fn load_pattern_table(&mut self, chrs: Vec<u8>) {
        assert_eq!(0x2000, chrs.len());
        for (i, c )in chrs.into_iter().enumerate() {
            self.vram[i] = c;
        }
    }

    fn blank_asseted(&mut self) -> bool {
        self.register.ppuctrl & 0b1000_0000 > 0
    }

    pub fn add_cpu(&mut self, cpu: Rc<RefCell<cpu::Cpu>>) {
        self.ppu_bus.cpu = Rc::downgrade(&cpu);
    }
    // cyclesはppuが実行していいサイクル数
    // 1 Ppu cycle で 1dot処理
    // (256, 240), 内部では(341, 262)
    //
    // line 0-239: visible scanline: 256*240を描画、CPUはPpuにアクセスすべきでない
    // line 240: post render scanline Ppuはアイドル状態、フレーム境界
    // line 241-260: vertical blanking line: CPUからアクセスを行う
    // line 241: VBLANKフラグが立ちNMI割り込みが発生
    // line 261: pre-render scanline: VBLANKフラグが降ろされる
    pub fn run(&mut self, cycles: usize) {
        // println!("line@ppu: {} cycles: {} += {}", self.lines, self.cycles, cycles);
        self.cycles += cycles;
        if self.cycles > 341 {
            self.cycles -= 341;
            self.lines += 1;
        }

        match self.lines {
            0..=239 => {
                // nothing to do
                // println!("visible scanline"),
            }
            240 => {
                // println!("post render scanline");
                // 240line = 240 * 341 cycle 目にできた画面を転送
                println!("{} frame", self.frames);
                self.frames += 1;
                let screen = self.build_screen();
                self.ppu_bus.screen.draw(screen);
            }
            241 => {
                dbg!(&self.register);
                // interrupt NMI if VBLANK is asseted
                if self.blank_asseted() {
                    let cpu = self.ppu_bus.cpu.upgrade().unwrap();
                    cpu.borrow_mut().set_nmi_flag();
                }
            }
            242..=260 => {
                // println!("vertical blanking line");
            }
            // last line
            261 => {
                // println!("pre-render scanline");
                self.lines = 0;
            }
            _ => panic!("line: {}", self.lines),
        }
    }

    // TODO: readレジスタの動作を記述する
    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0x2000 => self.register.ppuctrl,
            0x2001 => self.register.ppumask,
            0x2002 => self.register.ppustatus,
            0x2003 => self.register.oamaddr,
            0x2004 => self.register.oamdata,
            0x2005 => self.register.ppuscroll,
            0x2006 => {
                panic!("2006には読み込みがないはず");
            }
            0x2007 => {
                // TODO: PPU mem addr += 1 or += 32
                if true {
                    self.buffer_2006.0 += 0x01;
                } else {
                    self.buffer_2006.0 += 0x20;
                }

                self.register.ppudata
            }
            _ => panic!("そんなppuれじすたない{:?}", addr),
        }
    }
    // load 2006 3f // buf: [3f ??] count: 0
    // load 2006 00 // buf: [3f 00] count: 1
    // st 2006 4a // $2006:[3f 00] -> [3f 01] 3f00に4aを書きたい
    // load 2006 10 buf:[10 00] or data:[3f 02]

    // TODO: writeレジスタの動作を記述する
    pub fn write_register(&mut self, addr: u16, data: u8) -> u8 {
        match addr {
            0x2000 => {
                self.register.ppuctrl = data;
            }
            0x2001 => self.register.ppumask = data,
            0x2002 => self.register.ppustatus = data,
            0x2003 => self.register.oamaddr = data,
            0x2004 => self.register.oamdata = data,
            0x2005 => self.register.ppuscroll = data,
            0x2006 => {
                // 1st: hi: u8, 2nd: low: u8 -> u16
                if self.buffer_2006.1 {
                    // 1st
                    self.buffer_2006.0 = (data as u16) << 8;
                    self.buffer_2006.1 = !self.buffer_2006.1;
                } else {
                    // 2nd
                    self.buffer_2006.0 += data as u16;
                    self.buffer_2006.1 = !self.buffer_2006.1;
                }
            }
            0x2007 => {
                self.write_vram(self.buffer_2006.0, data);
                // TODO: レジスタ見て切り替えるようにする
                if true {
                    self.buffer_2006.0 += 0x01;
                } else {
                    self.buffer_2006.0 += 0x20;
                }
            }
            _ => panic!("そんなppuれじすたない{:?}", addr),
        }
        data
    }

    // TODO: implement
    /// vramを参照しつつnesのビットカラーでu8の2次元配列を作成する
    pub fn build_screen(&mut self) -> Vec<Vec<u8>> {
        // Color: xx GG BB RR: u8
        // $1000 から 0x10(16 byte) ブロック刻み 256個
        // ブロックは前半8byte = Low, 後半 = Highとして [hi low]: 2bit で1pixel

        let mut debug_screen: Vec<u8> = vec![];

        // TODO: 今ネームテーブル属性テーブルは0番目しか使わない
        // 1つのベクトルあたり、chrが1つはいっている
        let mut chrs: Vec<Vec<u8>> = vec![];
        // ネームテーブルを走査
        let NAME_TABLE_HEAD = 0x2000;
        for addr in NAME_TABLE_HEAD..NAME_TABLE_HEAD+0x03c0 {
            // 0x2000: NAMETABLE HEAD
            let sprite_idx = self.vram[addr] as usize;
            // 16byteとばしで ADDR
            let sprite_addr = sprite_idx * 16;
            // 下位2bitしか使わないので注意
            // 1キャラクタ8*8
            type u2 = u8;
            let mut chr: Vec<u2> = Vec::with_capacity(64);
            // low, hi = low + 8
            // FIXME: まだ背景キャラクタしか使わない
            // 行ごとに
            for j in sprite_addr..sprite_addr + 8 {
                // 0..7
                let low_line: u8 = self.vram[j as usize];
                // 8..15
                let high_line: u8 = self.vram[j as usize + 8];
                let low_line = format!("{:08b}", low_line);
                let high_line = format!("{:08b}", high_line);
                // via string
                let chr_dots = izip!(low_line.chars(), high_line.chars())
                    .map(|(l, h)| l as i32 - '0' as i32 + ((h as i32 - '0' as i32) << 1))
                    .map(|a| a as u2)
                    .collect::<Vec<u2>>();
                chr.extend(chr_dots);
            }

            if sprite_addr != 0 {
                log_1(&format!("sprite_addr: 0x{:x}, sprite_idx: {}, chr: {:?}", sprite_addr, sprite_idx, chr).into());
            }
            assert!(chr.len() == 64);

            chrs.push(chr);
        }

        // length: 16 * 16だが16*15までしかホントは使われない
        // 中に
        let mut palettes = Vec::with_capacity(16 * 16);
        for i in 0..0x40 {
            // 4つブロック分の属性が入ってる
            // 0x23c0: attr table head
            let attr4: u8 = self.vram[0x23C0 + i];
            // each 2 bits
            for (j, sht) in vec![
                (0b11_00_00_00, 6),
                (0b00_11_00_00, 4),
                (0b00_00_11_00, 2),
                (0b00_00_00_11, 0),
            ] {
                // 0x3f00: bg palette head
                // block_attr: 2bit
                let block_attr = (attr4 & j) >> sht;
                // Z型
                let palette_addr = 0x3f00
                    // どのパレット使うか
                    + match block_attr {
                        0x00 => 0,
                        0x01 => 4,
                        0x10 => 8,
                        0x11 => 0xc0,
                        _ => panic!("wtf block attr"),
                    };
                // u8(index) x 4色
                let palette: Vec<u8> = self.vram[palette_addr..palette_addr+0x04].to_vec();
                assert!(palette.len() == 4);
                palettes.push(palette);
            }
        }
        // each block(16bytes)
        // TODO: まだバックグラウンドのキャラクタROMしか使えない
        let mut pixels = vec![vec![0; SCREEN_SIZE.0]; SCREEN_SIZE.1];

        // 240
        for y in 0..SCREEN_SIZE.1 {
            // 256
            for x in 0..SCREEN_SIZE.0 {
                // chrs(1列32個:thinking_face:)のインデックス (x, y) -> i を計算
                // キャラクタが横に32個並ぶ
                // (8 x 8)なんブロック目
                // chr_x: 0..30, chr_y: 0..32
                let (chr_x, chr_y) = (x / 8, y / 8);

                let chr = &chrs[chr_y * 32 + chr_x];
                // u2型: 0-3
                let c = chr[(y % 8) * 8 + x % 8];

                let palette = &palettes[y / 16 * 16 + x / 16];
                assert!(palette.len() == 4);
                // 色をセット
                pixels[y][x] = palette[c as usize];
            }
        }
        assert!(pixels.len() * pixels[0].len() == SCREEN_SIZE.0 * SCREEN_SIZE.1);
        debug_screen.extend(palettes.clone().into_iter().flatten().collect::<Vec<u8>>());
        debug_screen.extend(self.vram.clone().into_iter());
        debug_screen.extend(vec![20; 500]);
        debug_screen.extend(self.vram.clone()[0x2000..0x23c0].into_iter());
        debug_screen.extend(vec![20; 500]);
        debug_screen.extend(self.vram.clone()[0x0..0x1000].into_iter());
        self.ppu_bus.screen.draw_debug(debug_screen);
        pixels
    }

    /// vramを読む
    pub fn read_vram(&self, addr: u16) -> u8 {
        self.vram[addr as usize]
    }

    pub fn write_vram(&mut self, addr: u16, data: u8) -> u8 {
        self.vram[addr as usize] = data;
        data
    }
}

/// VRAM_SIZE = 0x1fff
/// 単位はbyte
const VRAM_SIZE: usize = 0x1000;
struct VRAM {
    memory: [u8; VRAM_SIZE],
}

impl Index<usize> for VRAM {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[index]
    }
}

mod tests {
    #[test]
    fn hjoge() {}
}
