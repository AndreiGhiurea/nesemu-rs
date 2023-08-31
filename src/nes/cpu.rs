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

    fn print_state(&self, instr: &Instruction) {
        print!("{:#06X} ", self.regs.pc);

        let mut max_length = 3;
        for idx in 0..instr.length {
            print!(
                "{:#04X} ",
                self.bus.borrow().read_u8(self.regs.pc + idx as u16)
            );
            max_length -= 1;
        }

        for _idx in 0..max_length {
            print!("     ");
            max_length -= 1;
        }

        print!("{:?} ", instr.variant);

        print!("                      {}\n", self);
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
            .expect(format!("Unknown opcode {}", opcode).as_str());

        instruction
    }

    fn stack_push_u16(&mut self, op: u16) {
        self.regs.sp -= 2;

        let addr = Addr::from_le_bytes([self.regs.sp, 0x01]);
        self.bus.borrow_mut().write_u16(addr, op);
    }

    fn stack_push_u8(&mut self, op: u8) {
        self.regs.sp -= 1;

        let addr = Addr::from_le_bytes([self.regs.sp, 0x01]);
        self.bus.borrow_mut().write_u8(addr, op);
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let addr = Addr::from_le_bytes([self.regs.sp, 0x01]);
        let res = self.bus.borrow().read_u16(addr);

        self.regs.sp += 2;
        res
    }

    fn stack_pop_u8(&mut self) -> u8 {
        let addr = Addr::from_le_bytes([self.regs.sp, 0x01]);
        let res = self.bus.borrow().read_u8(addr);

        self.regs.sp += 1;
        res
    }

    ///
    /// Convert this to resolve_addressing, and use the bus to read write inside the emulation functions
    ///
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
                let mut op = self.bus.borrow().read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_x);
                let addr = Addr::from_le_bytes([op, 0x00]);
                addr
            }
            AddressingMode::IndirectY => {
                let op = self.bus.borrow().read_u8(self.regs.pc);
                let lb = self.bus.borrow().read_u8(Addr::from_le_bytes([op, 0x00]));
                let addr = Addr::from_le_bytes([lb, self.regs.idx_y]);
                addr
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
            "A:{:#04X} X:{:#04X} Y:{:#04X} P:{:#04X} SP:{:#04X} PPU:  {}, {} CYC:{}",
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
