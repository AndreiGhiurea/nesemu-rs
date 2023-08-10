use std::rc::Rc;

use bitflags::bitflags;

mod instructions;
use instructions::{Instruction, INSTRUCTIONS};

use self::instructions::AddressingMode;
use super::Bus;

///
/// 6502 Microprocessor
///
pub struct Cpu {
    regs: Registers,
    bus: Rc<Bus>,
}

pub type Addr = u16;

impl Cpu {
    pub fn new(bus: Rc<Bus>) -> Cpu {
        let regs = Registers::default();

        Cpu { regs, bus }
    }

    pub fn execute(&mut self) {
        let instruction = Cpu::decode(0x69);
        self.emulate(instruction);
    }

    fn decode(opcode: u8) -> &'static Instruction {
        let instruction = INSTRUCTIONS
            .get(&opcode)
            .expect(format!("Unknown opcode {}", opcode).as_str());

        instruction
    }

    ///
    ///
    ///
    /// Convert this to resolve_addressing, and use the bus to read write inside the emulation functions
    ///
    ///
    ///
    fn fetch_operand(&self, mode: &AddressingMode, cycles: u8) -> u8 {
        match mode {
            AddressingMode::Immediate => self.bus.read_u8(self.regs.pc),
            AddressingMode::ZeroPage => {
                let op = self.bus.read_u8(self.regs.pc);
                let addr = Addr::from_le_bytes([op, 0x00]);
                self.bus.read_u8(addr)
            }
            AddressingMode::ZeroPageX => {
                let mut op = self.bus.read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_x);
                let addr = Addr::from_le_bytes([op, 0x00]);
                self.bus.read_u8(addr)
            }
            AddressingMode::ZeroPageY => {
                let mut op = self.bus.read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_y);
                let addr = Addr::from_le_bytes([op, 0x00]);
                self.bus.read_u8(addr)
            }
            AddressingMode::Absolute => {
                let addr = self.bus.read_u16(self.regs.pc);
                self.bus.read_u8(addr)
            }
            AddressingMode::AbsoluteX => {
                let mut addr = self.bus.read_u16(self.regs.pc);
                addr += self.regs.idx_x as u16;
                self.bus.read_u8(addr)
            }
            AddressingMode::AbsoluteY => {
                let mut addr = self.bus.read_u16(self.regs.pc);
                addr += self.regs.idx_y as u16;
                self.bus.read_u8(addr)
            }
            AddressingMode::IndirectX => {
                let mut op = self.bus.read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_x);
                let addr = Addr::from_le_bytes([op, 0x00]);
                self.bus.read_u8(addr)
            }
            AddressingMode::IndirectY => {
                let op = self.bus.read_u8(self.regs.pc);
                let lb = self.bus.read_u8(Addr::from_le_bytes([op, 0x00]));
                let addr = Addr::from_le_bytes([lb, self.regs.idx_y]);
                self.bus.read_u8(addr)
            }
            AddressingMode::Indirect => {
                // Only used by JMP.
                0xBD
            }
            AddressingMode::Accumulator => self.regs.acc,
        }
    }

    fn write_operand(&mut self, mode: &AddressingMode, operand: u8) {
        match mode {
            AddressingMode::Immediate => (),
            AddressingMode::ZeroPage => {
                let op = self.bus.read_u8(self.regs.pc);
                let addr = Addr::from_le_bytes([op, 0x00]);
                self.bus.write_u8(addr, operand);
            }
            AddressingMode::ZeroPageX => {
                let mut op = self.bus.read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_x);
                let addr = Addr::from_le_bytes([op, 0x00]);
                self.bus.write_u8(addr, operand);
            }
            AddressingMode::ZeroPageY => {
                let mut op = self.bus.read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_y);
                let addr = Addr::from_le_bytes([op, 0x00]);
                self.bus.write_u8(addr, operand);
            }
            AddressingMode::Absolute => {
                let addr = self.bus.read_u16(self.regs.pc);
                self.bus.write_u8(addr, operand);
            }
            AddressingMode::AbsoluteX => {
                let mut addr = self.bus.read_u16(self.regs.pc);
                addr += self.regs.idx_x as u16;
                self.bus.write_u8(addr, operand);
            }
            AddressingMode::AbsoluteY => {
                let mut addr = self.bus.read_u16(self.regs.pc);
                addr += self.regs.idx_y as u16;
                self.bus.write_u8(addr, operand);
            }
            AddressingMode::IndirectX => {
                let mut op = self.bus.read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_x);
                let addr = Addr::from_le_bytes([op, 0x00]);
                self.bus.write_u8(addr, operand);
            }
            AddressingMode::IndirectY => {
                let op = self.bus.read_u8(self.regs.pc);
                let lb = self.bus.read_u8(Addr::from_le_bytes([op, 0x00]));
                let addr = Addr::from_le_bytes([lb, self.regs.idx_y]);
                self.bus.write_u8(addr, operand);
            }
            AddressingMode::Indirect => {
                // Only used by JMP.
                ()
            }
            AddressingMode::Accumulator => self.regs.acc = operand,
        }
    }

    fn emulate(&mut self, instruction: &Instruction) {
        // Increment PC to get over the instruction opcode
        // so the emu functions can fetch the operand
        self.regs.pc += 1;

        match instruction {
            Instruction::ADC(mode, length, cycles) => self.adc(mode, *length, *cycles),
            Instruction::AND(mode, length, cycles) => self.and(mode, *length, *cycles),
            Instruction::ASL(mode, length, cycles) => self.asl(mode, *length, *cycles),
            Instruction::BCC(length, cycles) => self.bcc(*length, *cycles),
            _ => self.unimplemented(instruction),
        }
    }

    fn adc(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op = self.fetch_operand(mode, cycles);

        let carry = self.regs.status.contains(ProcessorStatus::CARRY_FLAG);
        if carry {
            op = op.wrapping_add(0x1);
        }

        let (result, overflow) = self.regs.acc.overflowing_add(op);
        // Set carry flag if overflow occured
        if overflow {
            self.regs.status |= ProcessorStatus::CARRY_FLAG;
        }
        // Set zero flag is result == 0
        if result == 0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }
        // Set overflow flag if result < -128 or > +127
        if result < 0x80 || result > 0xFF {
            self.regs.status |= ProcessorStatus::OVERFLOW_FLAG;
        }
        // Set negative flag if result has bit 7 set
        if result & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        self.regs.acc = result;
        self.regs.pc += length as u16 - 1;
    }

    fn and(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let op = self.fetch_operand(mode, cycles);

        self.regs.acc &= op;
        // Set zero flag is accumulator is zero
        if self.regs.acc == 0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }
        // Set negative flag if result has bit 7 set
        if self.regs.acc & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        self.regs.pc += length as u16 - 1;
    }

    fn asl(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op = self.fetch_operand(mode, cycles);

        // Put bit 7 into carry flag.
        if op & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::CARRY_FLAG;
        } else {
            self.regs.status &= !ProcessorStatus::CARRY_FLAG;
        }

        op <<= 0x1;

        // Set negative flag if result has bit 7 set
        if op & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        // Set zero flag if result is 0
        if op == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if let AddressingMode::Accumulator = mode {
            self.regs.acc = op;
        } else {
            self.write_operand(mode, op);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bcc(&mut self, length: u8, cycles: u8) {
        let op = self.bus.fetch_immediate();

        if !self.regs.status.contains(ProcessorStatus::CARRY_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bcs(&mut self, length: u8, cycles: u8) {
        let op = self.bus.fetch_immediate();

        if self.regs.status.contains(ProcessorStatus::CARRY_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn beq(&mut self, length: u8, cycles: u8) {
        let op = self.bus.fetch_immediate();

        if self.regs.status.contains(ProcessorStatus::ZERO_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bne(&mut self, length: u8, cycles: u8) {
        let op = self.bus.fetch_immediate();

        if !self.regs.status.contains(ProcessorStatus::ZERO_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bmi(&mut self, length: u8, cycles: u8) {
        let op = self.bus.fetch_immediate();

        if self.regs.status.contains(ProcessorStatus::NEGATIVE_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bpl(&mut self, length: u8, cycles: u8) {
        let op = self.bus.fetch_immediate();

        if !self.regs.status.contains(ProcessorStatus::NEGATIVE_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bvs(&mut self, length: u8, cycles: u8) {
        let op = self.bus.fetch_immediate();

        if self.regs.status.contains(ProcessorStatus::OVERFLOW_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bvc(&mut self, length: u8, cycles: u8) {
        let op = self.bus.fetch_immediate();

        if !self.regs.status.contains(ProcessorStatus::OVERFLOW_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bit(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op = self.fetch_operand(mode, cycles);

        op &= self.regs.acc;

        if op == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if op & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        if op & 0x40 != 0x0 {
            self.regs.status |= ProcessorStatus::OVERFLOW_FLAG;
        }

        self.regs.pc += length as u16 - 1;
    }

    fn clc(&mut self, length: u8, cycles: u8) {
        self.regs.status &= !ProcessorStatus::CARRY_FLAG;
    }

    fn cld(&mut self, length: u8, cycles: u8) {
        self.regs.status &= !ProcessorStatus::DECIMAL_MODE;
    }

    fn cli(&mut self, length: u8, cycles: u8) {
        self.regs.status &= !ProcessorStatus::INTERRUPT_DISABLE;
    }

    fn clv(&mut self, length: u8, cycles: u8) {
        self.regs.status &= !ProcessorStatus::OVERFLOW_FLAG;
    }

    fn cmp(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let op = self.fetch_operand(mode, cycles);

        let res = self.regs.acc - op;
        if res >= 0x0 {
            self.regs.status |= ProcessorStatus::CARRY_FLAG;
        }

        if res == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if res & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        self.regs.pc += length as u16 - 1;
    }

    fn cmx(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let op = self.fetch_operand(mode, cycles);

        let res = self.regs.idx_x - op;
        if res >= 0x0 {
            self.regs.status |= ProcessorStatus::CARRY_FLAG;
        }

        if res == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if res & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        self.regs.pc += length as u16 - 1;
    }

    fn cmy(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let op = self.fetch_operand(mode, cycles);

        let res = self.regs.idx_y - op;
        if res >= 0x0 {
            self.regs.status |= ProcessorStatus::CARRY_FLAG;
        }

        if res == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if res & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        self.regs.pc += length as u16 - 1;
    }

    fn dec(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op = self.fetch_operand(mode, cycles);

        op -= 1;

        if op == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if op & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        self.write_operand(mode, op);

        self.regs.pc += length as u16 - 1;
    }

    fn dex(&mut self, length: u8, cycles: u8) {
        self.regs.idx_x -= 1;

        if self.regs.idx_x == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.idx_x & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }
    }

    fn dey(&mut self, length: u8, cycles: u8) {
        self.regs.idx_y -= 1;

        if self.regs.idx_y == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.idx_y & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }
    }

    fn eor(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let op = self.fetch_operand(mode, cycles);

        self.regs.acc ^= op;

        if self.regs.acc == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.acc & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        self.regs.pc += length as u16 - 1;
    }

    fn inc(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op = self.fetch_operand(mode, cycles);

        op += 1;

        if op == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if op & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        self.write_operand(mode, op);

        self.regs.pc += length as u16 - 1;
    }

    fn inx(&mut self, length: u8, cycles: u8) {
        self.regs.idx_x += 1;

        if self.regs.idx_x == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.idx_x & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }
    }

    fn iny(&mut self, length: u8, cycles: u8) {
        self.regs.idx_y += 1;

        if self.regs.idx_y == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.idx_y & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }
    }

    fn jmp(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr: Addr;
        if let AddressingMode::Absolute = mode {
            addr = self.bus.read_u16(self.regs.pc);
        } else {
            // Indirect mode, only used by JMP.
            let op = self.bus.read_u16(self.regs.pc);
            addr = self.bus.read_u16(op);
        }

        self.regs.pc = addr;
    }

    fn jsr(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let op = self.bus.read_u16(self.regs.pc);

        self.regs.pc += length as u16;

        let stack_addr = Addr::from_le_bytes([self.regs.sp, 0x01]);
        self.bus.write_u16(stack_addr, self.regs.pc);

        self.regs.sp -= 2;

        self.regs.pc = op;
    }

    fn lda(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let op = self.fetch_operand(mode, cycles);

        self.regs.acc = op;
        if self.regs.acc == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.acc & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        self.regs.pc += length as u16 - 1;
    }

    fn ldx(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op = self.fetch_operand(mode, cycles);

        self.regs.idx_x = op;
        if self.regs.idx_x == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.idx_x & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        self.regs.pc += length as u16 - 1;
    }

    fn ldy(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op = self.fetch_operand(mode, cycles);

        self.regs.idx_y = op;
        if self.regs.idx_y == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.idx_y & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        self.regs.pc += length as u16 - 1;
    }

    fn lsr(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op = self.fetch_operand(mode, cycles);

        // Put bit 0 into carry flag.
        if op & 0x1 != 0x0 {
            self.regs.status |= ProcessorStatus::CARRY_FLAG;
        } else {
            self.regs.status &= !ProcessorStatus::CARRY_FLAG;
        }

        op >>= 0x1;

        // Set negative flag if result has bit 7 set
        if op & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        // Set zero flag if result is 0
        if op == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if let AddressingMode::Accumulator = mode {
            self.regs.acc = op;
        } else {
            self.write_operand(mode, op);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn nop(&mut self, length: u8, cycles: u8) {
        // *cracks open a cold one*
    }

    fn ora(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op = self.fetch_operand(mode, cycles);

        self.regs.acc |= op;

        // Set negative flag if result has bit 7 set
        if self.regs.acc & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        // Set zero flag if result is 0
        if self.regs.acc == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        self.regs.pc += length as u16 - 1;
    }

    fn pha(&mut self, length: u8, cycles: u8) {
        self.regs.sp -= 1;
        let stack_addr = Addr::from_le_bytes([self.regs.sp, 0x01]);

        self.bus.write_u8(stack_addr, self.regs.acc);
    }

    fn php(&mut self, length: u8, cycles: u8) {
        self.regs.sp -= 1;
        let stack_addr = Addr::from_le_bytes([self.regs.sp, 0x01]);

        self.bus.write_u8(stack_addr, self.regs.status.bits());
    }

    fn pla(&mut self, length: u8, cycles: u8) {
        let stack_addr = Addr::from_le_bytes([self.regs.sp, 0x01]);
        self.regs.acc = self.bus.read_u8(stack_addr);
        self.regs.sp += 1;

        // Set negative flag if result has bit 7 set
        if self.regs.acc & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        // Set zero flag if result is 0
        if self.regs.acc == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }
    }

    fn plp(&mut self, length: u8, cycles: u8) {
        let stack_addr = Addr::from_le_bytes([self.regs.sp, 0x01]);
        let new_status = self.bus.read_u8(stack_addr);
        self.regs.sp += 1;

        self.regs.status = ProcessorStatus::from_bits(new_status).unwrap();
    }

    fn rol(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op = self.fetch_operand(mode, cycles);

        // Save current carry flag
        let is_current_carry_set = self.regs.status.contains(ProcessorStatus::CARRY_FLAG);

        // Put bit 7 into carry flag.
        if op & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::CARRY_FLAG;
        } else {
            self.regs.status &= !ProcessorStatus::CARRY_FLAG;
        }

        op <<= 0x1;

        if is_current_carry_set {
            op |= 0x1;
        }

        // Set negative flag if result has bit 7 set
        if op & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        // Set zero flag if result is 0
        if op == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if let AddressingMode::Accumulator = mode {
            self.regs.acc = op;
        } else {
            self.write_operand(mode, op);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn ror(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op = self.fetch_operand(mode, cycles);

        // Save current carry flag
        let is_current_carry_set = self.regs.status.contains(ProcessorStatus::CARRY_FLAG);

        // Put bit 0 into carry flag.
        if op & 0x1 != 0x0 {
            self.regs.status |= ProcessorStatus::CARRY_FLAG;
        } else {
            self.regs.status &= !ProcessorStatus::CARRY_FLAG;
        }

        op >>= 0x1;

        if is_current_carry_set {
            op |= 0x1 << 7;
        }

        // Set negative flag if result has bit 7 set
        if op & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        // Set zero flag if result is 0
        if op == 0x0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if let AddressingMode::Accumulator = mode {
            self.regs.acc = op;
        } else {
            self.write_operand(mode, op);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn rti(&mut self, length: u8, cycles: u8) {
        // Return from interrupt
        self.unimplemented(&Instruction::RTI(1, 1));
    }

    fn rts(&mut self, length: u8, cycles: u8) {
        let stack_addr = Addr::from_le_bytes([self.regs.sp, 0x01]);
        self.regs.pc = self.bus.read_u16(stack_addr);

        self.regs.sp += 2;
    }

    fn sbc(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op = self.fetch_operand(mode, cycles);

        let carry = self.regs.status.contains(ProcessorStatus::CARRY_FLAG);
        if !carry {
            op = op.wrapping_sub(0x1);
        }

        let (result, overflow) = self.regs.acc.overflowing_sub(op);
        // Set carry flag if overflow occured
        if overflow {
            self.regs.status &= !ProcessorStatus::CARRY_FLAG;
        }
        // Set zero flag is result == 0
        if result == 0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }
        // Set overflow flag if result < -128 or > +127
        if result < 0x80 || result > 0xFF {
            self.regs.status |= ProcessorStatus::OVERFLOW_FLAG;
        }
        // Set negative flag if result has bit 7 set
        if result & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }

        self.regs.acc = result;
        self.regs.pc += length as u16 - 1;
    }

    fn sec(&mut self, length: u8, cycles: u8) {
        self.regs.status |= ProcessorStatus::CARRY_FLAG;
    }

    fn sed(&mut self, length: u8, cycles: u8) {
        self.regs.status |= ProcessorStatus::DECIMAL_MODE;
    }

    fn sei(&mut self, length: u8, cycles: u8) {
        self.regs.status |= ProcessorStatus::INTERRUPT_DISABLE;
    }

    fn sta(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        self.write_operand(mode, self.regs.acc);
        self.regs.pc += length as u16 - 1;
    }

    fn stx(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        self.write_operand(mode, self.regs.idx_x);
        self.regs.pc += length as u16 - 1;
    }

    fn sty(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        self.write_operand(mode, self.regs.idx_y);
        self.regs.pc += length as u16 - 1;
    }

    fn tax(&mut self, length: u8, cycles: u8) {
        self.regs.idx_x = self.regs.acc;

        if self.regs.idx_x == 0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.idx_x & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }
    }

    fn tay(&mut self, length: u8, cycles: u8) {
        self.regs.idx_y = self.regs.acc;

        if self.regs.idx_y == 0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.idx_y & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }
    }

    fn tsx(&mut self, length: u8, cycles: u8) {
        self.regs.idx_x = self.regs.sp;

        if self.regs.idx_x == 0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.idx_x & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }
    }

    fn txa(&mut self, length: u8, cycles: u8) {
        self.regs.acc = self.regs.idx_x;

        if self.regs.acc == 0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.acc & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }
    }

    fn txs(&mut self, length: u8, cycles: u8) {
        self.regs.sp = self.regs.idx_x;
    }

    fn tya(&mut self, length: u8, cycles: u8) {
        self.regs.acc = self.regs.idx_y;

        if self.regs.acc == 0 {
            self.regs.status |= ProcessorStatus::ZERO_FLAG;
        }

        if self.regs.acc & (0x1 << 7) != 0x0 {
            self.regs.status |= ProcessorStatus::NEGATIVE_FLAG;
        }
    }

    fn unimplemented(&self, instruction: &Instruction) {
        panic!("Instruction unimplemented {:?}", instruction);
    }
}

#[derive(Default)]
struct Registers {
    pc: u16,                 // Program counter
    sp: u8,                  // Stack pointer
    acc: u8,                 // Accumulator
    idx_x: u8,               // Index register X
    idx_y: u8,               // Index register Y
    status: ProcessorStatus, // Bitfield with various flags
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
    struct ProcessorStatus: u8 {
        const CARRY_FLAG = 0x1;
        const ZERO_FLAG = 0x1 << 1;
        const INTERRUPT_DISABLE = 0x1 << 2;
        const DECIMAL_MODE = 0x1 << 3;
        const BREAK_CMD = 0x1 << 4;
        const OVERFLOW_FLAG = 0x1 << 5;
        const NEGATIVE_FLAG = 0x1 << 6;
    }
}
