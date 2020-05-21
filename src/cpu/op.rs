
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum AddressingMode {
    /// # Implied
    /// イミディエイト・アドレス指定（Immediate Addressing）
    /// 2番目のバイトをデータそのものとして使用します。
    /// - バイト形式: [オペコード][データ]
    /// - 使用するOperand: Operand::None,
    Implied,

    /// # Accumulator
    /// Aレジスタを操作するため、アドレス操作無し
    /// - 使用するOperand: Operand::None,
    Accumulator,

    /// - 使用するOperand: Operand::Byte(byte),

    Immediate,

    /// # Zeropage
    /// ゼロページ・アドレス指定（Zero Page Addressing）
    /// 上位アドレスとして$00、下位アドレスとして2番目のバイトを使用し実効アドレスとします。
    /// - バイト形式: [オペコード][下位アドレス]
    /// - 使用するOperand: Operand::Byte(byte)
    Zeropage,

    /// # ZeropageIndex
    /// ゼロページ・インデックス・アドレス指定（Indexed Zero Page Addressing）
    /// 上位アドレスとして$00、 下位アドレスとして2番目のバイトにインデックスレジスタ（X,Y）を加算（8） した値を実効アドレスとします。
    /// インデックスレジスタX、Yそれぞれについて、Zero Page, X、Zero Page, Y のアドレッシングモードがあります。
    /// - バイト形式: [オペコード][下位アドレス]
    /// - 使用するOperand: Operand::Byte(byte)
    ZeropageX,
    /// - 使用するOperand: Operand::Byte(byte)
    ZeropageY,

    /// # Absolute
    /// アブソリュート・アドレス指定（Absolute Addressing）
    /// 2番目のバイトを下位アドレス、 3番目のバイトを上位アドレスとして実効アドレスとします。
    /// - バイト形式: [オペコード][下位アドレス][上位アドレス]
    /// - 使用するOperand: Operand::Word(addr)
    Absolute,

    /// # AbsoluteIndex
    /// アブソリュート・インデックス・アドレス指定（Indexed Absolute Addressing）
    /// 2番目のバイトを下位アドレス、3番目のバイトを上位アドレスとして、 このアドレスにインデックスレジスタ（X,Y）を加算（16）したものを実効アドレスとします。
    /// インデックスレジスタX、Yそれぞれ、Absolute, X、Absolute, Y のアドレッシングモードがあります。
    /// - バイト形式: [オペコード][下位アドレス][上位アドレス]
    /// - 使用するOperand: Operand::Word(addr)
    AbsoluteX,
    /// - 使用するOperand: Operand::Word(addr)
    AbsoluteY,

    /// # Relative
    /// リラティブ・アドレス指定（Relative Addressing）
    /// 条件分岐命令で使用されます。 次の命令を示すプログラムカウンタに2番目のバイトを加算（符号拡張）した値を実効アドレスとします。
    /// オフセットとして、-128（$80）～+127（$7F）を指定できます。
    /// - バイト形式: [オペコード][オフセット]
    /// - 使用するOperand: Operand::Word(addr)
    Relative,

    /// # IndexedIndirect
    /// インデックスインダイレクト・アドレス指定（Indexed Indirect Addressing）
    /// 上位アドレスを$00とし、 また2番目のバイトにインデックスレジスタXを加算（8）した値を下位アドレスとします。
    /// このアドレスに格納されている値を実効アドレスの下位バイト、 そしてその次のアドレスに格納されている値を実効アドレスの上位バイトとします。
    /// このインクリメントにおいてキャリーは無視します。
    /// - バイト形式: [オペコード][下位アドレス]
    /// - 使用するOperand: Operand::Word(addr)
    IndexedIndirect,

    /// # IndirectIndexed
    /// インダイレクト・インデックス・アドレス指定（Indirect Indexed Addressing）
    /// まず上位アドレスを$00とし、下位アドレスとして2番目のバイトを使用します。
    /// このアドレスに格納されている値を次の上位アドレス、 その次のアドレスに格納されている値を次の下位アドレスとします。
    /// このときのインクリメントにおけるキャリーは無視します。 得られたアドレスにインデックスレジスタYを加算（16）したものを実効アドレスとします。
    /// - バイト形式: [オペコード][下位アドレス]
    /// - 使用するOperand: Operand::Word(addr)
    IndirectIndexed,

    /// # AbsoluteIndirect
    /// アブソリュート・インダイレクト・アドレス指定（Absolute Indirect Addressing）
    /// 2、3番目のバイトで示されるアドレスに格納されている値を実効アドレスの下位バイト、 その次のアドレスに格納されている値を実効アドレスの上位バイトとします。
    /// このインクリメントで下位バイトからのキャリーは無視します。
    /// - バイト形式: [オペコード][下位アドレス][上位アドレス]
    /// - 使用するOperand: Operand::Word(addr)
    AbsoluteIndirect,
}

