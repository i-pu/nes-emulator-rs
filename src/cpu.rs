mod op;
use crate::cpu_bus;
struct Register {
    A: u8,	                // 8bit	アキュームレータ	汎用演算
    X: u8,	                // 8bit	インデックスレジスタ	アドレッシング、カウンタなど
    Y: u8,	                // 8bit	インデックスレジスタ	アドレッシング、カウンタなど
    S: u8,	                // 8bit	スタックポインタ	スタックの位置を保持
    P: StatusRegister,	    // 8bit	ステータスレジスタ	CPUの各種状態を保持
    PC: u16,                // 16bit	プログラムカウンタ	実行している位置を保持
}

impl Register {
    pub fn new() -> Self {
        Self {
            A: 0x00 as u8,
            X: 0x00 as u8,
            Y: 0x00 as u8,
            S: 0x00 as u8,
            P: StatusRegister::new(),
            PC: 0x0000 as u16,
        }
    }
}

/// # ステータス・レジスタ
///
/// ステータスレジスタの詳細です。bit5は常に1で、bit3はNESでは未実装です。
/// IRQは割り込み、BRKはソフトウエア割り込みです。
struct StatusRegister {
	negative: bool, 	    // 7 N ネガティブ	Aの7ビット目と同じになります。負数の判定用。
	overflow: bool, 	    // 6 V オーバーフロー	演算がオーバーフローを起こした場合セットされます。
	reserved: bool, 	    // 5 R 予約済み	使用できません。常にセットされています。
	breakm: bool, 	        // 4 B ブレークモード	BRK発生時はセットされ、IRQ発生時はクリアされます。
	decimal: bool, 	        // 3 D デシマルモード	セットすると、BCDモードで動作します。(ファミコンでは未実装)
	interrupt: bool, 	    // 2 I IRQ禁止	クリアするとIRQが許可され、セットするとIRQが禁止になります。
	zero: bool, 	        // 1 Z ゼロ	演算結果が0になった場合セットされます。ロード命令でも変化します。
	carry: bool, 	        // 0 C キャリー	キャリー発生時セットされます。
}

