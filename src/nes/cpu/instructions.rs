use phf::phf_map;

use super::{emulator::Emu, Cpu};

pub type Cycles = u8;
pub type Length = u8;

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    Relative,
    Implied,
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

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum InstructionVariant {
    ADC,
    AND,
    ASL,
    BIT,
    BPL,
    BMI,
    BVC,
    BVS,
    BCC,
    BCS,
    BNE,
    BEQ,
    BRK,
    CMP,
    CPX,
    CPY,
    DEC,
    EOR,
    CLC,
    SEC,
    CLI,
    SEI,
    CLV,
    CLD,
    SED,
    INC,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    TAX,
    TXA,
    DEX,
    INX,
    TAY,
    TYA,
    DEY,
    INY,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    STA,
    TXS,
    TSX,
    PHA,
    PLA,
    PHP,
    PLP,
    STX,
    STY,
    // Unofficial opcodes
    LAX,
    SAX,
    DCP,
    ISB,
    SLO,
    RLA,
    SRE,
    RRA,
}

pub struct Instruction {
    pub variant: InstructionVariant,
    pub mode: AddressingMode,
    pub length: Length,
    pub cycles: Cycles,
    pub emu_fn: fn(&mut Cpu, &Instruction),
}

pub static INSTRUCTIONS: phf::Map<u8, Instruction> = phf_map! {
    // ADC instruction
    0x69u8 => Instruction{variant: InstructionVariant::ADC, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::adc},
    0x65u8 => Instruction{variant: InstructionVariant::ADC, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::adc},
    0x75u8 => Instruction{variant: InstructionVariant::ADC, mode: AddressingMode::ZeroPageX, length: 2, cycles: 4, emu_fn: Emu::adc},
    0x6Du8 => Instruction{variant: InstructionVariant::ADC, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::adc},
    0x7Du8 => Instruction{variant: InstructionVariant::ADC, mode: AddressingMode::AbsoluteX, length: 3, cycles: 4, emu_fn: Emu::adc},
    0x79u8 => Instruction{variant: InstructionVariant::ADC, mode: AddressingMode::AbsoluteY, length: 3, cycles: 4, emu_fn: Emu::adc},
    0x61u8 => Instruction{variant: InstructionVariant::ADC, mode: AddressingMode::IndirectX, length: 2, cycles: 6, emu_fn: Emu::adc},
    0x71u8 => Instruction{variant: InstructionVariant::ADC, mode: AddressingMode::IndirectY, length: 2, cycles: 5, emu_fn: Emu::adc},

    // AND instruction
    0x29u8 => Instruction{variant: InstructionVariant::AND, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::and},
    0x25u8 => Instruction{variant: InstructionVariant::AND, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::and},
    0x35u8 => Instruction{variant: InstructionVariant::AND, mode: AddressingMode::ZeroPageX, length: 2, cycles: 4, emu_fn: Emu::and},
    0x2Du8 => Instruction{variant: InstructionVariant::AND, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::and},
    0x3Du8 => Instruction{variant: InstructionVariant::AND, mode: AddressingMode::AbsoluteX, length: 3, cycles: 4, emu_fn: Emu::and},
    0x39u8 => Instruction{variant: InstructionVariant::AND, mode: AddressingMode::AbsoluteY, length: 3, cycles: 4, emu_fn: Emu::and},
    0x21u8 => Instruction{variant: InstructionVariant::AND, mode: AddressingMode::IndirectX, length: 2, cycles: 6, emu_fn: Emu::and},
    0x31u8 => Instruction{variant: InstructionVariant::AND, mode: AddressingMode::IndirectY, length: 2, cycles: 5, emu_fn: Emu::and},

    // ASL instruction
    0x0Au8 => Instruction{variant: InstructionVariant::ASL, mode: AddressingMode::Accumulator, length: 1, cycles: 2, emu_fn: Emu::asl},
    0x06u8 => Instruction{variant: InstructionVariant::ASL, mode: AddressingMode::ZeroPage, length: 2, cycles: 5, emu_fn: Emu::asl},
    0x16u8 => Instruction{variant: InstructionVariant::ASL, mode: AddressingMode::ZeroPageX, length: 2, cycles: 6, emu_fn: Emu::asl},
    0x0Eu8 => Instruction{variant: InstructionVariant::ASL, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::asl},
    0x1Eu8 => Instruction{variant: InstructionVariant::ASL, mode: AddressingMode::AbsoluteX, length: 3, cycles: 7, emu_fn: Emu::asl},

    // BIT instruction
    0x24u8 => Instruction{variant: InstructionVariant::BIT, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::bit},
    0x2Cu8 => Instruction{variant: InstructionVariant::BIT, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::bit},

    // Branch instructions
    0x10u8 => Instruction{variant: InstructionVariant::BPL, mode: AddressingMode::Relative, length: 2, cycles: 2, emu_fn: Emu::bpl},
    0x30u8 => Instruction{variant: InstructionVariant::BMI, mode: AddressingMode::Relative, length: 2, cycles: 2, emu_fn: Emu::bmi},
    0x50u8 => Instruction{variant: InstructionVariant::BVC, mode: AddressingMode::Relative, length: 2, cycles: 2, emu_fn: Emu::bvc},
    0x70u8 => Instruction{variant: InstructionVariant::BVS, mode: AddressingMode::Relative, length: 2, cycles: 2, emu_fn: Emu::bvs},
    0x90u8 => Instruction{variant: InstructionVariant::BCC, mode: AddressingMode::Relative, length: 2, cycles: 2, emu_fn: Emu::bcc},
    0xB0u8 => Instruction{variant: InstructionVariant::BCS, mode: AddressingMode::Relative, length: 2, cycles: 2, emu_fn: Emu::bcs},
    0xD0u8 => Instruction{variant: InstructionVariant::BNE, mode: AddressingMode::Relative, length: 2, cycles: 2, emu_fn: Emu::bne},
    0xF0u8 => Instruction{variant: InstructionVariant::BEQ, mode: AddressingMode::Relative, length: 2, cycles: 2, emu_fn: Emu::beq},

    // Break instruction
    0x00u8 => Instruction{variant: InstructionVariant::BRK, mode: AddressingMode::Implied, length: 1, cycles: 7, emu_fn: Emu::brk},

    // CMP instruction
    0xC9u8 => Instruction{variant: InstructionVariant::CMP, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::cmp},
    0xC5u8 => Instruction{variant: InstructionVariant::CMP, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::cmp},
    0xD5u8 => Instruction{variant: InstructionVariant::CMP, mode: AddressingMode::ZeroPageX, length: 2, cycles: 4, emu_fn: Emu::cmp},
    0xCDu8 => Instruction{variant: InstructionVariant::CMP, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::cmp},
    0xDDu8 => Instruction{variant: InstructionVariant::CMP, mode: AddressingMode::AbsoluteX, length: 3, cycles: 4, emu_fn: Emu::cmp},
    0xD9u8 => Instruction{variant: InstructionVariant::CMP, mode: AddressingMode::AbsoluteY, length: 3, cycles: 4, emu_fn: Emu::cmp},
    0xC1u8 => Instruction{variant: InstructionVariant::CMP, mode: AddressingMode::IndirectX, length: 2, cycles: 6, emu_fn: Emu::cmp},
    0xD1u8 => Instruction{variant: InstructionVariant::CMP, mode: AddressingMode::IndirectY, length: 2, cycles: 5, emu_fn: Emu::cmp},

    // CPX instruction
    0xE0u8 => Instruction{variant: InstructionVariant::CPX, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::cpx},
    0xE4u8 => Instruction{variant: InstructionVariant::CPX, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::cpx},
    0xECu8 => Instruction{variant: InstructionVariant::CPX, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::cpx},

    // CPY instruction
    0xC0u8 => Instruction{variant: InstructionVariant::CPY, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::cpy},
    0xC4u8 => Instruction{variant: InstructionVariant::CPY, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::cpy},
    0xCCu8 => Instruction{variant: InstructionVariant::CPY, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::cpy},

    // DEC instruction
    0xC6u8 => Instruction{variant: InstructionVariant::DEC, mode: AddressingMode::ZeroPage, length: 2, cycles: 5, emu_fn: Emu::dec},
    0xD6u8 => Instruction{variant: InstructionVariant::DEC, mode: AddressingMode::ZeroPageX, length: 2, cycles: 6, emu_fn: Emu::dec},
    0xCEu8 => Instruction{variant: InstructionVariant::DEC, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::dec},
    0xDEu8 => Instruction{variant: InstructionVariant::DEC, mode: AddressingMode::AbsoluteX, length: 3, cycles: 7, emu_fn: Emu::dec},

    // EOR instruction
    0x49u8 => Instruction{variant: InstructionVariant::EOR, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::eor},
    0x45u8 => Instruction{variant: InstructionVariant::EOR, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::eor},
    0x55u8 => Instruction{variant: InstructionVariant::EOR, mode: AddressingMode::ZeroPageX, length: 2, cycles: 4, emu_fn: Emu::eor},
    0x4Du8 => Instruction{variant: InstructionVariant::EOR, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::eor},
    0x5Du8 => Instruction{variant: InstructionVariant::EOR, mode: AddressingMode::AbsoluteX, length: 3, cycles: 4, emu_fn: Emu::eor},
    0x59u8 => Instruction{variant: InstructionVariant::EOR, mode: AddressingMode::AbsoluteY, length: 3, cycles: 4, emu_fn: Emu::eor},
    0x41u8 => Instruction{variant: InstructionVariant::EOR, mode: AddressingMode::IndirectX, length: 2, cycles: 6, emu_fn: Emu::eor},
    0x51u8 => Instruction{variant: InstructionVariant::EOR, mode: AddressingMode::IndirectY, length: 2, cycles: 5, emu_fn: Emu::eor},

    // Flag instructions
    0x18u8 => Instruction{variant: InstructionVariant::CLC, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::clc},
    0x38u8 => Instruction{variant: InstructionVariant::SEC, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::sec},
    0x58u8 => Instruction{variant: InstructionVariant::CLI, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::cli},
    0x78u8 => Instruction{variant: InstructionVariant::SEI, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::sei},
    0xB8u8 => Instruction{variant: InstructionVariant::CLV, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::clv},
    0xD8u8 => Instruction{variant: InstructionVariant::CLD, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::cld},
    0xF8u8 => Instruction{variant: InstructionVariant::SED, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::sed},

    // INC instruction
    0xE6u8 => Instruction{variant: InstructionVariant::INC, mode: AddressingMode::ZeroPage, length: 2, cycles: 5, emu_fn: Emu::inc},
    0xF6u8 => Instruction{variant: InstructionVariant::INC, mode: AddressingMode::ZeroPageX, length: 2, cycles: 6, emu_fn: Emu::inc},
    0xEEu8 => Instruction{variant: InstructionVariant::INC, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::inc},
    0xFEu8 => Instruction{variant: InstructionVariant::INC, mode: AddressingMode::AbsoluteX, length: 3, cycles: 7, emu_fn: Emu::inc},

    // JMP instruction
    0x4Cu8 => Instruction{variant: InstructionVariant::JMP, mode: AddressingMode::Absolute, length: 3, cycles: 3, emu_fn: Emu::jmp},
    0x6Cu8 => Instruction{variant: InstructionVariant::JMP, mode: AddressingMode::Indirect, length: 3, cycles: 5, emu_fn: Emu::jmp},

    // JSR instruction
    0x20u8 => Instruction{variant: InstructionVariant::JSR, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::jsr},

    // LDA instruction
    0xA9u8 => Instruction{variant: InstructionVariant::LDA, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::lda},
    0xA5u8 => Instruction{variant: InstructionVariant::LDA, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::lda},
    0xB5u8 => Instruction{variant: InstructionVariant::LDA, mode: AddressingMode::ZeroPageX, length: 2, cycles: 4, emu_fn: Emu::lda},
    0xADu8 => Instruction{variant: InstructionVariant::LDA, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::lda},
    0xBDu8 => Instruction{variant: InstructionVariant::LDA, mode: AddressingMode::AbsoluteX, length: 3, cycles: 4, emu_fn: Emu::lda},
    0xB9u8 => Instruction{variant: InstructionVariant::LDA, mode: AddressingMode::AbsoluteY, length: 3, cycles: 4, emu_fn: Emu::lda},
    0xA1u8 => Instruction{variant: InstructionVariant::LDA, mode: AddressingMode::IndirectX, length: 2, cycles: 6, emu_fn: Emu::lda},
    0xB1u8 => Instruction{variant: InstructionVariant::LDA, mode: AddressingMode::IndirectY, length: 2, cycles: 5, emu_fn: Emu::lda},

    // LDX instruction
    0xA2u8 => Instruction{variant: InstructionVariant::LDX, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::ldx},
    0xA6u8 => Instruction{variant: InstructionVariant::LDX, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::ldx},
    0xB6u8 => Instruction{variant: InstructionVariant::LDX, mode: AddressingMode::ZeroPageY, length: 2, cycles: 4, emu_fn: Emu::ldx},
    0xAEu8 => Instruction{variant: InstructionVariant::LDX, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::ldx},
    0xBEu8 => Instruction{variant: InstructionVariant::LDX, mode: AddressingMode::AbsoluteY, length: 3, cycles: 4, emu_fn: Emu::ldx},

    // LDY instruction
    0xA0u8 => Instruction{variant: InstructionVariant::LDY, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::ldy},
    0xA4u8 => Instruction{variant: InstructionVariant::LDY, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::ldy},
    0xB4u8 => Instruction{variant: InstructionVariant::LDY, mode: AddressingMode::ZeroPageX, length: 2, cycles: 4, emu_fn: Emu::ldy},
    0xACu8 => Instruction{variant: InstructionVariant::LDY, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::ldy},
    0xBCu8 => Instruction{variant: InstructionVariant::LDY, mode: AddressingMode::AbsoluteX, length: 3, cycles: 4, emu_fn: Emu::ldy},

    // LSR instruction
    0x4Au8 => Instruction{variant: InstructionVariant::LSR, mode: AddressingMode::Accumulator, length: 1, cycles: 2, emu_fn: Emu::lsr},
    0x46u8 => Instruction{variant: InstructionVariant::LSR, mode: AddressingMode::ZeroPage, length: 2, cycles: 5, emu_fn: Emu::lsr},
    0x56u8 => Instruction{variant: InstructionVariant::LSR, mode: AddressingMode::ZeroPageX, length: 2, cycles: 6, emu_fn: Emu::lsr},
    0x4Eu8 => Instruction{variant: InstructionVariant::LSR, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::lsr},
    0x5Eu8 => Instruction{variant: InstructionVariant::LSR, mode: AddressingMode::AbsoluteX, length: 3, cycles: 7, emu_fn: Emu::lsr},

    // NOP instruction
    0xEAu8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::nop},

    // ORA instruction
    0x09u8 => Instruction{variant: InstructionVariant::ORA, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::ora},
    0x05u8 => Instruction{variant: InstructionVariant::ORA, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::ora},
    0x15u8 => Instruction{variant: InstructionVariant::ORA, mode: AddressingMode::ZeroPageX, length: 2, cycles: 4, emu_fn: Emu::ora},
    0x0Du8 => Instruction{variant: InstructionVariant::ORA, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::ora},
    0x1Du8 => Instruction{variant: InstructionVariant::ORA, mode: AddressingMode::AbsoluteX, length: 3, cycles: 4, emu_fn: Emu::ora},
    0x19u8 => Instruction{variant: InstructionVariant::ORA, mode: AddressingMode::AbsoluteY, length: 3, cycles: 4, emu_fn: Emu::ora},
    0x01u8 => Instruction{variant: InstructionVariant::ORA, mode: AddressingMode::IndirectX, length: 2, cycles: 6, emu_fn: Emu::ora},
    0x11u8 => Instruction{variant: InstructionVariant::ORA, mode: AddressingMode::IndirectY, length: 2, cycles: 5, emu_fn: Emu::ora},

    // Register instructions
    0xAAu8 => Instruction{variant: InstructionVariant::TAX, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::tax},
    0x8Au8 => Instruction{variant: InstructionVariant::TXA, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::txa},
    0xCAu8 => Instruction{variant: InstructionVariant::DEX, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::dex},
    0xE8u8 => Instruction{variant: InstructionVariant::INX, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::inx},
    0xA8u8 => Instruction{variant: InstructionVariant::TAY, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::tay},
    0x98u8 => Instruction{variant: InstructionVariant::TYA, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::tya},
    0x88u8 => Instruction{variant: InstructionVariant::DEY, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::dey},
    0xC8u8 => Instruction{variant: InstructionVariant::INY, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::iny},

    // ROL instruction
    0x2Au8 => Instruction{variant: InstructionVariant::ROL, mode: AddressingMode::Accumulator, length: 1, cycles: 2, emu_fn: Emu::rol},
    0x26u8 => Instruction{variant: InstructionVariant::ROL, mode: AddressingMode::ZeroPage, length: 2, cycles: 5, emu_fn: Emu::rol},
    0x36u8 => Instruction{variant: InstructionVariant::ROL, mode: AddressingMode::ZeroPageX, length: 2, cycles: 6, emu_fn: Emu::rol},
    0x2Eu8 => Instruction{variant: InstructionVariant::ROL, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::rol},
    0x3Eu8 => Instruction{variant: InstructionVariant::ROL, mode: AddressingMode::AbsoluteX, length: 3, cycles: 7, emu_fn: Emu::rol},

    // ROR instruction
    0x6Au8 => Instruction{variant: InstructionVariant::ROR, mode: AddressingMode::Accumulator, length: 1, cycles: 2, emu_fn: Emu::ror},
    0x66u8 => Instruction{variant: InstructionVariant::ROR, mode: AddressingMode::ZeroPage, length: 2, cycles: 5, emu_fn: Emu::ror},
    0x76u8 => Instruction{variant: InstructionVariant::ROR, mode: AddressingMode::ZeroPageX, length: 2, cycles: 6, emu_fn: Emu::ror},
    0x6Eu8 => Instruction{variant: InstructionVariant::ROR, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::ror},
    0x7Eu8 => Instruction{variant: InstructionVariant::ROR, mode: AddressingMode::AbsoluteX, length: 3, cycles: 7, emu_fn: Emu::ror},

    // RTI instruction
    0x40u8 => Instruction{variant: InstructionVariant::RTI, mode: AddressingMode::Implied, length: 1, cycles: 6, emu_fn: Emu::rti},

    // RTS instruction
    0x60u8 => Instruction{variant: InstructionVariant::RTS, mode: AddressingMode::Implied, length: 1, cycles: 6, emu_fn: Emu::rts},

    // SBC instruction
    0xE9u8 => Instruction{variant: InstructionVariant::SBC, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::sbc},
    0xE5u8 => Instruction{variant: InstructionVariant::SBC, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::sbc},
    0xF5u8 => Instruction{variant: InstructionVariant::SBC, mode: AddressingMode::ZeroPageX, length: 2, cycles: 4, emu_fn: Emu::sbc},
    0xEDu8 => Instruction{variant: InstructionVariant::SBC, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::sbc},
    0xFDu8 => Instruction{variant: InstructionVariant::SBC, mode: AddressingMode::AbsoluteX, length: 3, cycles: 4, emu_fn: Emu::sbc},
    0xF9u8 => Instruction{variant: InstructionVariant::SBC, mode: AddressingMode::AbsoluteY, length: 3, cycles: 4, emu_fn: Emu::sbc},
    0xE1u8 => Instruction{variant: InstructionVariant::SBC, mode: AddressingMode::IndirectX, length: 2, cycles: 6, emu_fn: Emu::sbc},
    0xF1u8 => Instruction{variant: InstructionVariant::SBC, mode: AddressingMode::IndirectY, length: 2, cycles: 5, emu_fn: Emu::sbc},

    // STA instruction
    0x85u8 => Instruction{variant: InstructionVariant::STA, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::sta},
    0x95u8 => Instruction{variant: InstructionVariant::STA, mode: AddressingMode::ZeroPageX, length: 2, cycles: 4, emu_fn: Emu::sta},
    0x8Du8 => Instruction{variant: InstructionVariant::STA, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::sta},
    0x9Du8 => Instruction{variant: InstructionVariant::STA, mode: AddressingMode::AbsoluteX, length: 3, cycles: 5, emu_fn: Emu::sta},
    0x99u8 => Instruction{variant: InstructionVariant::STA, mode: AddressingMode::AbsoluteY, length: 3, cycles: 5, emu_fn: Emu::sta},
    0x81u8 => Instruction{variant: InstructionVariant::STA, mode: AddressingMode::IndirectX, length: 2, cycles: 6, emu_fn: Emu::sta},
    0x91u8 => Instruction{variant: InstructionVariant::STA, mode: AddressingMode::IndirectY, length: 2, cycles: 6, emu_fn: Emu::sta},

    // Stack instructions
    0x9Au8 => Instruction{variant: InstructionVariant::TXS, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::txs},
    0xBAu8 => Instruction{variant: InstructionVariant::TSX, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::tsx},
    0x48u8 => Instruction{variant: InstructionVariant::PHA, mode: AddressingMode::Implied, length: 1, cycles: 3, emu_fn: Emu::pha},
    0x68u8 => Instruction{variant: InstructionVariant::PLA, mode: AddressingMode::Implied, length: 1, cycles: 4, emu_fn: Emu::pla},
    0x08u8 => Instruction{variant: InstructionVariant::PHP, mode: AddressingMode::Implied, length: 1, cycles: 3, emu_fn: Emu::php},
    0x28u8 => Instruction{variant: InstructionVariant::PLP, mode: AddressingMode::Implied, length: 1, cycles: 4, emu_fn: Emu::plp},

    // STX instruction
    0x86u8 => Instruction{variant: InstructionVariant::STX, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::stx},
    0x96u8 => Instruction{variant: InstructionVariant::STX, mode: AddressingMode::ZeroPageY, length: 2, cycles: 4, emu_fn: Emu::stx},
    0x8Eu8 => Instruction{variant: InstructionVariant::STX, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::stx},

    // STY instruction
    0x84u8 => Instruction{variant: InstructionVariant::STY, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::sty},
    0x94u8 => Instruction{variant: InstructionVariant::STY, mode: AddressingMode::ZeroPageX, length: 2, cycles: 4, emu_fn: Emu::sty},
    0x8Cu8 => Instruction{variant: InstructionVariant::STY, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::sty},

    // Unofficial NOP instructions
    0x04u8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::ZeroPage, length: 2, cycles: 2, emu_fn: Emu::nop},
    0x44u8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::ZeroPage, length: 2, cycles: 2, emu_fn: Emu::nop},
    0x64u8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::ZeroPage, length: 2, cycles: 2, emu_fn: Emu::nop},
    0x0Cu8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::Absolute, length: 3, cycles: 2, emu_fn: Emu::nop},
    0x14u8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::ZeroPageX, length: 2, cycles: 2, emu_fn: Emu::nop},
    0x34u8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::ZeroPageX, length: 2, cycles: 2, emu_fn: Emu::nop},
    0x54u8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::ZeroPageX, length: 2, cycles: 2, emu_fn: Emu::nop},
    0x74u8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::ZeroPageX, length: 2, cycles: 2, emu_fn: Emu::nop},
    0xD4u8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::ZeroPageX, length: 2, cycles: 2, emu_fn: Emu::nop},
    0xF4u8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::ZeroPageX, length: 2, cycles: 2, emu_fn: Emu::nop},
    0x80u8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::nop},
    0x1Cu8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::AbsoluteX, length: 3, cycles: 2, emu_fn: Emu::nop},
    0x3Cu8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::AbsoluteX, length: 3, cycles: 2, emu_fn: Emu::nop},
    0x5Cu8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::AbsoluteX, length: 3, cycles: 2, emu_fn: Emu::nop},
    0x7Cu8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::AbsoluteX, length: 3, cycles: 2, emu_fn: Emu::nop},
    0xDCu8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::AbsoluteX, length: 3, cycles: 2, emu_fn: Emu::nop},
    0xFCu8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::AbsoluteX, length: 3, cycles: 2, emu_fn: Emu::nop},
    0x1Au8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::nop},
    0x3Au8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::nop},
    0x5Au8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::nop},
    0x7Au8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::nop},
    0xDAu8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::nop},
    0xFAu8 => Instruction{variant: InstructionVariant::NOP, mode: AddressingMode::Implied, length: 1, cycles: 2, emu_fn: Emu::nop},

    // Unofficial LAX instruction
    0xA7u8 => Instruction{variant: InstructionVariant::LAX, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::lax},
    0xB7u8 => Instruction{variant: InstructionVariant::LAX, mode: AddressingMode::ZeroPageY, length: 2, cycles: 4, emu_fn: Emu::lax},
    0xAFu8 => Instruction{variant: InstructionVariant::LAX, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::lax},
    0xBFu8 => Instruction{variant: InstructionVariant::LAX, mode: AddressingMode::AbsoluteY, length: 3, cycles: 4, emu_fn: Emu::lax},
    0xA3u8 => Instruction{variant: InstructionVariant::LAX, mode: AddressingMode::IndirectX, length: 2, cycles: 6, emu_fn: Emu::lax},
    0xB3u8 => Instruction{variant: InstructionVariant::LAX, mode: AddressingMode::IndirectY, length: 2, cycles: 5, emu_fn: Emu::lax},

    // Unofficial SAX instruction
    0x87u8 => Instruction{variant: InstructionVariant::SAX, mode: AddressingMode::ZeroPage, length: 2, cycles: 3, emu_fn: Emu::sax},
    0x97u8 => Instruction{variant: InstructionVariant::SAX, mode: AddressingMode::ZeroPageY, length: 2, cycles: 4, emu_fn: Emu::sax},
    0x8Fu8 => Instruction{variant: InstructionVariant::SAX, mode: AddressingMode::Absolute, length: 3, cycles: 4, emu_fn: Emu::sax},
    0x83u8 => Instruction{variant: InstructionVariant::SAX, mode: AddressingMode::IndirectX, length: 2, cycles: 6, emu_fn: Emu::sax},

    // Unofficial SBC instruction
    0xEBu8 => Instruction{variant: InstructionVariant::SBC, mode: AddressingMode::Immediate, length: 2, cycles: 2, emu_fn: Emu::sbc},

    // Unofficial DCP instruction
    0xC7u8 => Instruction{variant: InstructionVariant::DCP, mode: AddressingMode::ZeroPage, length: 2, cycles: 5, emu_fn: Emu::dcp},
    0xD7u8 => Instruction{variant: InstructionVariant::DCP, mode: AddressingMode::ZeroPageX, length: 2, cycles: 6, emu_fn: Emu::dcp},
    0xCFu8 => Instruction{variant: InstructionVariant::DCP, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::dcp},
    0xDFu8 => Instruction{variant: InstructionVariant::DCP, mode: AddressingMode::AbsoluteX, length: 3, cycles: 7, emu_fn: Emu::dcp},
    0xDBu8 => Instruction{variant: InstructionVariant::DCP, mode: AddressingMode::AbsoluteY, length: 3, cycles: 7, emu_fn: Emu::dcp},
    0xC3u8 => Instruction{variant: InstructionVariant::DCP, mode: AddressingMode::IndirectX, length: 2, cycles: 8, emu_fn: Emu::dcp},
    0xD3u8 => Instruction{variant: InstructionVariant::DCP, mode: AddressingMode::IndirectY, length: 2, cycles: 8, emu_fn: Emu::dcp},

    // Unofficial ISB instruction
    0xE7u8 => Instruction{variant: InstructionVariant::ISB, mode: AddressingMode::ZeroPage, length: 2, cycles: 5, emu_fn: Emu::isb},
    0xF7u8 => Instruction{variant: InstructionVariant::ISB, mode: AddressingMode::ZeroPageX, length: 2, cycles: 6, emu_fn: Emu::isb},
    0xEFu8 => Instruction{variant: InstructionVariant::ISB, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::isb},
    0xFFu8 => Instruction{variant: InstructionVariant::ISB, mode: AddressingMode::AbsoluteX, length: 3, cycles: 7, emu_fn: Emu::isb},
    0xFBu8 => Instruction{variant: InstructionVariant::ISB, mode: AddressingMode::AbsoluteY, length: 3, cycles: 7, emu_fn: Emu::isb},
    0xE3u8 => Instruction{variant: InstructionVariant::ISB, mode: AddressingMode::IndirectX, length: 2, cycles: 8, emu_fn: Emu::isb},
    0xF3u8 => Instruction{variant: InstructionVariant::ISB, mode: AddressingMode::IndirectY, length: 2, cycles: 8, emu_fn: Emu::isb},

    // Unofficial SLO instruction
    0x07u8 => Instruction{variant: InstructionVariant::SLO, mode: AddressingMode::ZeroPage, length: 2, cycles: 5, emu_fn: Emu::slo},
    0x17u8 => Instruction{variant: InstructionVariant::SLO, mode: AddressingMode::ZeroPageX, length: 2, cycles: 6, emu_fn: Emu::slo},
    0x0Fu8 => Instruction{variant: InstructionVariant::SLO, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::slo},
    0x1Fu8 => Instruction{variant: InstructionVariant::SLO, mode: AddressingMode::AbsoluteX, length: 3, cycles: 7, emu_fn: Emu::slo},
    0x1Bu8 => Instruction{variant: InstructionVariant::SLO, mode: AddressingMode::AbsoluteY, length: 3, cycles: 7, emu_fn: Emu::slo},
    0x03u8 => Instruction{variant: InstructionVariant::SLO, mode: AddressingMode::IndirectX, length: 2, cycles: 8, emu_fn: Emu::slo},
    0x13u8 => Instruction{variant: InstructionVariant::SLO, mode: AddressingMode::IndirectY, length: 2, cycles: 8, emu_fn: Emu::slo},

    // Unofficial RLA instruction
    0x27u8 => Instruction{variant: InstructionVariant::RLA, mode: AddressingMode::ZeroPage, length: 2, cycles: 5, emu_fn: Emu::rla},
    0x37u8 => Instruction{variant: InstructionVariant::RLA, mode: AddressingMode::ZeroPageX, length: 2, cycles: 6, emu_fn: Emu::rla},
    0x2Fu8 => Instruction{variant: InstructionVariant::RLA, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::rla},
    0x3Fu8 => Instruction{variant: InstructionVariant::RLA, mode: AddressingMode::AbsoluteX, length: 3, cycles: 7, emu_fn: Emu::rla},
    0x3Bu8 => Instruction{variant: InstructionVariant::RLA, mode: AddressingMode::AbsoluteY, length: 3, cycles: 7, emu_fn: Emu::rla},
    0x23u8 => Instruction{variant: InstructionVariant::RLA, mode: AddressingMode::IndirectX, length: 2, cycles: 8, emu_fn: Emu::rla},
    0x33u8 => Instruction{variant: InstructionVariant::RLA, mode: AddressingMode::IndirectY, length: 2, cycles: 8, emu_fn: Emu::rla},

    // Unofficial SRE instruction
    0x47u8 => Instruction{variant: InstructionVariant::SRE, mode: AddressingMode::ZeroPage, length: 2, cycles: 5, emu_fn: Emu::sre},
    0x57u8 => Instruction{variant: InstructionVariant::SRE, mode: AddressingMode::ZeroPageX, length: 2, cycles: 6, emu_fn: Emu::sre},
    0x4Fu8 => Instruction{variant: InstructionVariant::SRE, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::sre},
    0x5Fu8 => Instruction{variant: InstructionVariant::SRE, mode: AddressingMode::AbsoluteX, length: 3, cycles: 7, emu_fn: Emu::sre},
    0x5Bu8 => Instruction{variant: InstructionVariant::SRE, mode: AddressingMode::AbsoluteY, length: 3, cycles: 7, emu_fn: Emu::sre},
    0x43u8 => Instruction{variant: InstructionVariant::SRE, mode: AddressingMode::IndirectX, length: 2, cycles: 8, emu_fn: Emu::sre},
    0x53u8 => Instruction{variant: InstructionVariant::SRE, mode: AddressingMode::IndirectY, length: 2, cycles: 8, emu_fn: Emu::sre},

    // Unofficial RRA instruction
    0x67u8 => Instruction{variant: InstructionVariant::RRA, mode: AddressingMode::ZeroPage, length: 2, cycles: 5, emu_fn: Emu::rra},
    0x77u8 => Instruction{variant: InstructionVariant::RRA, mode: AddressingMode::ZeroPageX, length: 2, cycles: 6, emu_fn: Emu::rra},
    0x6Fu8 => Instruction{variant: InstructionVariant::RRA, mode: AddressingMode::Absolute, length: 3, cycles: 6, emu_fn: Emu::rra},
    0x7Fu8 => Instruction{variant: InstructionVariant::RRA, mode: AddressingMode::AbsoluteX, length: 3, cycles: 7, emu_fn: Emu::rra},
    0x7Bu8 => Instruction{variant: InstructionVariant::RRA, mode: AddressingMode::AbsoluteY, length: 3, cycles: 7, emu_fn: Emu::rra},
    0x63u8 => Instruction{variant: InstructionVariant::RRA, mode: AddressingMode::IndirectX, length: 2, cycles: 8, emu_fn: Emu::rra},
    0x73u8 => Instruction{variant: InstructionVariant::RRA, mode: AddressingMode::IndirectY, length: 2, cycles: 8, emu_fn: Emu::rra},
};