/// see <http://pgate1.at-ninja.jp/NES_on_FPGA/nes_cpu.htm#instruction>
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum OpCode {
    // 演算
    ADC,
    SBC,
    // 論理演算
    AND,
    ORA,
    EOR,
    // シフトローテーション
    ASL,
    LSR,
    ROL,
    ROR,
    // 条件分岐
    BCC,
    BCS,
    BEQ,
    BNE,
    BVC,
    BVS,
    BPL,
    BMI,
    // ビット検査
    BIT,
    // ジャンプ
    JMP,
    JSR,
    RTS,
    // 割り込み
    BRK,
    RTI,
    // 比較
    CMP,
    CPX,
    CPY,
    // インクリメント・デクリメント
    INC,
    DEC,
    INX,
    DEX,
    INY,
    DEY,
    // フラグ操作
    CLC,
    SEC,
    CLI,
    SEI,
    CLD,
    SED,
    CLV,
    // ロード
    LDA,
    LDX,
    LDY,
    // ストア
    STA,
    STX,
    STY,
    // レジスタ間転送
    TAX,
    TXA,
    TAY,
    TYA,
    TSX,
    TXS,
    // スタック
    PHA,
    PLA,
    PHP,
    PLP,
    // noop
    NOP,
}

type Cycles = u8;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Instruction(pub OpCode, pub AddressingMode, pub Cycles);

