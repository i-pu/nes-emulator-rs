use crate::cpu_bus;

mod op;
mod tests;
struct Register {
    A: u8,	                // 8bit	アキュームレータ	汎用演算
    X: u8,	                // 8bit	インデックスレジスタ	アドレッシング、カウンタなど
    Y: u8,	                // 8bit	インデックスレジスタ	アドレッシング、カウンタなど
    S: u8,	                // 8bit	スタックポインタ	スタックの位置を保持
    P: StatusRegister,	    // 8bit	ステータスレジスタ	CPUの各種状態を保持
    PC: u16,                // 16bit	プログラムカウンタ	実行している位置を保持
    SP: u16,                // 16bit スタックポインタ $0100 - $01FF (上位8bitは0x01固定)
}

impl Register {
    pub fn new() -> Self {
        Self {
            A: 0x00u8,
            X: 0x00u8,
            Y: 0x00u8,
            S: 0x00u8,
            P: StatusRegister::new(),
            PC: 0x0000u16,
            SP: 0x01ffu16,
        }
    }
}

/// # ステータス・レジスタ
///
/// ステータスレジスタの詳細です。bit5は常に1で、bit3はNESでは未実装です。
/// IRQは割り込み、BRKはソフトウエア割り込みです。
struct StatusRegister {
    /// # negative
    /// 7 N ネガティブ 負数の判定用。
    negative: bool,
    /// # overflow
    /// 6 V オーバーフロー 演算がオーバーフローを起こした場合セットされます。
    /// V = C6 xor C7
    overflow: bool,
    /// # reserved
    /// 5 R 予約済み 使用できません。常にセットされています。
    reserved: bool,
    /// # breakm
    /// 4 B ブレークモード BRK発生時はセットされ、IRQ発生時はクリアされます。
    breakm: bool,
    /// # decimal
    ///3Dデシマルモード セットすると、BCDモードで動作します。(ファミコンでは未実装)
    decimal: bool,
    /// # interrupt
    /// 2 I IRQ禁止 クリアするとIRQが許可され、セットするとIRQが禁止になります。
    interrupt: bool,
    /// # zero
    /// 1 Z ゼロ 演算結果が0になった場合セットされます。ロード命令でも変化します。
    zero: bool,
    /// # carry
    /// 0 C キャリー キャリー発生時セットされます。
	carry: bool,
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
    fn reset(&mut self) {
        self.register = Register::new();
    }

    /// スタックにプッシュ(下方向に伸びる)
    /// see <https://pgate1.at-ninja.jp/NES_on_FPGA/nes_cpu.htm#stack>
    fn stack_push(&mut self, cpu_bus: &mut cpu_bus::CpuBus, data: u8) {
        cpu_bus.write(self.register.SP, data);
        self.register.SP -= 1;
    }

    /// スタックからポップ
    fn stack_pop(&mut self, cpu_bus: &mut cpu_bus::CpuBus) -> u8 {
        self.register.SP += 1;
        cpu_bus.read(self.register.SP)
    }

    fn pop_status(&mut self, cpu_bus: &mut cpu_bus::CpuBus) {
        let status = self.stack_pop(cpu_bus);
        self.set_flags(status);
    }
    fn push_status(&mut self, cpu_bus: &mut cpu_bus::CpuBus) {
        let flags = self.get_flags();
        self.stack_push(cpu_bus, flags);
    }

    fn popPC(&mut self, cpu_bus: &mut cpu_bus::CpuBus) {
        self.register.PC = self.stack_pop(cpu_bus) as u16;
        self.register.PC += (self.stack_pop(cpu_bus) as u16).rotate_left(8);
    }

    /// フラグレジスタ
    fn get_flags(&mut self) -> u8 {
        // 7-0: [N V R B D I Z C]
        (self.register.P.negative as u8) << 7 +
            (self.register.P.overflow as u8) << 6 +
            (self.register.P.reserved as u8) << 5 +
            (self.register.P.breakm as u8) << 4 +
            (self.register.P.decimal as u8) << 3 +
            (self.register.P.interrupt as u8) << 2 +
            (self.register.P.zero as u8) << 1 +
            (self.register.P.carry as u8)
    }

