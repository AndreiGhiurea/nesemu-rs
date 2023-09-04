use std::{cell::RefCell, fmt, rc::Rc};

use bitflags::bitflags;

mod emulator;
mod instructions;
use instructions::{AddressingMode, Instruction, INSTRUCTIONS};

use self::instructions::InstructionVariant;

use super::Bus;

///
/// 6502 Microprocessor
///
pub struct Cpu {
    regs: Registers,
    bus: Rc<RefCell<Bus>>,
    cycles: usize,
}

pub type Addr = u16;

impl Cpu {
    pub fn new(bus: Rc<RefCell<Bus>>) -> Cpu {
        let regs = Registers::default();

        Cpu {
            regs,
            bus,
            cycles: 0,
        }
    }

    fn print_adressing(&self, instr: &Instruction) {
        match instr.mode {
            AddressingMode::Implied => {
                print!("{: <28}", " ");
            }
            AddressingMode::Accumulator => {
                print!("{: <28}", "A");
            }
            AddressingMode::Relative => {
                let op = self.bus.borrow().read_i8(self.regs.pc + 1);
                let mut addr = self.regs.pc + 2;
                addr = addr.wrapping_add_signed(op as i16);
                print!("${: <27}", format!("{:02X}", addr));
            }
            AddressingMode::Immediate => {
                let op = self.bus.borrow().read_u8(self.regs.pc + 1);
                print!("#${: <26}", format!("{:02X}", op));
            }
            AddressingMode::ZeroPage => {
                let op = self.bus.borrow().read_u8(self.regs.pc + 1);
                print!("${: <27}", format!("{:02X}", op));
            }
            AddressingMode::ZeroPageX => {
                let op = self.bus.borrow().read_u8(self.regs.pc + 1);
                print!("${: <25},X", format!("{:02X}", op));
            }
            AddressingMode::ZeroPageY => {
                let op = self.bus.borrow().read_u8(self.regs.pc + 1);
                print!("${: <25},Y", format!("{:02X}", op));
            }
            AddressingMode::Absolute => {
                let addr = self.bus.borrow().read_u16(self.regs.pc + 1);
                print!("${: <27}", format!("{:04X}", addr));
            }
            AddressingMode::AbsoluteX => {
                let addr = self.bus.borrow().read_u16(self.regs.pc + 1);
                print!("${: <25},X", format!("{:04X}", addr));
            }
            AddressingMode::AbsoluteY => {
                let addr = self.bus.borrow().read_u16(self.regs.pc + 1);
                print!("${: <25},Y", format!("{:04X}", addr));
            }
            AddressingMode::IndirectX => {
                let op = self.bus.borrow().read_u8(self.regs.pc + 1);
                print!("(${: <23},X)", format!("{:02X}", op));
            }
            AddressingMode::IndirectY => {
                let op = self.bus.borrow().read_u8(self.regs.pc + 1);
                print!("(${: <23},Y)", format!("{:02X}", op));
            }
            AddressingMode::Indirect => {
                let indirect_addr = self.bus.borrow().read_u16(self.regs.pc + 1);
                let addr = self.bus.borrow().read_u16(indirect_addr);
                print!("(${: <25})", format!("{:02X}", addr));
            }
        }
    }

    fn print_state(&self, instr: &Instruction) {
        print!("{:04X}  ", self.regs.pc);

        let mut max_length = 3;
        for idx in 0..instr.length {
            print!(
                "{:02X} ",
                self.bus.borrow().read_u8(self.regs.pc + idx as u16)
            );
            max_length -= 1;
        }

        for _idx in 0..max_length {
            print!("   ");
            max_length -= 1;
        }

        print!(" {:?} ", instr.variant);

        self.print_adressing(instr);

        print!("{}\n", self);
    }

    pub fn execute(&mut self) {
        let opcode = self.bus.borrow().read_u8(self.regs.pc);
        let instruction = Cpu::decode(opcode);

        self.print_state(instruction);

        self.emulate(instruction);
    }

    pub fn set_pc(&mut self, value: Addr) {
        self.regs.pc = value;
    }

    fn decode(opcode: u8) -> &'static Instruction {
        let instruction = INSTRUCTIONS
            .get(&opcode)
            .unwrap_or_else(|| panic!("Unknown opcode {}", opcode));