const cycles: [u8; 0x100] = [
    /*0x00*/ 7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,
    /*0x10*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 6, 7,
    /*0x20*/ 6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,
    /*0x30*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 6, 7,
    /*0x40*/ 6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
    /*0x50*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 6, 7,
    /*0x60*/ 6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
    /*0x70*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 6, 7,
    /*0x80*/ 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
    /*0x90*/ 2, 6, 2, 6, 4, 4, 4, 4, 2, 4, 2, 5, 5, 4, 5, 5,
    /*0xA0*/ 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
    /*0xB0*/ 2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
    /*0xC0*/ 2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
    /*0xD0*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    /*0xE0*/ 2, 6, 3, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
    /*0xF0*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
];

/// see <https://qiita.com/bokuweb/items/1575337bef44ae82f4d3#%E5%91%BD%E4%BB%A4%E3%82%BB%E3%83%83%E3%83%88>
pub fn decode_op(op: u8) -> Instruction {
    match op {
        // 0x0X
        0x00 => Instruction(OpCode::BRK, AddressingMode::Immediate, cycles[op as usize]),
        0x01 => Instruction(OpCode::ORA, AddressingMode::IndexedIndirect, cycles[op as usize]),
        0x05 => Instruction(OpCode::ORA, AddressingMode::Zeropage, cycles[op as usize]),
        0x06 => Instruction(OpCode::ASL, AddressingMode::Zeropage, cycles[op as usize]),
        0x08 => Instruction(OpCode::PHP, AddressingMode::Implied, cycles[op as usize]),
        0x09 => Instruction(OpCode::ORA, AddressingMode::Immediate, cycles[op as usize]),
        0x0a => Instruction(OpCode::ASL, AddressingMode::Accumulator, cycles[op as usize]),
        0x0d => Instruction(OpCode::ORA, AddressingMode::Absolute, cycles[op as usize]),
        0x0e => Instruction(OpCode::ASL, AddressingMode::Absolute, cycles[op as usize]),

        // 0x1X
        0x10 => Instruction(OpCode::BPL, AddressingMode::Relative, cycles[op as usize]),
        0x11 => Instruction(OpCode::ORA, AddressingMode::IndirectIndexed, cycles[op as usize]),
        0x15 => Instruction(OpCode::ORA, AddressingMode::ZeropageX, cycles[op as usize]),
        0x16 => Instruction(OpCode::ASL, AddressingMode::ZeropageX, cycles[op as usize]),
        0x18 => Instruction(OpCode::CLC, AddressingMode::Implied, cycles[op as usize]),
        0x19 => Instruction(OpCode::ORA, AddressingMode::AbsoluteY, cycles[op as usize]),
        0x1d => Instruction(OpCode::ORA, AddressingMode::AbsoluteX, cycles[op as usize]),
        0x1e => Instruction(OpCode::ASL, AddressingMode::AbsoluteX, cycles[op as usize]),

        // 0x2X
        0x20 => Instruction(OpCode::JSR, AddressingMode::Absolute, cycles[op as usize]),
        0x21 => Instruction(OpCode::AND, AddressingMode::IndexedIndirect, cycles[op as usize]),
        0x24 => Instruction(OpCode::BIT, AddressingMode::Zeropage, cycles[op as usize]),
        0x25 => Instruction(OpCode::AND, AddressingMode::Zeropage, cycles[op as usize]),
        0x26 => Instruction(OpCode::ROR, AddressingMode::Zeropage, cycles[op as usize]),
        0x28 => Instruction(OpCode::PLP, AddressingMode::Implied, cycles[op as usize]),
        0x29 => Instruction(OpCode::AND, AddressingMode::Immediate, cycles[op as usize]),
        0x2a => Instruction(OpCode::ROR, AddressingMode::Accumulator, cycles[op as usize]),
        0x2c => Instruction(OpCode::BIT, AddressingMode::Absolute, cycles[op as usize]),
        0x2d => Instruction(OpCode::AND, AddressingMode::Absolute, cycles[op as usize]),
        0x2e => Instruction(OpCode::ROR, AddressingMode::Absolute, cycles[op as usize]),

        // 0x3X
        0x30 => Instruction(OpCode::BMI, AddressingMode::Relative, cycles[op as usize]),
        0x31 => Instruction(OpCode::AND, AddressingMode::IndirectIndexed, cycles[op as usize]),
        0x35 => Instruction(OpCode::AND, AddressingMode::ZeropageX, cycles[op as usize]),
        0x36 => Instruction(OpCode::ROL, AddressingMode::ZeropageX, cycles[op as usize]),
        0x38 => Instruction(OpCode::SEC, AddressingMode::Implied, cycles[op as usize]),
        0x39 => Instruction(OpCode::AND, AddressingMode::AbsoluteY, cycles[op as usize]),
        0x3d => Instruction(OpCode::AND, AddressingMode::AbsoluteX, cycles[op as usize]),
        0x3e => Instruction(OpCode::ROL, AddressingMode::AbsoluteX, cycles[op as usize]),

        // 0x4X
        0x40 => Instruction(OpCode::RTI, AddressingMode::Implied, cycles[op as usize]),
        0x41 => Instruction(OpCode::EOR, AddressingMode::IndexedIndirect, cycles[op as usize]),
        0x45 => Instruction(OpCode::EOR, AddressingMode::Zeropage, cycles[op as usize]),
        0x46 => Instruction(OpCode::LSR, AddressingMode::Zeropage, cycles[op as usize]),
        0x48 => Instruction(OpCode::PHA, AddressingMode::Implied, cycles[op as usize]),
        0x49 => Instruction(OpCode::EOR, AddressingMode::Immediate, cycles[op as usize]),
        0x4a => Instruction(OpCode::LSR, AddressingMode::Accumulator, cycles[op as usize]),
        0x4c => Instruction(OpCode::JMP, AddressingMode::Absolute, cycles[op as usize]),
        0x4d => Instruction(OpCode::EOR, AddressingMode::Absolute, cycles[op as usize]),
        0x4e => Instruction(OpCode::LSR, AddressingMode::Absolute, cycles[op as usize]),

        // 0x5X
        0x50 => Instruction(OpCode::BMI, AddressingMode::Relative, cycles[op as usize]),
        0x51 => Instruction(OpCode::AND, AddressingMode::IndirectIndexed, cycles[op as usize]),
        0x55 => Instruction(OpCode::AND, AddressingMode::ZeropageX, cycles[op as usize]),
        0x56 => Instruction(OpCode::ROL, AddressingMode::ZeropageX, cycles[op as usize]),
        0x58 => Instruction(OpCode::SEC, AddressingMode::Implied, cycles[op as usize]),
        0x59 => Instruction(OpCode::AND, AddressingMode::AbsoluteY, cycles[op as usize]),
        0x5d => Instruction(OpCode::AND, AddressingMode::AbsoluteX, cycles[op as usize]),
        0x5e => Instruction(OpCode::ROL, AddressingMode::AbsoluteX, cycles[op as usize]),

        // 0x6X
        0x60 => Instruction(OpCode::RTS, AddressingMode::Implied, cycles[op as usize]),
        0x61 => Instruction(OpCode::ADC, AddressingMode::IndexedIndirect, cycles[op as usize]),
        0x65 => Instruction(OpCode::ADC, AddressingMode::Zeropage, cycles[op as usize]),
        0x66 => Instruction(OpCode::ROR, AddressingMode::Zeropage, cycles[op as usize]),
        0x68 => Instruction(OpCode::PLA, AddressingMode::Implied, cycles[op as usize]),
        0x69 => Instruction(OpCode::ADC, AddressingMode::Immediate, cycles[op as usize]),
        0x6a => Instruction(OpCode::ROR, AddressingMode::Accumulator, cycles[op as usize]),
        0x6c => Instruction(OpCode::JMP, AddressingMode::IndirectIndexed, cycles[op as usize]),
        0x6d => Instruction(OpCode::ADC, AddressingMode::Absolute, cycles[op as usize]),
        0x6e => Instruction(OpCode::ROR, AddressingMode::Absolute, cycles[op as usize]),

        // 0x7X
        0x70 => Instruction(OpCode::BVS, AddressingMode::Relative, cycles[op as usize]),
        0x71 => Instruction(OpCode::ADC, AddressingMode::IndirectIndexed, cycles[op as usize]),
        0x75 => Instruction(OpCode::ADC, AddressingMode::ZeropageX, cycles[op as usize]),
        0x76 => Instruction(OpCode::ROR, AddressingMode::ZeropageX, cycles[op as usize]),
        0x78 => Instruction(OpCode::SEI, AddressingMode::Implied, cycles[op as usize]),
        0x79 => Instruction(OpCode::ADC, AddressingMode::AbsoluteY, cycles[op as usize]),
        0x7d => Instruction(OpCode::ADC, AddressingMode::AbsoluteX, cycles[op as usize]),
        0x7e => Instruction(OpCode::ROR, AddressingMode::AbsoluteX, cycles[op as usize]),

        // 0x8X
        0x81 => Instruction(OpCode::AND, AddressingMode::IndirectIndexed, cycles[op as usize]),
        0x84 => Instruction(OpCode::AND, AddressingMode::Zeropage, cycles[op as usize]),
        0x85 => Instruction(OpCode::ROL, AddressingMode::Zeropage, cycles[op as usize]),
        0x86 => Instruction(OpCode::SEC, AddressingMode::Implied, cycles[op as usize]),
        0x88 => Instruction(OpCode::AND, AddressingMode::Absolute, cycles[op as usize]),
        0x8a => Instruction(OpCode::AND, AddressingMode::Absolute, cycles[op as usize]),
        0x8c => Instruction(OpCode::AND, AddressingMode::Absolute, cycles[op as usize]),
        0x8d => Instruction(OpCode::AND, AddressingMode::Absolute, cycles[op as usize]),
        0x8e => Instruction(OpCode::ROL, AddressingMode::Absolute, cycles[op as usize]),

        // 0x9X
        0x90 => Instruction(OpCode::BCC, AddressingMode::Relative, cycles[op as usize]),
        0x91 => Instruction(OpCode::STA, AddressingMode::IndirectIndexed, cycles[op as usize]),
        0x94 => Instruction(OpCode::STY, AddressingMode::ZeropageX, cycles[op as usize]),
        0x95 => Instruction(OpCode::STA, AddressingMode::ZeropageY, cycles[op as usize]),
        0x96 => Instruction(OpCode::STX, AddressingMode::ZeropageY, cycles[op as usize]),
        0x98 => Instruction(OpCode::TYA, AddressingMode::Implied, cycles[op as usize]),
        0x99 => Instruction(OpCode::STA, AddressingMode::IndirectIndexed, cycles[op as usize]),
        0x9a => Instruction(OpCode::TXS, AddressingMode::Implied, cycles[op as usize]),
        0x9d => Instruction(OpCode::STA, AddressingMode::AbsoluteX, cycles[op as usize]),

        // 0xaX
        0xa0 => Instruction(OpCode::LDY, AddressingMode::Immediate, cycles[op as usize]),
        0xa1 => Instruction(OpCode::LDA, AddressingMode::IndexedIndirect, cycles[op as usize]),
        0xa2 => Instruction(OpCode::LDX, AddressingMode::Immediate, cycles[op as usize]),
        0xa4 => Instruction(OpCode::LDY, AddressingMode::Zeropage, cycles[op as usize]),
        0xa5 => Instruction(OpCode::LDA, AddressingMode::Zeropage, cycles[op as usize]),
        0xa6 => Instruction(OpCode::LDX, AddressingMode::Zeropage, cycles[op as usize]),
        0xa8 => Instruction(OpCode::TAY, AddressingMode::Implied, cycles[op as usize]),
        0xa9 => Instruction(OpCode::LDA, AddressingMode::Immediate, cycles[op as usize]),
        0xaa => Instruction(OpCode::TAX, AddressingMode::Implied, cycles[op as usize]),
        0xac => Instruction(OpCode::LDY, AddressingMode::Absolute, cycles[op as usize]),
        0xad => Instruction(OpCode::LDA, AddressingMode::Absolute, cycles[op as usize]),
        0xae => Instruction(OpCode::LDX, AddressingMode::Absolute, cycles[op as usize]),

        // 0xbX
        0xb0 => Instruction(OpCode::BCS, AddressingMode::Relative, cycles[op as usize]),
        0xb1 => Instruction(OpCode::LDA, AddressingMode::IndirectIndexed, cycles[op as usize]),
        0xb4 => Instruction(OpCode::LDY, AddressingMode::ZeropageX, cycles[op as usize]),
        0xb5 => Instruction(OpCode::LDA, AddressingMode::ZeropageX, cycles[op as usize]),
        0xb6 => Instruction(OpCode::LDX, AddressingMode::ZeropageY, cycles[op as usize]),
        0xb8 => Instruction(OpCode::CLV, AddressingMode::Implied, cycles[op as usize]),
        0xb9 => Instruction(OpCode::LDA, AddressingMode::AbsoluteY, cycles[op as usize]),
        0xba => Instruction(OpCode::TSX, AddressingMode::Implied, cycles[op as usize]),
        0xbc => Instruction(OpCode::LDY, AddressingMode::AbsoluteX, cycles[op as usize]),
        0xbd => Instruction(OpCode::LDA, AddressingMode::AbsoluteX, cycles[op as usize]),
        0xbe => Instruction(OpCode::LDX, AddressingMode::AbsoluteY, cycles[op as usize]),

        // 0xcX
        0xc0 => Instruction(OpCode::CPY, AddressingMode::Immediate, cycles[op as usize]),
        0xc1 => Instruction(OpCode::CMP, AddressingMode::IndirectIndexed, cycles[op as usize]),
        0xc4 => Instruction(OpCode::CPY, AddressingMode::Zeropage, cycles[op as usize]),
        0xc5 => Instruction(OpCode::CMP, AddressingMode::Zeropage, cycles[op as usize]),
        0xc6 => Instruction(OpCode::DEC, AddressingMode::Zeropage, cycles[op as usize]),
        0xc8 => Instruction(OpCode::INY, AddressingMode::Implied, cycles[op as usize]),
        0xc9 => Instruction(OpCode::CMP, AddressingMode::Immediate, cycles[op as usize]),
        0xca => Instruction(OpCode::DEX, AddressingMode::Implied, cycles[op as usize]),
        0xcc => Instruction(OpCode::CPY, AddressingMode::Absolute, cycles[op as usize]),
        0xcd => Instruction(OpCode::CMP, AddressingMode::Absolute, cycles[op as usize]),
        0xce => Instruction(OpCode::DEC, AddressingMode::Absolute, cycles[op as usize]),

        // 0xdX
        0xd0 => Instruction(OpCode::BNE, AddressingMode::Relative, cycles[op as usize]),
        0xd1 => Instruction(OpCode::CMP, AddressingMode::IndirectIndexed, cycles[op as usize]),
        0xd5 => Instruction(OpCode::CMP, AddressingMode::ZeropageX, cycles[op as usize]),
        0xd6 => Instruction(OpCode::DEC, AddressingMode::ZeropageX, cycles[op as usize]),
        0xd8 => Instruction(OpCode::CLD, AddressingMode::Immediate, cycles[op as usize]),
        0xd9 => Instruction(OpCode::CMP, AddressingMode::AbsoluteY, cycles[op as usize]),
        0xdd => Instruction(OpCode::CMP, AddressingMode::AbsoluteX, cycles[op as usize]),
        0xde => Instruction(OpCode::DEC, AddressingMode::AbsoluteX, cycles[op as usize]),

        // 0xeX
        0xe0 => Instruction(OpCode::CPX, AddressingMode::Immediate, cycles[op as usize]),
        0xe1 => Instruction(OpCode::SBC, AddressingMode::IndexedIndirect, cycles[op as usize]),
        0xe4 => Instruction(OpCode::CPX, AddressingMode::Zeropage, cycles[op as usize]),
        0xe5 => Instruction(OpCode::SBC, AddressingMode::Zeropage, cycles[op as usize]),
        0xe6 => Instruction(OpCode::INC , AddressingMode::Zeropage, cycles[op as usize]),
        0xe8 => Instruction(OpCode::INX, AddressingMode::Implied, cycles[op as usize]),
        0xe9 => Instruction(OpCode::SBC, AddressingMode::Immediate, cycles[op as usize]),
        0xea => Instruction(OpCode::NOP, AddressingMode:: Implied, cycles[op as usize]),
        0xec => Instruction(OpCode::CPX, AddressingMode::Absolute, cycles[op as usize]),
        0xed => Instruction(OpCode::SBC, AddressingMode::Absolute, cycles[op as usize]),
        0xee => Instruction(OpCode::INC, AddressingMode::Absolute, cycles[op as usize]),

        // 0xfX
        0xf0 => Instruction(OpCode::BEQ, AddressingMode::Relative, cycles[op as usize]),
        0xf1 => Instruction(OpCode::SBC, AddressingMode::IndexedIndirect, cycles[op as usize]),
        0xf5 => Instruction(OpCode::SBC, AddressingMode::ZeropageX, cycles[op as usize]),
        0xf6 => Instruction(OpCode::INC, AddressingMode::ZeropageX, cycles[op as usize]),
        0xf8 => Instruction(OpCode::SED, AddressingMode::Implied, cycles[op as usize]),
        0xf9 => Instruction(OpCode::SBC, AddressingMode::AbsoluteY, cycles[op as usize]),
        0xfd => Instruction(OpCode::SBC, AddressingMode::AbsoluteX, cycles[op as usize]),
        0xfe => Instruction(OpCode::INC, AddressingMode::AbsoluteX, cycles[op as usize]),

        _ => panic!("やばいです")
    }
}