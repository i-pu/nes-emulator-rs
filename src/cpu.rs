mod op;
use crate::cpu_bus;
struct Register {
    A: u8,	                // 8bit	アキュームレータ	汎用演算
    X: u8,	                // 8bit	インデックスレジスタ	アドレッシング、カウンタなど
    Y: u8,	                // 8bit	インデックスレジスタ	アドレッシング、カウンタなど
    S: u8,	                // 8bit	スタックポインタ	スタックの位置を保持
    P: StatusRegister,	    // 8bit	ステータスレジスタ	CPUの各種状態を保持
    PC: u16,                // 	16bit	プログラムカウンタ	実行している位置を保持
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

//ステータス・レジスタ
/*
ステータスレジスタの詳細です。bit5は常に1で、bit3はNESでは未実装です。
IRQは割り込み、BRKはソフトウエア割り込みです。
*/
struct StatusRegister {
	negative: bool, 	    // 7 N ネガティブ	Aの7ビット目と同じになります。負数の判定用。
	overflow: bool, 	    // 6 V オーバーフロー	演算がオーバーフローを起こした場合セットされます。
	reserved: bool, 	    // 5 R 予約済み	使用できません。常にセットされています。
	breakm: bool, 	    // 4 B ブレークモード	BRK発生時はセットされ、IRQ発生時はクリアされます。
	decimal: bool, 	    // 3 D デシマルモード	セットすると、BCDモードで動作します。(ファミコンでは未実装)
	interrupt: bool, 	    // 2 I IRQ禁止	クリアするとIRQが許可され、セットするとIRQが禁止になります。
	zero: bool, 	    // 1 Z ゼロ	演算結果が0になった場合セットされます。ロード命令でも変化します。
	carry: bool, 	    // 0 C キャリー	キャリー発生時セットされます。
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

struct Cpu {
    register: Register,
    bus: cpu_bus::CpuBus,
    // 実装イメージ
    //CPUは基本的には以下の手順を繰り返せばいいことが分かります。
    //1. PC（プログラムカウンタ）からオペコードをフェッチ（PCをインクリメント）
    //2. 命令とアドレッシング・モードを判別
    //3.（必要であれば）オペランドをフェッチ（PCをインクリメント）
    //4.（必要であれば）演算対象となるアドレスを算出
    //5. 命令を実行
    //6. 1に戻る
}

impl Cpu {
    /// CPUの実行
    // 実行タイミング調整のために実行にかかったサイクル数を返す
    fn run(mut self) -> u8 {
        let opcode = self.fetch();
        let op::Instruction(opcode, mode, cycle) = op::decode_op(opcode);
        let opeland = self.fetch_opeland(mode);
        self.exec(opcode, opeland, mode);
        return cycle;
    }

    fn read(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn write(&mut self, data: u8) {
        self.bus.write(data);
    }

    /// レジスタとプラグラムカウンタを上げ、命令コードをメモリから読み込む
    fn fetch(&mut self) -> u8 {
		self.register.PC += 1;
        return self.read(self.register.PC);
    }

    // レジスタの初期化
    fn reset(&mut self) -> () {
        self.register = Register::new();
    }

    /// fetch_opelandはアドレッシングモードからアドレスを返す
    /// アドレスを返さない場合があるのでそのときはNoneを返す
    fn fetch_opeland(&mut self, mode: op::AddressingMode) -> Option<u16> {
        match mode {
            op::AddressingMode::Accumulator => None,
            op::AddressingMode::Implied => None,
            op::AddressingMode::Immediate => Some(self.fetch() as u16),
            op::AddressingMode::Zeropage => Some(self.fetch() as u16),
            op::AddressingMode::ZeropageX => {
                let addr = self.fetch();
                Some((addr + self.register.X) as u16 & 0xffff)
            },
            op::AddressingMode::ZeropageY => {
                let addr = self.fetch();
                Some((addr + self.register.Y) as u16 & 0xffff)
            },
            // TODO: バグだらけちゃんとアドレッシングモードを理解しろ
            op::AddressingMode::IndexedIndirect => {
                let base_addr: u16 = (self.fetch() + self.register.X) as u16 & 0xffff;
                let addr: u16 = self.read(base_addr as u16) as u16 + (self.read((base_addr + 1) & 0xff) << 8) as u16;
                Some(addr & 0xffff)
            },
            op::AddressingMode::IndirectIndexed => {
                let addr_or_data: u16 = self.fetch() as u16;
                let base_addr: u16 = self.read(addr_or_data) as u16 + (self.read((addr_or_data + 1) & 0xff) << 8) as u16;
                let addr = base_addr + self.register.Y as u16;
                Some(addr & 0xffff)
            },
            op::AddressingMode::AbsoluteIndirect => {
                //       const addrOrData = this.fetchWord();
                //         const addr = this.reaid(addrOrData) + (this.read((addrOrData & 0xFF00) | (((addrOrData & 0xFF) + 1) & 0xFF)) << 8);
                //       return addr & 0xFFFF;
                unimplemented!()
            }
            _ => panic!("やばいです@アドレッシングモード"),
        }
    }

    fn exec(&mut self, opcode: op::OpCode, opeland: Option<u16>, mode: op::AddressingMode) {
        match opcode {
            op::OpCode::LDA => {
                self.register.A = if mode == op::AddressingMode::Immediate {
                    // TODO: unwrapやめましょう
                    // TODO: 絶対バグある
                    opeland.unwrap() as u8
                } else {
                    // TODO: unwrapやめましょう
                    self.read(opeland.unwrap() as u16)
                };
                self.register.P.negative = (self.register.A & 0x80) == 0;
                self.register.P.zero = !(self.register.A == 0);
            },
            _ => todo!(""),
        }
        /* ...略... */
        /* 残りの命令を実装 */
    }

}