        instruction
    }

    fn stack_push_u16(&mut self, op: u16) {
        let bytes = op.to_le_bytes();
        self.stack_push_u8(bytes[1]);
        self.stack_push_u8(bytes[0]);
    }

    fn stack_push_u8(&mut self, op: u8) {
        let addr = Addr::from_le_bytes([self.regs.sp, 0x01]);
        self.bus.borrow_mut().write_u8(addr, op);

        self.regs.sp -= 1;
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let byte1 = self.stack_pop_u8();
        let byte2 = self.stack_pop_u8();

        u16::from_le_bytes([byte1, byte2])
    }

    fn stack_pop_u8(&mut self) -> u8 {
        self.regs.sp += 1;

        let addr = Addr::from_le_bytes([self.regs.sp, 0x01]);
        let res = self.bus.borrow().read_u8(addr);

        res
    }

    fn resolve_adressing(&self, mode: AddressingMode, _cycles: u8) -> Addr {
        match mode {
            AddressingMode::Implied => 0xFFFF,
            AddressingMode::Relative => self.regs.pc,
            AddressingMode::Immediate => self.regs.pc,
            AddressingMode::ZeroPage => {
                let op = self.bus.borrow().read_u8(self.regs.pc);
                Addr::from_le_bytes([op, 0x00])
            }
            AddressingMode::ZeroPageX => {
                let mut op = self.bus.borrow().read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_x);
                Addr::from_le_bytes([op, 0x00])
            }
            AddressingMode::ZeroPageY => {
                let mut op = self.bus.borrow().read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_y);
                Addr::from_le_bytes([op, 0x00])
            }
            AddressingMode::Absolute => {
                let addr = self.bus.borrow().read_u16(self.regs.pc);
                addr
            }
            AddressingMode::AbsoluteX => {
                let mut addr = self.bus.borrow().read_u16(self.regs.pc);
                addr += self.regs.idx_x as u16;
                addr
            }
            AddressingMode::AbsoluteY => {
                let mut addr = self.bus.borrow().read_u16(self.regs.pc);
                addr += self.regs.idx_y as u16;
                addr
            }
            AddressingMode::IndirectX => {
                let op = self.bus.borrow().read_u8(self.regs.pc);
                let addr = self
                    .bus
                    .borrow()
                    .read_u16((op as u16 + self.regs.idx_x as u16) & 0x00FF);

                addr
            }
            AddressingMode::IndirectY => {
                let op = self.bus.borrow().read_u8(self.regs.pc);
                let lb = self.bus.borrow().read_u8(op as Addr);
                let hb = self.bus.borrow().read_u8((op as Addr + 0x1) & 0x00FF);

                let addr = Addr::from_le_bytes([lb, hb]);
                addr.wrapping_add(self.regs.idx_y as u16)
            }
            AddressingMode::Indirect => {
                let indirect_addr = self.bus.borrow().read_u16(self.regs.pc);
                let addr = self.bus.borrow().read_u16(indirect_addr);
                addr
            }
            _ => {
                panic!("Cannot resolve addresing for mode {:?}", mode);
            }
        }
    }

    fn emulate(&mut self, instruction: &Instruction) {
        // Increment PC to get over the instruction opcode
        // so the emu functions can fetch the operand directly.
        self.regs.pc += 1;

        (instruction.emu_fn)(self, instruction);

        self.cycles += instruction.cycles as usize;

        match instruction.variant {
            // These instructions modify the PC directly, no need to add the length.
            InstructionVariant::JMP | InstructionVariant::JSR | InstructionVariant::RTS => {}
            _ => {
                self.regs.pc += instruction.length as u16 - 1;
            }
        }
    }
}

struct Registers {
    pc: u16,                 // Program counter
    sp: u8,                  // Stack pointer
    acc: u8,                 // Accumulator
    idx_x: u8,               // Index register X
    idx_y: u8,               // Index register Y
    status: ProcessorStatus, // Bitfield with various flags
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            pc: 0x0,
            sp: 0xFD,
            acc: 0x0,
            idx_x: 0x0,
            idx_y: 0x0,
            status: ProcessorStatus::from_bits(0x24).unwrap(),
        }
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:  {}, {} CYC:{}",
            self.regs.acc,
            self.regs.idx_x,
            self.regs.idx_y,
            self.regs.status.bits(),
            self.regs.sp,
            0x00,
            0x00,
            self.cycles
        )
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
    struct ProcessorStatus: u8 {
        const CARRY_FLAG = 0x1;
        const ZERO_FLAG = 0x1 << 1;
        const INTERRUPT_DISABLE = 0x1 << 2;
        const DECIMAL_MODE = 0x1 << 3;
        const BREAK_CMD = 0x1 << 4;
        const BREAK_CMD2 = 0x1 << 5;
        const OVERFLOW_FLAG = 0x1 << 6;
        const NEGATIVE_FLAG = 0x1 << 7;
    }
}

impl ProcessorStatus {
    fn set_carry_flag(&mut self, overflow: bool) -> &mut Self {
        self.set(ProcessorStatus::CARRY_FLAG, overflow);

        self
    }

    fn set_zero_flag(&mut self, result: u8) -> &mut Self {
        self.set(ProcessorStatus::ZERO_FLAG, result == 0x0);

        self
    }

    fn set_overflow_flag(&mut self, overflow: bool) -> &mut Self {
        self.set(ProcessorStatus::OVERFLOW_FLAG, overflow);

        self
    }

    fn set_negative_flag(&mut self, result: u8) -> &mut Self {
        self.set(ProcessorStatus::NEGATIVE_FLAG, result & (0x1 << 7) != 0x0);

        self
    }
}
