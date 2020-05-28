use std::fmt::{self, Debug};
use std::ops::Index;
use crate::screen;
use crate::screen::{SCREEN_SIZE, INTERNAL_SIZE};
use crate::cpu;
use crate::cpu::op;
use crate::cpu_bus;
use std::rc::{Weak, Rc};
use std::cell::RefCell;

pub struct PpuBus {
    screen: screen::Screen,
    cpu: Weak<RefCell<cpu::Cpu>>,
}

impl PpuBus {
    pub fn new(screen: screen::Screen, cpu: Weak<RefCell<cpu::Cpu>>) -> Self {
        PpuBus { screen, cpu }
    }
}

pub struct Ppu {
    pub register: Register,

    /// internal screen
    screen: Vec<Vec<u8>>,

    /// ppu bus
    ppu_bus: PpuBus,
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
    frames: usize,
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
    pub oamdata	: u8,
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
        write!(f, r#"
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
            register: Register{
                ppuctrl: 0,
                ppumask: 0,
                ppustatus: 0,
                oamaddr: 0,
                oamdata: 0,
                ppuscroll: 0,
                ppuaddr: 0,
                ppudata: 0,
            },
            screen: vec![vec![0u8; SCREEN_SIZE.0]; SCREEN_SIZE.1],
            ppu_bus: PpuBus::new(screen, cpu),
            vram: vec![0; 0x4000],
            cycles: 0,
            lines: 0,
            frames: 0,
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
                let s = vec![vec![]];
                self.ppu_bus.screen.draw(s);
            }
            241 => {
                dbg!(&self.register);
                // interrupt NMI if VBLANK is asseted
                if self.blank_asseted() {
                    let mut cpu = self.ppu_bus.cpu.upgrade().unwrap();
                    cpu.borrow_mut().set_nmi_flag();
                }
            }
            242..=260 => {
                if self.lines % 8 == 0 {
                    self.build_background();
                    // println!("vertical blanking line");
                }
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
    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0x2000 => self.register.ppuctrl,
            0x2001 => self.register.ppumask,
            0x2002 => self.register.ppustatus,
            0x2003 => self.register.oamaddr,
            0x2004 => self.register.oamdata,
            0x2005 => self.register.ppuscroll,
            0x2006 => self.register.ppuaddr,
            0x2007 => self.register.ppudata,
            _ => panic!("そんなppuれじすたない{:?}", addr)
        }
    }

    // TODO: implement
    pub fn build_background(&mut self) {
        // todo!("implement");
    }

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
                // if self.write_addr.len() == 2 {
                //     let addr = self.write_addr;
                //     self.vram[addr] = data;
                // } else {
                //     self.write_addr.push(data);
                // }
                // self.register.ppuaddr = data;
            }
            0x2007 => self.register.ppudata = data,
            _ => panic!("そんなppuれじすたない{:?}", addr)
        }
        data
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
const VRAM_SIZE:usize = 0x1000;
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
    fn hjoge() {

    }
}