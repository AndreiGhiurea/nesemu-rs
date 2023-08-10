use phf::phf_map;

pub type Cycles = u8;
pub type Length = u8;

#[derive(Debug)]
pub enum AddressingMode {
    Indirect,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
}

#[derive(Debug)]
pub enum Instruction {
    ADC(AddressingMode, Length, Cycles),
    AND(AddressingMode, Length, Cycles),
    ASL(AddressingMode, Length, Cycles),
    BIT(AddressingMode, Length, Cycles),
    BPL(Length, Cycles),
    BMI(Length, Cycles),
    BVC(Length, Cycles),
    BVS(Length, Cycles),
    BCC(Length, Cycles),
    BCS(Length, Cycles),
    BNE(Length, Cycles),
    BEQ(Length, Cycles),
    BRK(Length, Cycles),
    CMP(AddressingMode, Length, Cycles),
    CPX(AddressingMode, Length, Cycles),
    CPY(AddressingMode, Length, Cycles),
    DEC(AddressingMode, Length, Cycles),
    EOR(AddressingMode, Length, Cycles),
    CLC(Length, Cycles),
    SEC(Length, Cycles),
    CLI(Length, Cycles),
    SEI(Length, Cycles),
    CLV(Length, Cycles),
    CLD(Length, Cycles),
    SED(Length, Cycles),
    INC(AddressingMode, Length, Cycles),
    JMP(AddressingMode, Length, Cycles),
    JSR(AddressingMode, Length, Cycles),
    LDA(AddressingMode, Length, Cycles),
    LDX(AddressingMode, Length, Cycles),
    LDY(AddressingMode, Length, Cycles),
    LSR(AddressingMode, Length, Cycles),
    NOP(Length, Cycles),
    ORA(AddressingMode, Length, Cycles),
    TAX(Length, Cycles),
    TXA(Length, Cycles),
    DEX(Length, Cycles),
    INX(Length, Cycles),
    TAY(Length, Cycles),
    TYA(Length, Cycles),
    DEY(Length, Cycles),
    INY(Length, Cycles),
    ROL(AddressingMode, Length, Cycles),
    ROR(AddressingMode, Length, Cycles),
    RTI(Length, Cycles),
    RTS(Length, Cycles),
    SBC(AddressingMode, Length, Cycles),
    STA(AddressingMode, Length, Cycles),
    TXS(Length, Cycles),
    TSX(Length, Cycles),
    PHA(Length, Cycles),
    PLA(Length, Cycles),
    PHP(Length, Cycles),
    PLP(Length, Cycles),
    STX(AddressingMode, Length, Cycles),
    STY(AddressingMode, Length, Cycles),
}