impl StatusRegister {
    pub fn new() -> Self {
        Self {
            negative: false,
            overflow: false,
            reserved: true,
            breakm: true,
            decimal: false,
            interrupt: true,
            zero: false,
            carry: false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Operand {
    None,
    Byte(u8),
    Word(u16),
}

pub struct Cpu {
    /// # 実装イメージ
    /// CPUは基本的には以下の手順を繰り返します
    /// 1. PC（プログラムカウンタ）からオペコードをフェッチ（PCをインクリメント）
    /// 2. 命令とアドレッシング・モードを判別
    /// 3.（必要であれば）オペランドをフェッチ（PCをインクリメント）
    /// 4.（必要であれば）演算対象となるアドレスを算出
    /// 5. 命令を実行
    /// 6. 1に戻る
    register: Register,
}

impl Cpu {
    /// レジスタの初期化
    pub fn new() -> Cpu {
        Cpu {
            register: Register::new(),
        }
    }

    /// CPUの実行
    /// 実行タイミング調整のために実行にかかったサイクル数を返す
    pub fn run(&mut self, cpu_bus: &mut cpu_bus::CpuBus) -> u8 {
        let opcode = self.fetch(cpu_bus);
        let instruction = op::decode_op(opcode);
        let operand = self.fetch_operand(instruction.1, cpu_bus);
        self.exec(instruction, operand, cpu_bus)
    }

    /// cpu_busからbyteデータをfetchするレジスタとプラグラムカウンタを上げる
    fn fetch(&mut self, cpu_bus: &mut cpu_bus::CpuBus) -> u8 {
        let byte = cpu_bus.read(self.register.PC);
        self.register.PC += 1;
        byte
    }

    /// レジスタの初期化
    fn reset(&mut self) -> () {
        self.register = Register::new();
    }

    /// fetch_operandはアドレッシングモードからアドレスを返す
    /// アドレスを返さない場合があるのでそのときはNoneを返す
    /// 返り値のu16はほとんどがu8で済むが一部のアドレッシングモードにおいてu16を返す必要があります
    fn fetch_operand(&mut self, mode: op::AddressingMode, cpu_bus: &mut cpu_bus::CpuBus) -> Operand {

        // TODO: テストをしよう
        // TODO: u8とu16の変換が問題ないかを確認する
        match mode {
            op::AddressingMode::Accumulator => Operand::None,
            op::AddressingMode::Implied => Operand::None,
            op::AddressingMode::Immediate => Operand::Byte(self.fetch(cpu_bus)),
            op::AddressingMode::Zeropage => Operand::Byte(self.fetch(cpu_bus)),
            op::AddressingMode::ZeropageX => {
                let addr = self.fetch(cpu_bus);
                Operand::Byte(addr.wrapping_add(self.register.X))
            },
            op::AddressingMode::ZeropageY => {
                let addr = self.fetch(cpu_bus);
                Operand::Byte(addr.wrapping_add(self.register.Y))
            },
            op::AddressingMode::IndexedIndirect => {
                let base_addr: u16 = self.fetch(cpu_bus).wrapping_add(self.register.X) as u16;
                let addr: u16 = cpu_bus.read(base_addr) as u16 + (cpu_bus.read((base_addr + 1) & 0x00ff) as u16 ) << 8;
                Operand::Word(addr)
            },
            op::AddressingMode::IndirectIndexed => {
                let addr_or_data: u16 = self.fetch(cpu_bus) as u16;
                // 0ページ内での操作だと思うので、& 0x00ffはキャリーを無視させるため
                let base_addr: u16 = cpu_bus.read(addr_or_data) as u16 + (cpu_bus.read((addr_or_data + 1) & 0x00ff) as u16) << 8;
                let addr = base_addr + self.register.Y as u16;
                Operand::Word(addr)
            },
            op::AddressingMode::AbsoluteIndirect => {
                // 0ページ内での操作だと思うので、& 0x00ffはキャリーを無視させるため
                let addr: u16 = self.fetch(cpu_bus) as u16 + (self.fetch(cpu_bus) as u16) << 8;
                let data: u16 = cpu_bus.read(addr) as u16 + (cpu_bus.read((addr + 1) & 0x00ff) as u16) << 8;
                Operand::Word(data)
            },
            op::AddressingMode::Absolute => {
                // [high: 8, low: 8]
                let addr_or_data = self.fetch(cpu_bus) as u16 + (self.fetch(cpu_bus) as u16) << 8;
                Operand::Word(addr_or_data)
            },
            op::AddressingMode::AbsoluteX => {
                // [high: 8, low: 8] + X
                let addr_or_data = self.fetch(cpu_bus) as u16 + (self.fetch(cpu_bus) as u16) << 8;
                Operand::Word(addr_or_data + self.register.X as u16)
            },
            op::AddressingMode::AbsoluteY => {
                // [high: 8, low: 8] + Y
                let addr_or_data = self.fetch(cpu_bus) as u16 + (self.fetch(cpu_bus) as u16) << 8;
                Operand::Word(addr_or_data + self.register.Y as u16)
            },
            op::AddressingMode::Relative => {
                // 符号拡張のためi8にcast
                let addr = self.fetch(cpu_bus) as i8;
                Operand::Word(self.register.PC + addr as u16)
            },
        }
    }

    fn exec(&mut self, op::Instruction(opcode, mode, cycles): op::Instruction, operand: Operand, cpu_bus: &mut cpu_bus::CpuBus) -> u8 {
        match opcode {
            op::OpCode::LDA => {
                match (mode, operand) {
                    (op::AddressingMode::Immediate, Operand::Byte(byte)) => {
                        self.register.A = byte;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageY, Operand::Byte(byte)) => {
                        self.register.A = cpu_bus.read(byte as u16);
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteY, Operand::Word(word)) |
                    (op::AddressingMode::IndirectIndexed, Operand::Word(word)) |
                    (op::AddressingMode::IndexedIndirect, Operand::Word(word)) => {
                        self.register.A = cpu_bus.read(word);
                    }
                    _ => panic!("そんなアドレッシングモードとオペランドの組み合わせはありません opcode: {:?}, mode: {:?}", mode, operand)
                }
                self.register.P.negative =  self.register.A & 0b10000000 != 0;
                self.register.P.zero = self.register.A == 0;
            }
            _ => panic!("そんな命令はありません. opcode: {:?}", opcode)
        }
        cycles
    }
}

#[test]
fn it_works() {

}