    fn set_flags(&mut self, flags: u8) {
        self.register.P.negative = (flags & (1 << 7)) != 0;
        self.register.P.overflow = (flags & (1 << 6)) != 0;
        self.register.P.reserved = (flags & (1 << 5)) != 0;
        self.register.P.breakm = (flags & (1 << 4)) != 0;
        self.register.P.decimal = (flags & (1 << 3)) != 0;
        self.register.P.interrupt = (flags & (1 << 2)) != 0;
        self.register.P.zero = (flags & (1 << 1)) != 0;
        self.register.P.carry = (flags & 1) != 0;
    }

    fn pagesDiffer(a: u16, b: u16) -> bool {
        return a&0xFF00 != b&0xFF00;
    }

    // adds a cycle for taking a branch
    fn addBranchCycles(&mut self, addr: u16, mut cycles: u8) -> u8 {
        cycles += 1;
        if Self::pagesDiffer(self.register.PC, addr) {
            cycles += 1;
        }
        cycles
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

    fn exec(&mut self, op::Instruction(opcode, mode, mut cycles): op::Instruction, operand: Operand, cpu_bus: &mut cpu_bus::CpuBus) -> u8 {
        match opcode {
            // 転送命令
            // see <http://hp.vector.co.jp/authors/VA042397/nes/6502.html#translate>
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

                // N Z
                self.register.P.negative =  self.register.A & 0b10000000 != 0;
                self.register.P.zero = self.register.A == 0;
            }
            // アドレス「IM16 + X」の8bit値をXにロード
            op::OpCode::LDX => {
                match (mode, operand) {
                    (op::AddressingMode::Immediate, Operand::Byte(byte)) => {
                        self.register.X = byte;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageY, Operand::Byte(byte)) => {
                        self.register.X = cpu_bus.read(byte as u16);
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteY, Operand::Word(word)) => {
                        self.register.X = cpu_bus.read(word);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative =  self.register.X & 0b10000000 != 0;
                self.register.P.zero = self.register.X == 0;
            }
            // アドレス「IM16 + Y」の8bit値をAにロード
            op::OpCode::LDY => {
                match (mode, operand) {
                    (op::AddressingMode::Immediate, Operand::Byte(byte)) => {
                        self.register.Y = byte;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        self.register.Y = cpu_bus.read(byte as u16);
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) => {
                        self.register.Y = cpu_bus.read(word);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative =  self.register.Y & 0b10000000 != 0;
                self.register.P.zero = self.register.Y == 0;
            }
            op::OpCode::STA => {
                match (mode, operand) {
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        cpu_bus.write(byte as u16, self.register.A);
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) |
                    (op::AddressingMode::IndirectIndexed, Operand::Word(word)) |
                    (op::AddressingMode::IndexedIndirect, Operand::Word(word)) => {
                        cpu_bus.write(word, self.register.A);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::STX => {
                match (mode, operand) {
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageY, Operand::Byte(byte)) => {
                        cpu_bus.write(byte as u16, self.register.X);
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) => {
                        cpu_bus.write(word, self.register.X);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::STY => {
                match (mode, operand) {
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        cpu_bus.write(byte as u16, self.register.Y);
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) => {
                        cpu_bus.write(word, self.register.Y);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            // レジスタ間転送
            op::OpCode::TAX => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.X = self.register.A;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative =  self.register.X & 0b10000000 != 0;
                self.register.P.zero = self.register.X == 0;
            }
            op::OpCode::TXA => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.A = self.register.X;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                //  N Z
                self.register.P.negative =  self.register.A & 0b10000000 != 0;
                self.register.P.zero = self.register.A == 0;
            }
            op::OpCode::TAY => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.Y = self.register.A;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative =  self.register.Y & 0b10000000 != 0;
                self.register.P.zero = self.register.Y == 0;
            }
            op::OpCode::TYA => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.A = self.register.Y;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // NZ
                self.register.P.negative =  self.register.A & 0b10000000 != 0;
                self.register.P.zero = self.register.A == 0;
            }
            op::OpCode::TSX => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.X = self.register.S;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative =  self.register.X & 0b10000000 != 0;
                self.register.P.zero = self.register.X == 0;
            }
            op::OpCode::TXS => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.S = self.register.X;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative =  self.register.S & 0b10000000 != 0;
                self.register.P.zero = self.register.S == 0;
            }

            // 算術演算
            op::OpCode::ADC => {
                match (mode, operand) {
                    (op::AddressingMode::Immediate, Operand::Byte(byte)) => {
                        // calc on u16
                        let result = self.register.A as u16 + byte as u16 + self.register.P.carry as u16;
                        // N V Z C
                        // FIXME: 多分バグらせ侍
                        // TODO: オーバフローのフラグ治す
                        // over u8
                        self.register.P.carry = result > 0x00ffu16;
                        // タブンチガウ
                        self.register.P.overflow = self.register.P.carry;
                        // u16 -> u8
                        self.register.A = (result & 0xff) as u8;
                        self.register.P.negative = self.register.A & 0b10000000 != 0;
                        self.register.P.zero = self.register.A == 0;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        let data = cpu_bus.read(byte as u16);
                        // calc on u16
                        let result = self.register.A as u16 + data as u16 + self.register.P.carry as u16;
                        // N V Z C
                        self.register.P.carry = result > 0x00ffu16;
                        self.register.P.overflow = self.register.P.carry;
                        self.register.A = (result & 0xff) as u8;
                        self.register.P.negative =  self.register.A & 0b10000000 != 0;
                        self.register.P.zero = self.register.A == 0;
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteY, Operand::Word(word)) |
                    (op::AddressingMode::IndirectIndexed, Operand::Word(word)) |
                    (op::AddressingMode::IndexedIndirect, Operand::Word(word)) => {
                        let data = cpu_bus.read(word);
                        // calc on u16
                        let result = self.register.A as u16 + data as u16 + self.register.P.carry as u16;
                        // N V Z C
                        self.register.P.carry = result > 0x00ffu16;
                        self.register.P.overflow = self.register.P.carry;
                        self.register.A = (result & 0xff) as u8;
                        self.register.P.negative =  self.register.A & 0b10000000 != 0;
                        self.register.P.zero = self.register.A & 0b10000000 != 0;
                        self.register.P.zero = self.register.A == 0;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            // 論理演算
            op::OpCode::AND => {
                match (mode, operand) {
                    (op::AddressingMode::Immediate, Operand::Byte(byte)) => {
                        self.register.A &= byte;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        self.register.A &= cpu_bus.read(byte as u16);
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteY, Operand::Word(word))|
                    (op::AddressingMode::IndirectIndexed, Operand::Word(word)) |
                    (op::AddressingMode::IndexedIndirect, Operand::Word(word)) => {
                        self.register.A &= cpu_bus.read(word);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative =  self.register.A & 0b10000000 != 0;
                self.register.P.zero = self.register.A == 0;
            }
            op::OpCode::ORA => {
                match (mode, operand) {
                    (op::AddressingMode::Immediate, Operand::Byte(byte)) => {
                        self.register.A |= byte;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        self.register.A |= cpu_bus.read(byte as u16);
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteY, Operand::Word(word))|
                    (op::AddressingMode::IndirectIndexed, Operand::Word(word)) |
                    (op::AddressingMode::IndexedIndirect, Operand::Word(word)) => {
                        self.register.A |= cpu_bus.read(word);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative =  self.register.A & 0b10000000 != 0;
                self.register.P.zero = self.register.A == 0;
            }
            op::OpCode::EOR => {
                match (mode, operand) {
                    (op::AddressingMode::Immediate, Operand::Byte(byte)) => {
                        self.register.A ^= byte;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        self.register.A ^= cpu_bus.read(byte as u16);
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteY, Operand::Word(word))|
                    (op::AddressingMode::IndirectIndexed, Operand::Word(word)) |
                    (op::AddressingMode::IndexedIndirect, Operand::Word(word)) => {
                        self.register.A ^= cpu_bus.read(word);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative =  self.register.A & 0b10000000 != 0;
                self.register.P.zero = self.register.A == 0;
            }
            // インクリメント・デクリメント
            op::OpCode::INC => {
                let res = match (mode, operand) {
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        let data = cpu_bus.read(byte as u16);
                        cpu_bus.write(byte as u16, data.wrapping_add(1))
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) => {
                        let data = cpu_bus.read(word);
                        cpu_bus.write(word, data.wrapping_add(1))
                    }
                    _ => panic!("error, mode: {:?}, operand: {:?}", mode, operand)
                };

                // N Z
                self.register.P.negative =  res & 0b10000000 != 0;
                self.register.P.zero = res == 0;
            }
            op::OpCode::DEC => {
                let res = match (mode, operand) {
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        let data = cpu_bus.read(byte as u16);
                        cpu_bus.write(byte as u16, data.wrapping_sub(1))
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) => {
                        let data = cpu_bus.read(word);
                        cpu_bus.write(word, data.wrapping_sub(1))
                    }
                    _ => panic!("error, mode: {:?}, operand: {:?}", mode, operand)
                };
                // N Z
                self.register.P.negative =  res & 0b10000000 != 0;
                self.register.P.zero = res == 0;
            }
            op::OpCode::INX => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.X = self.register.X.wrapping_add(1)
                    }
                    _ => panic!("error, mode: {:?}, operand: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative =  self.register.X & 0b10000000 != 0;
                self.register.P.zero = self.register.X == 0;
            }
            op::OpCode::DEX => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.X = self.register.X.wrapping_sub(1);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative =  self.register.X & 0b10000000 != 0;
                self.register.P.zero = self.register.X == 0;
            }
            op::OpCode::INY => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        // avoid overflow
                        self.register.Y = self.register.Y.wrapping_add(1);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative =  self.register.Y & 0b10000000 != 0;
                self.register.P.zero = self.register.Y == 0;
            }
            op::OpCode::DEY => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.Y = self.register.Y.wrapping_sub(1);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

                // N Z
                self.register.P.negative = self.register.Y & 0b10000000 != 0;
                self.register.P.zero = self.register.Y == 0;
            }

            // 比較
            // see <http://6502.org/tutorials/compare_instructions.html>
            op::OpCode::CMP => {
                match (mode, operand) {
                    (op::AddressingMode::Immediate, Operand::Byte(byte)) => {
                        let a = self.register.A;
                        // N Z C
                        self.register.P.carry = a >= byte;
                        self.register.P.zero = a == byte;
                        self.register.P.negative = (a.wrapping_sub(byte)) & 0b10000_000 != 0;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        let mem = cpu_bus.read(byte as u16);
                        let a = self.register.A;
                        // N Z C
                        self.register.P.carry = a >= mem;
                        self.register.P.zero = a == mem;
                        self.register.P.negative = (a.wrapping_sub(mem)) & 0b10000_000 != 0;
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteY, Operand::Word(word))|
                    (op::AddressingMode::IndirectIndexed, Operand::Word(word)) |
                    (op::AddressingMode::IndexedIndirect, Operand::Word(word)) => {
                        let mem = cpu_bus.read(word);
                        let a = self.register.A;

                        // N Z C
                        self.register.P.carry = a >= mem;
                        self.register.P.zero = a == mem;
                        self.register.P.negative = (a.wrapping_sub(mem)) & 0b10000_000 == 1;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::CPX => {
                match (mode, operand) {
                    (op::AddressingMode::Immediate, Operand::Byte(byte)) => {
                        let x = self.register.X;
                        // N Z C
                        self.register.P.carry = x >= byte;
                        self.register.P.zero = x == byte;
                        self.register.P.negative = (x.wrapping_sub(byte)) & 0b10000_000 != 0;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) => {
                        let mem = cpu_bus.read(byte as u16);
                        let x = self.register.X;
                        // N Z C
                        self.register.P.carry = x >= mem;
                        self.register.P.zero = x == mem;
                        self.register.P.negative = (x.wrapping_sub(mem)) & 0b10000_000 != 0;
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word))=> {
                        let mem = cpu_bus.read(word);
                        let x = self.register.X;
                        // N Z C
                        self.register.P.carry = x >= mem;
                        self.register.P.zero = x == mem;
                        self.register.P.negative = (x.wrapping_sub(mem)) & 0b10000_000 != 0;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::CPY => {
                match (mode, operand) {
                    (op::AddressingMode::Immediate, Operand::Byte(byte)) => {
                        let y = self.register.Y;
                        // N Z C
                        self.register.P.carry = y >= byte;
                        self.register.P.zero = y == byte;
                        self.register.P.negative = (y.wrapping_sub(byte)) & 0b10000_000 != 0;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) => {
                        let mem = cpu_bus.read(byte as u16);
                        let y = self.register.Y;
                        // N Z C
                        self.register.P.carry = y >= mem;
                        self.register.P.zero = y == mem;
                        self.register.P.negative = (y.wrapping_sub(mem)) & 0b10000_000 != 0;

                    }
                    (op::AddressingMode::Absolute, Operand::Word(word))=> {
                        let mem = cpu_bus.read(word);
                        let y = self.register.Y;
                        // N Z C
                        self.register.P.carry = y >= mem;
                        self.register.P.zero = y == mem;
                        self.register.P.negative = (y.wrapping_sub(mem)) & 0b10000_000 != 0;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            },

            // // シフトローテーション
            op::OpCode::ASL => {
                match (mode, operand) {
                    (op::AddressingMode::Accumulator, Operand::None) => {
                        self.register.P.carry = self.register.A & 0b10000_000 != 0;
                        self.register.A = self.register.A << 1;
                        self.register.P.zero = self.register.A == 0;
                        self.register.P.negative = self.register.A & 0b10000_000 != 0;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        let mut data = cpu_bus.read(byte as u16);
                        self.register.P.carry = data & 0b10000_000 != 0;
                        data =  data << 1;
                        cpu_bus.write(byte as u16, data);
                        self.register.P.zero = data == 0;
                        self.register.P.negative = data & 0b10000_000 != 0;
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) => {
                        let mut data = cpu_bus.read(word);
                        self.register.P.carry = data & 0b10000_000 != 0;
                        data =  data << 1;
                        cpu_bus.write(word, data);
                        self.register.P.zero = data == 0;
                        self.register.P.negative = data & 0b10000_000 != 0;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::LSR => {
                match (mode, operand) {
                    (op::AddressingMode::Accumulator, Operand::None) => {
                        self.register.P.carry = self.register.A & 0b00000_001 != 0;
                        self.register.A = self.register.A >> 1;
                        self.register.P.zero = self.register.A == 0;
                        self.register.P.negative = self.register.A & 0b10000_000 != 0;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        let mut data = cpu_bus.read(byte as u16);
                        self.register.P.carry = data & 0b00000_001 != 0;
                        data =  data >> 1;
                        cpu_bus.write(byte as u16, data);
                        self.register.P.zero = data == 0;
                        self.register.P.negative = data & 0b10000_000 != 0;
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) => {
                        let mut data = cpu_bus.read(word);
                        self.register.P.carry = data & 0b00000_001 != 0;
                        data =  data >> 1;
                        cpu_bus.write(word, data);
                        self.register.P.zero = data == 0;
                        self.register.P.negative = data & 0b10000_000 != 0;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::ROL => {
                match (mode, operand) {
                    (op::AddressingMode::Accumulator, Operand::None) => {
                        let a = self.register.A;
                        self.register.A = a.rotate_left(1);
                        self.register.P.carry = a & 0x80 != 0;
                        self.register.P.zero = self.register.A == 0;
                        self.register.P.negative = self.register.A & 0x80 != 0;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        let data = cpu_bus.read(byte as u16);
                        let data_written = data.rotate_left(1);
                        cpu_bus.write(byte as u16, data_written);
                        self.register.P.carry = data & 0x80 != 0;
                        self.register.P.zero = data_written == 0;
                        self.register.P.negative = data_written & 0x80 != 0;
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) => {
                        let data = cpu_bus.read(word);
                        let data_written = data.rotate_left(1);
                        cpu_bus.write(word, data_written);
                        self.register.P.carry = data & 0x80 != 0;
                        self.register.P.zero = data_written == 0;
                        self.register.P.negative = data_written & 0x80 != 0;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::ROR => {
                match (mode, operand) {
                    (op::AddressingMode::Accumulator, Operand::None) => {
                        let a = self.register.A;
                        self.register.A = a.rotate_right(1);
                        self.register.P.carry = a & 0x01 != 0;
                        self.register.P.zero = self.register.A == 0;
                        self.register.P.negative = self.register.A & 0x80 != 0;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        let data = cpu_bus.read(byte as u16);
                        let data_written = data.rotate_right(1);
                        cpu_bus.write(byte as u16, data_written);
                        self.register.P.carry = data & 0x01 != 0;
                        self.register.P.zero = data_written == 0;
                        self.register.P.negative = data_written & 0x80 != 0;
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) => {
                        let data = cpu_bus.read(word);
                        let data_written = data.rotate_right(1);
                        cpu_bus.write(word, data_written);
                        self.register.P.carry = data & 0x01 != 0;
                        self.register.P.zero = data_written == 0;
                        self.register.P.negative = data_written & 0x80 != 0;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::SBC => {
                match (mode, operand) {
                    (op::AddressingMode::Immediate, Operand::Byte(byte)) => {
                        // calc on u16
                        let result = self.register.A as u16 - byte as u16 - (1 - self.register.P.carry as u16);
                        self.register.P.carry = result > 0x00ffu16;
                        self.register.P.overflow = ((self.register.A as u16 ^ result) & 0x80) != 0 && ((self.register.A ^ byte) & 0x80) != 0;
                        self.register.A = (result & 0xff) as u8;
                        self.register.P.negative = self.register.A & 0b10000000 != 0;
                        self.register.P.zero = self.register.A == 0;
                    }
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) |
                    (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {
                        // TODO: 自信ない
                        let data = cpu_bus.read(byte as u16);
                        let result = self.register.A as u16 - data as u16 - (1 - self.register.P.carry as u16);
                        self.register.P.carry = result > 0x00ffu16;
                        self.register.P.overflow = ((self.register.A as u16 ^ result) & 0x80) != 0 && ((self.register.A ^ data) & 0x80) != 0;
                        self.register.A = (result & 0xff) as u8;
                        self.register.P.negative = self.register.A & 0b10000000 != 0;
                        self.register.P.zero = self.register.A == 0;
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteX, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteY, Operand::Word(word))|
                    (op::AddressingMode::AbsoluteIndirect, Operand::Word(word)) |
                    (op::AddressingMode::IndirectIndexed, Operand::Word(word)) => {
                        let data = cpu_bus.read(word);
                        let result = self.register.A as u16 - data as u16 - (1 - self.register.P.carry as u16);
                        self.register.P.carry = result > 0x00ffu16;
                        self.register.P.overflow = ((self.register.A as u16 ^ result) & 0x80) != 0 && ((self.register.A ^ data) & 0x80) != 0;
                        self.register.A = (result & 0xff) as u8;
                        self.register.P.negative = self.register.A & 0b10000000 != 0;
                        self.register.P.zero = self.register.A == 0;
                    }

                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }

            // ビット検査
            op::OpCode::BIT => {
                match (mode, operand) {
                    (op::AddressingMode::Zeropage, Operand::Byte(byte)) => {
                        let data = cpu_bus.read(byte as u16);
                        self.register.P.negative = data & 0x80 != 0;
                        self.register.P.overflow = data & 0x40 != 0;
                        self.register.P.zero = self.register.A & data != 0;
                    }
                    (op::AddressingMode::Absolute, Operand::Word(word)) => {
                        let data = cpu_bus.read(word);
                        self.register.P.negative = data & 0x80 != 0;
                        self.register.P.overflow = data & 0x40 != 0;
                        self.register.P.zero = self.register.A & data != 0;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }

            }

            // スタック
            op::OpCode::PHA => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.stack_push(cpu_bus, self.register.A);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::PLA => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.A = self.stack_pop(cpu_bus);
                        // N Z
                        self.register.P.negative = self.register.A & 0b10000000 != 0;
                        self.register.P.zero = self.register.A == 0;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::PHP => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        let flag = self.get_flags();
                        self.stack_push(cpu_bus, flag);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::PLP => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        let flag = self.stack_pop(cpu_bus);
                        self.set_flags(flag);
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }

            // ジャンプ
            op::OpCode::JMP => {
                match (mode, operand) {
                    (op::AddressingMode::Absolute, Operand::Word(word)) |
                    (op::AddressingMode::AbsoluteIndirect, Operand::Word(word)) => {
                        self.register.PC = word;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::JSR => {
                match (mode, operand) {
                    (op::AddressingMode::AbsoluteIndirect, Operand::Word(word)) => {
                        let mut pc = self.register.PC.wrapping_sub(1);
                        self.stack_push(cpu_bus, pc.rotate_right(8) as u8);
                        self.stack_push(cpu_bus, pc as u8);
                        self.register.PC = word;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::RTS => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.stack_pop(cpu_bus);
                        self.register.PC += 1;
                    }

                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::RTI => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.pop_status(cpu_bus);
                        self.popPC(cpu_bus);
                        // ↓ jsの人はこのようにが仕様にないし、goの人もやっていない
                        // self.register.P.reserved = true;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }

            // 条件分岐

            // Branch if Carry Clear
            op::OpCode::BCC => {
                match (mode, operand) {
                    (op::AddressingMode::Relative, Operand::Word(word)) => {
                        if !self.register.P.carry {
                            self.register.PC = word;
                            cycles += self.addBranchCycles(word, cycles);
                        }
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            // Branch if Carry set
            op::OpCode::BCS => {
                match (mode, operand) {
                    (op::AddressingMode::Relative, Operand::Word(word)) => {
                        if self.register.P.carry {
                            self.register.PC = word;
                            cycles += self.addBranchCycles(word, cycles);
                        }
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            // Branch if Equal
            op::OpCode::BEQ => {
                match (mode, operand) {
                    (op::AddressingMode::Relative, Operand::Word(word)) => {
                        if self.register.P.zero {
                            self.register.PC = word;
                            cycles += self.addBranchCycles(word, cycles);
                        }
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            // Branch if Not Equal
            op::OpCode::BNE => {
                match (mode, operand) {
                    (op::AddressingMode::Relative, Operand::Word(word)) => {
                        if !self.register.P.zero {
                            self.register.PC = word;
                            cycles += self.addBranchCycles(word, cycles);
                        }
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            // Branch if Overflow Clear
            op::OpCode::BVC => {
                match (mode, operand) {
                    (op::AddressingMode::Relative, Operand::Word(word)) => {
                        if !self.register.P.overflow {
                            self.register.PC = word;
                            cycles += self.addBranchCycles(word, cycles);
                        }
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            // Branch if Overflow Set
            op::OpCode::BVS => {
                match (mode, operand) {
                    (op::AddressingMode::Relative, Operand::Word(word)) => {
                        if self.register.P.overflow {
                            self.register.PC = word;
                            cycles += self.addBranchCycles(word, cycles);
                        }

                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            // Branch if Positive
            op::OpCode::BPL => {
                match (mode, operand) {
                    (op::AddressingMode::Relative, Operand::Word(word)) => {
                        if !self.register.P.negative {
                            self.register.PC = word;
                            cycles += self.addBranchCycles(word, cycles);
                        }
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            // Branch if Minus
            op::OpCode::BMI => {
                match (mode, operand) {
                    (op::AddressingMode::Relative, Operand::Word(word)) => {
                        if self.register.P.negative {
                            self.register.PC = word;
                            cycles += self.addBranchCycles(word, cycles);
                        }
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }

            // フラグ操作
            op::OpCode::CLC => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.P.carry = false;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::SEC => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.P.carry = true;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::CLI => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.P.interrupt = false;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::SEI => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.P.interrupt = true;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::CLD => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.P.interrupt = false;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::SED => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.P.interrupt = true;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::CLV => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        self.register.P.overflow = false;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }

            // その他
            op::OpCode::BRK => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => {
                        let old_interrupt = self.register.P.interrupt;
                        self.register.PC += 1;
                        self.stack_push(cpu_bus, (self.register.PC >> 8) as u8);
                        self.stack_push(cpu_bus, (self.register.PC & 0xFF)as u8);
                        self.register.P.breakm = true;
                        self.push_status(cpu_bus);
                        self.register.P.interrupt = true;
                        if !old_interrupt {
                            let addr: u16 = cpu_bus.read(0xfffe) as u16 | ((cpu_bus.read(0xffff) as u16) << 8);
                            self.register.PC = addr;
                        }
                        self.register.PC -= 1;
                    }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            op::OpCode::NOP => {
                match (mode, operand) {
                    (op::AddressingMode::Implied, Operand::None) => { }
                    _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
                }
            }
            // template
            // op::OpCode::XXX => {
            //     match (mode, operand) {
            //         (op::AddressingMode::Immediate, Operand::Byte(byte)) => {}
            //         (op::AddressingMode::Accumulator, Operand::None) => {}
            //         (op::AddressingMode::Zeropage, Operand::Byte(byte)) => {}
            //         (op::AddressingMode::ZeropageX, Operand::Byte(byte)) => {}
            //         (op::AddressingMode::ZeropageY, Operand::Byte(byte)) => {}
            //         (op::AddressingMode::Relative, Operand::Word(word)) => {}
            //         (op::AddressingMode::Absolute, Operand::Word(word))=> {}
            //         (op::AddressingMode::AbsoluteX, Operand::Word(word)) => {}
            //         (op::AddressingMode::AbsoluteY, Operand::Word(word)) => {}
            //         (op::AddressingMode::AbsoluteIndirect, Operand::Word(word)) => {}
            //         (op::AddressingMode::IndirectIndexed, Operand::Word(word)) => {}
            //         (op::AddressingMode::IndexedIndirect, Operand::Word(word)) => {}
            //         _ => panic!("error opcode: {:?}, mode: {:?}", mode, operand)
            //     }
            // }
        }
        cycles
    }
}