pub static INSTRUCTIONS: phf::Map<u8, Instruction> = phf_map! {
    // ADC instruction
    0x69u8 => Instruction::ADC(AddressingMode::Immediate, 2, 2),
    0x65u8 => Instruction::ADC(AddressingMode::ZeroPage, 2, 3),
    0x75u8 => Instruction::ADC(AddressingMode::ZeroPageX, 2, 4),
    0x6Du8 => Instruction::ADC(AddressingMode::Absolute, 3, 4),
    0x7Du8 => Instruction::ADC(AddressingMode::AbsoluteX, 3, 4),
    0x79u8 => Instruction::ADC(AddressingMode::AbsoluteY, 3, 4),
    0x61u8 => Instruction::ADC(AddressingMode::IndirectX, 2, 6),
    0x71u8 => Instruction::ADC(AddressingMode::IndirectY, 2, 5),

    // AND instruction
    0x29u8 => Instruction::AND(AddressingMode::Immediate, 2, 2),
    0x25u8 => Instruction::AND(AddressingMode::ZeroPage, 2, 3),
    0x35u8 => Instruction::AND(AddressingMode::ZeroPageX, 2, 4),
    0x2Du8 => Instruction::AND(AddressingMode::Absolute, 3, 4),
    0x3Du8 => Instruction::AND(AddressingMode::AbsoluteX, 3, 4),
    0x39u8 => Instruction::AND(AddressingMode::AbsoluteY, 3, 4),
    0x21u8 => Instruction::AND(AddressingMode::IndirectX, 2, 6),
    0x31u8 => Instruction::AND(AddressingMode::IndirectY, 2, 5),

    // ASL instruction
    0x0Au8 => Instruction::ASL(AddressingMode::Accumulator, 1, 2),
    0x06u8 => Instruction::ASL(AddressingMode::ZeroPage, 2, 5),
    0x16u8 => Instruction::ASL(AddressingMode::ZeroPageX, 2, 6),
    0x0Eu8 => Instruction::ASL(AddressingMode::Absolute, 3, 6),
    0x1Eu8 => Instruction::ASL(AddressingMode::AbsoluteX, 3, 7),

    // BIT instruction
    0x24u8 => Instruction::BIT(AddressingMode::ZeroPage, 2, 3),
    0x2Cu8 => Instruction::BIT(AddressingMode::Absolute, 3, 4),

    // Branch instructions
    0x10u8 => Instruction::BPL(2, 2),
    0x30u8 => Instruction::BMI(2, 2),
    0x50u8 => Instruction::BVC(2, 2),
    0x70u8 => Instruction::BVS(2, 2),
    0x90u8 => Instruction::BCC(2, 2),
    0xB0u8 => Instruction::BCS(2, 2),
    0xD0u8 => Instruction::BNE(2, 2),
    0xF0u8 => Instruction::BEQ(2, 2),

    // Break instruction
    0x00u8 => Instruction::BRK(1, 7),

    // CMP instruction
    0xC9u8 => Instruction::CMP(AddressingMode::Immediate, 2, 2),
    0xC5u8 => Instruction::CMP(AddressingMode::ZeroPage, 2, 3),
    0xD5u8 => Instruction::CMP(AddressingMode::ZeroPageX, 2, 4),
    0xCDu8 => Instruction::CMP(AddressingMode::Absolute, 3, 4),
    0xDDu8 => Instruction::CMP(AddressingMode::AbsoluteX, 3, 4),
    0xD9u8 => Instruction::CMP(AddressingMode::AbsoluteY, 3, 4),
    0xC1u8 => Instruction::CMP(AddressingMode::IndirectX, 2, 6),
    0xD1u8 => Instruction::CMP(AddressingMode::IndirectY, 2, 5),

    // CPX instruction
    0xE0u8 => Instruction::CPX(AddressingMode::Immediate, 2, 2),
    0xE4u8 => Instruction::CPX(AddressingMode::ZeroPage, 2, 3),
    0xECu8 => Instruction::CPX(AddressingMode::Absolute, 3, 4),

    // CPY instruction
    0xC0u8 => Instruction::CPY(AddressingMode::Immediate, 2, 2),
    0xC4u8 => Instruction::CPY(AddressingMode::ZeroPage, 2, 3),
    0xCCu8 => Instruction::CPY(AddressingMode::Absolute, 3, 4),

    // DEC instruction
    0xC6u8 => Instruction::DEC(AddressingMode::ZeroPage, 2, 5),
    0xD6u8 => Instruction::DEC(AddressingMode::ZeroPageX, 2, 6),
    0xCEu8 => Instruction::DEC(AddressingMode::Absolute, 3, 6),
    0xDEu8 => Instruction::DEC(AddressingMode::AbsoluteX, 3, 7),

    // EOR instruction
    0x49u8 => Instruction::EOR(AddressingMode::Immediate, 2, 2),
    0x45u8 => Instruction::EOR(AddressingMode::ZeroPage, 2, 3),
    0x55u8 => Instruction::EOR(AddressingMode::ZeroPageX, 2, 4),
    0x4Du8 => Instruction::EOR(AddressingMode::Absolute, 3, 4),
    0x5Du8 => Instruction::EOR(AddressingMode::AbsoluteX, 3, 4),
    0x59u8 => Instruction::EOR(AddressingMode::AbsoluteY, 3, 4),
    0x41u8 => Instruction::EOR(AddressingMode::IndirectX, 2, 6),
    0x51u8 => Instruction::EOR(AddressingMode::IndirectY, 2, 5),

    // Flag instructions
    0x18u8 => Instruction::CLC(1, 2),
    0x38u8 => Instruction::SEC(1, 2),
    0x58u8 => Instruction::CLI(1, 2),
    0x78u8 => Instruction::SEI(1, 2),
    0xB8u8 => Instruction::CLV(1, 2),
    0xD8u8 => Instruction::CLD(1, 2),
    0xF8u8 => Instruction::SED(1, 2),

    // INC instruction
    0xE6u8 => Instruction::INC(AddressingMode::ZeroPage, 2, 5),
    0xF6u8 => Instruction::INC(AddressingMode::ZeroPageX, 2, 6),
    0xEEu8 => Instruction::INC(AddressingMode::Absolute, 3, 6),
    0xFEu8 => Instruction::INC(AddressingMode::AbsoluteX, 3,7),

    // JMP instruction
    0x4Cu8 => Instruction::JMP(AddressingMode::Absolute, 3, 3),
    0x6Cu8 => Instruction::JMP(AddressingMode::Indirect, 3, 5),

    // JSR instruction
    0x20u8 => Instruction::JSR(AddressingMode::Absolute, 3, 6),

    // LDA instruction
    0xA9u8 => Instruction::LDA(AddressingMode::Immediate, 2, 2),
    0xA5u8 => Instruction::LDA(AddressingMode::ZeroPage, 2, 3),
    0xB5u8 => Instruction::LDA(AddressingMode::ZeroPageX, 2, 4),
    0xADu8 => Instruction::LDA(AddressingMode::Absolute, 3, 4),
    0xBDu8 => Instruction::LDA(AddressingMode::AbsoluteX, 3, 4),
    0xB9u8 => Instruction::LDA(AddressingMode::AbsoluteY, 3, 4),
    0xA1u8 => Instruction::LDA(AddressingMode::IndirectX, 2, 6),
    0xB1u8 => Instruction::LDA(AddressingMode::IndirectY, 2, 5),

    // LDX instruction
    0xA2u8 => Instruction::LDX(AddressingMode::Immediate, 2, 2),
    0xA6u8 => Instruction::LDX(AddressingMode::ZeroPage, 2, 3),
    0xB6u8 => Instruction::LDX(AddressingMode::ZeroPageY, 2, 4),
    0xAEu8 => Instruction::LDX(AddressingMode::Absolute, 3, 4),
    0xBEu8 => Instruction::LDX(AddressingMode::AbsoluteY, 3, 4),

    // LDY instruction
    0xA0u8 => Instruction::LDY(AddressingMode::Immediate, 2, 2),
    0xA4u8 => Instruction::LDY(AddressingMode::ZeroPage, 2, 3),
    0xB4u8 => Instruction::LDY(AddressingMode::ZeroPageX, 2, 4),
    0xACu8 => Instruction::LDY(AddressingMode::Absolute, 3, 4),
    0xBCu8 => Instruction::LDY(AddressingMode::AbsoluteX, 3, 4),

    // LSR instruction
    0x4Au8 => Instruction::LSR(AddressingMode::Accumulator, 1, 2),
    0x46u8 => Instruction::LSR(AddressingMode::ZeroPage, 2, 5),
    0x56u8 => Instruction::LSR(AddressingMode::ZeroPageX, 2, 6),
    0x4Eu8 => Instruction::LSR(AddressingMode::Absolute, 3, 6),
    0x5Eu8 => Instruction::LSR(AddressingMode::AbsoluteX, 3, 7),

    // NOP instruction
    0xEAu8 => Instruction::NOP(1, 2),

    // ORA instruction
    0x09u8 => Instruction::ORA(AddressingMode::Immediate, 2, 2),
    0x05u8 => Instruction::ORA(AddressingMode::ZeroPage, 2, 3),
    0x15u8 => Instruction::ORA(AddressingMode::ZeroPageX, 2, 4),
    0x0Du8 => Instruction::ORA(AddressingMode::Absolute, 3, 4),
    0x1Du8 => Instruction::ORA(AddressingMode::AbsoluteX, 3, 4),
    0x19u8 => Instruction::ORA(AddressingMode::AbsoluteY, 3, 4),
    0x01u8 => Instruction::ORA(AddressingMode::IndirectX, 2, 6),
    0x11u8 => Instruction::ORA(AddressingMode::IndirectY, 2, 5),

    // Register instructions
    0xAAu8 => Instruction::TAX(1, 2),
    0x8Au8 => Instruction::TXA(1, 2),
    0xCAu8 => Instruction::DEX(1, 2),
    0xE8u8 => Instruction::INX(1, 2),
    0xA8u8 => Instruction::TAY(1, 2),
    0x98u8 => Instruction::TYA(1, 2),
    0x88u8 => Instruction::DEY(1, 2),
    0xC8u8 => Instruction::INY(1, 2),

    // ROL instruction
    0x2Au8 => Instruction::ROL(AddressingMode::Accumulator, 1, 2),
    0x26u8 => Instruction::ROL(AddressingMode::ZeroPage, 2, 5),
    0x36u8 => Instruction::ROL(AddressingMode::ZeroPageX, 2, 6),
    0x2Eu8 => Instruction::ROL(AddressingMode::Absolute, 3, 6),
    0x3Eu8 => Instruction::ROL(AddressingMode::AbsoluteX, 3, 7),

    // ROR instruction
    0x6Au8 => Instruction::ROR(AddressingMode::Accumulator, 1, 2),
    0x66u8 => Instruction::ROR(AddressingMode::ZeroPage, 2, 5),
    0x76u8 => Instruction::ROR(AddressingMode::ZeroPageX, 2, 6),
    0x6Eu8 => Instruction::ROR(AddressingMode::Absolute, 3, 6),
    0x7Eu8 => Instruction::ROR(AddressingMode::AbsoluteX, 3, 7),

    // RTI instruction
    0x40u8 => Instruction::RTI(1, 6),

    // RTS instruction
    0x60u8 => Instruction::RTS(1, 6),

    // SBC instruction
    0xE9u8 => Instruction::SBC(AddressingMode::Immediate, 2, 2),
    0xE5u8 => Instruction::SBC(AddressingMode::ZeroPage, 2, 3),
    0xF5u8 => Instruction::SBC(AddressingMode::ZeroPageX, 2, 4),
    0xEDu8 => Instruction::SBC(AddressingMode::Absolute, 3, 4),
    0xFDu8 => Instruction::SBC(AddressingMode::AbsoluteX, 3, 4),
    0xF9u8 => Instruction::SBC(AddressingMode::AbsoluteY, 3, 4),
    0xE1u8 => Instruction::SBC(AddressingMode::IndirectX, 2, 6),
    0xF1u8 => Instruction::SBC(AddressingMode::IndirectY, 2, 5),

    // STA instruction
    0x85u8 => Instruction::STA(AddressingMode::ZeroPage, 2, 3),
    0x95u8 => Instruction::STA(AddressingMode::ZeroPageX, 2, 4),
    0x8Du8 => Instruction::STA(AddressingMode::Absolute, 3, 4),
    0x9Du8 => Instruction::STA(AddressingMode::AbsoluteX, 3, 5),
    0x99u8 => Instruction::STA(AddressingMode::AbsoluteY, 3, 5),
    0x81u8 => Instruction::STA(AddressingMode::IndirectX, 2, 6),
    0x91u8 => Instruction::STA(AddressingMode::IndirectY, 2, 6),

    // Stack instructions
    0x9Au8 => Instruction::TXS(1, 2),
    0xBAu8 => Instruction::TSX(1, 2),
    0x48u8 => Instruction::PHA(1, 3),
    0x68u8 => Instruction::PLA(1, 4),
    0x08u8 => Instruction::PHP(1, 3),
    0x28u8 => Instruction::PLP(1, 4),

    // STX instruction
    0x86u8 => Instruction::STX(AddressingMode::ZeroPage, 2, 3),
    0x96u8 => Instruction::STX(AddressingMode::ZeroPageY, 2, 4),
    0x8Eu8 => Instruction::STX(AddressingMode::Absolute, 3, 4),

    // STY instruction
    0x84u8 => Instruction::STY(AddressingMode::ZeroPage, 2, 3),
    0x94u8 => Instruction::STY(AddressingMode::ZeroPageX, 2, 4),
    0x8Cu8 => Instruction::STY(AddressingMode::Absolute, 3, 4),
};
