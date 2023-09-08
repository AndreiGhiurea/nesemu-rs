mod emulator;
mod instructions;
mod registers;
mod trace;

use super::Bus;
use instructions::{AddressingMode, Instruction, InstructionVariant, INSTRUCTIONS};
use registers::Registers;
use trace::Trace;

pub type Addr = u16;

///
/// 6502 Microprocessor
///
pub struct Cpu {
    regs: Registers,
    bus: Bus,
    cycles: usize,
}

impl Cpu {
    pub fn new(bus: Bus) -> Cpu {
        let regs = Registers::default();

        Cpu {
            regs,
            bus,
            cycles: 0,
        }
    }

    pub fn execute(&mut self) {
        let opcode = self.bus.read_u8(self.regs.pc);
        let instruction = Cpu::decode(opcode);

        Trace::print_state(self, instruction);

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
        self.bus.write_u8(addr, op);

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
        self.bus.read_u8(addr)
    }

    fn resolve_adressing(&self, mode: AddressingMode, _cycles: u8) -> Addr {
        match mode {
            AddressingMode::Implied => 0xFFFF,
            AddressingMode::Relative => self.regs.pc,
            AddressingMode::Immediate => self.regs.pc,
            AddressingMode::ZeroPage => {
                let op = self.bus.read_u8(self.regs.pc);
                Addr::from_le_bytes([op, 0x00])
            }
            AddressingMode::ZeroPageX => {
                let mut op = self.bus.read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_x);
                Addr::from_le_bytes([op, 0x00])
            }
            AddressingMode::ZeroPageY => {
                let mut op = self.bus.read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_y);
                Addr::from_le_bytes([op, 0x00])
            }
            AddressingMode::Absolute => self.bus.read_u16(self.regs.pc),
            AddressingMode::AbsoluteX => {
                let addr = self.bus.read_u16(self.regs.pc);
                addr.wrapping_add(self.regs.idx_x as u16)
            }
            AddressingMode::AbsoluteY => {
                let addr = self.bus.read_u16(self.regs.pc);
                addr.wrapping_add(self.regs.idx_y as u16)
            }
            AddressingMode::IndirectX => {
                let op = self.bus.read_u8(self.regs.pc);
                let lb = self
                    .bus
                    .read_u8((op as Addr + self.regs.idx_x as Addr) & 0x00FF);
                let hb = self
                    .bus
                    .read_u8((op as Addr + self.regs.idx_x as Addr + 0x1) & 0x00FF);

                Addr::from_le_bytes([lb, hb])
            }
            AddressingMode::IndirectY => {
                let op = self.bus.read_u8(self.regs.pc);
                let lb = self.bus.read_u8(op as Addr);
                let hb = self.bus.read_u8((op as Addr + 0x1) & 0x00FF);

                let addr = Addr::from_le_bytes([lb, hb]);
                addr.wrapping_add(self.regs.idx_y as u16)
            }
            AddressingMode::Indirect => {
                let indirect_addr = self.bus.read_u16(self.regs.pc);
                let lb: u8;
                let hb: u8;

                // Note:
                // An original 6502 has does not correctly fetch the target address
                // if the indirect address falls on a page boundary (e.g. $xxFF where xx is any value from $00 to $FF).
                // In this case fetches the LSB from $xxFF as expected but takes the MSB from $xx00.
                // This is fixed in some later chips like the 65SC02, but for compatibility programs ensure
                // the indirect vector is not at the end of the page.
                // We're building a NES emulator so we will emulate the classic 6502 behaviour.
                if indirect_addr & 0x00FF == 0x00FF {
                    lb = self.bus.read_u8(indirect_addr);
                    hb = self.bus.read_u8(indirect_addr & 0xFF00);
                } else {
                    (lb, hb) = self.bus.read_u16(indirect_addr).to_le_bytes().into();
                }

                Addr::from_le_bytes([lb, hb])
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
