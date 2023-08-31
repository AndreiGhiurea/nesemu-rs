use std::{cell::RefCell, rc::Rc};

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
    bus: Rc<RefCell<Bus>>,
}

pub type Addr = u16;

impl Cpu {
    pub fn new(bus: Rc<RefCell<Bus>>) -> Cpu {
        let regs = Registers::default();

        Cpu { regs, bus }
    }

    pub fn execute(&mut self) {
        let opcode = self.bus.borrow().read_u8(self.regs.pc);
        let instruction = Cpu::decode(opcode);

        println!("{:#04x}   {:?}", self.regs.pc, instruction);

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
    fn resolve_adressing(&self, mode: &AddressingMode, _cycles: u8) -> Addr {
        match mode {
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

        match instruction {
            Instruction::ADC(mode, length, cycles) => self.adc(mode, *length, *cycles),
            Instruction::AND(mode, length, cycles) => self.and(mode, *length, *cycles),
            Instruction::ASL(mode, length, cycles) => self.asl(mode, *length, *cycles),
            Instruction::BIT(mode, length, cycles) => self.bit(mode, *length, *cycles),
            Instruction::BCC(length, cycles) => self.bcc(*length, *cycles),
            Instruction::BPL(length, cycles) => self.bpl(*length, *cycles),
            Instruction::BMI(length, cycles) => self.bmi(*length, *cycles),
            Instruction::BVC(length, cycles) => self.bvc(*length, *cycles),
            Instruction::BVS(length, cycles) => self.bvs(*length, *cycles),
            Instruction::BCS(length, cycles) => self.bcs(*length, *cycles),
            Instruction::BNE(length, cycles) => self.bne(*length, *cycles),
            Instruction::BEQ(length, cycles) => self.beq(*length, *cycles),
            Instruction::BRK(length, cycles) => self.brk(*length, *cycles),
            Instruction::CMP(mode, length, cycles) => self.cmp(mode, *length, *cycles),
            Instruction::CPX(mode, length, cycles) => self.cpx(mode, *length, *cycles),
            Instruction::CPY(mode, length, cycles) => self.cpy(mode, *length, *cycles),
            Instruction::DEC(mode, length, cycles) => self.dec(mode, *length, *cycles),
            Instruction::EOR(mode, length, cycles) => self.eor(mode, *length, *cycles),
            Instruction::CLC(length, cycles) => self.clc(*length, *cycles),
            Instruction::SEC(length, cycles) => self.sec(*length, *cycles),
            Instruction::CLI(length, cycles) => self.cli(*length, *cycles),
            Instruction::SEI(length, cycles) => self.sei(*length, *cycles),
            Instruction::CLV(length, cycles) => self.clv(*length, *cycles),
            Instruction::CLD(length, cycles) => self.cld(*length, *cycles),
            Instruction::SED(length, cycles) => self.sed(*length, *cycles),
            Instruction::INC(mode, length, cycles) => self.inc(mode, *length, *cycles),
            Instruction::JMP(mode, length, cycles) => self.jmp(mode, *length, *cycles),
            Instruction::JSR(mode, length, cycles) => self.jsr(mode, *length, *cycles),
            Instruction::LDA(mode, length, cycles) => self.lda(mode, *length, *cycles),
            Instruction::LDX(mode, length, cycles) => self.ldx(mode, *length, *cycles),
            Instruction::LDY(mode, length, cycles) => self.ldy(mode, *length, *cycles),
            Instruction::LSR(mode, length, cycles) => self.lsr(mode, *length, *cycles),
            Instruction::NOP(length, cycles) => self.nop(*length, *cycles),
            Instruction::ORA(mode, length, cycles) => self.ora(mode, *length, *cycles),
            Instruction::TAX(length, cycles) => self.tax(*length, *cycles),
            Instruction::TXA(length, cycles) => self.txa(*length, *cycles),
            Instruction::DEX(length, cycles) => self.dex(*length, *cycles),
            Instruction::INX(length, cycles) => self.inx(*length, *cycles),
            Instruction::TAY(length, cycles) => self.tay(*length, *cycles),
            Instruction::TYA(length, cycles) => self.tya(*length, *cycles),
            Instruction::DEY(length, cycles) => self.dey(*length, *cycles),
            Instruction::INY(length, cycles) => self.iny(*length, *cycles),
            Instruction::ROL(mode, length, cycles) => self.rol(mode, *length, *cycles),
            Instruction::ROR(mode, length, cycles) => self.ror(mode, *length, *cycles),
            Instruction::RTI(length, cycles) => self.rti(*length, *cycles),
            Instruction::RTS(length, cycles) => self.rts(*length, *cycles),
            Instruction::SBC(mode, length, cycles) => self.sbc(mode, *length, *cycles),
            Instruction::STA(mode, length, cycles) => self.sta(mode, *length, *cycles),
            Instruction::TXS(length, cycles) => self.txs(*length, *cycles),
            Instruction::TSX(length, cycles) => self.tsx(*length, *cycles),
            Instruction::PHA(length, cycles) => self.pha(*length, *cycles),
            Instruction::PLA(length, cycles) => self.pla(*length, *cycles),
            Instruction::PHP(length, cycles) => self.php(*length, *cycles),
            Instruction::PLP(length, cycles) => self.plp(*length, *cycles),
            Instruction::STX(mode, length, cycles) => self.stx(mode, *length, *cycles),
            Instruction::STY(mode, length, cycles) => self.sty(mode, *length, *cycles),
        }
    }

    fn adc(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let mut op = self.bus.borrow().read_u8(addr);

        let carry = self.regs.status.contains(ProcessorStatus::CARRY_FLAG);
        if carry {
            op = op.wrapping_add(0x1);
        }

        let (result, overflow) = self.regs.acc.overflowing_add(op);
        self.regs
            .status
            .set_carry_flag(overflow)
            .set_zero_flag(result)
            .set_overflow_flag(result)
            .set_negative_flag(result);

        self.regs.acc = result;
        self.regs.pc += length as u16 - 1;
    }

    fn and(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let op = self.bus.borrow().read_u8(addr);

        self.regs.acc &= op;

        self.regs
            .status
            .set_zero_flag(self.regs.acc)
            .set_negative_flag(self.regs.acc);

        self.regs.pc += length as u16 - 1;
    }

    fn asl(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op: u8;
        if let AddressingMode::Accumulator = mode {
            op = self.regs.acc;
        } else {
            let addr = self.resolve_adressing(mode, cycles);
            op = self.bus.borrow().read_u8(addr);
        }

        // Put bit 7 into carry flag.
        let is_bit_set = op & (0x1 << 7) != 0;
        self.regs
            .status
            .set(ProcessorStatus::CARRY_FLAG, is_bit_set);

        op <<= 0x1;

        self.regs.status.set_negative_flag(op).set_zero_flag(op);

        if let AddressingMode::Accumulator = mode {
            self.regs.acc = op;
        } else {
            let addr = self.resolve_adressing(mode, cycles);
            self.bus.borrow_mut().write_u8(addr, op);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bcc(&mut self, length: u8, _cycles: u8) {
        let op = self.bus.borrow().read_i8(self.regs.pc);

        if !self.regs.status.contains(ProcessorStatus::CARRY_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bcs(&mut self, length: u8, _cycles: u8) {
        let op = self.bus.borrow().read_i8(self.regs.pc);

        if self.regs.status.contains(ProcessorStatus::CARRY_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn beq(&mut self, length: u8, _cycles: u8) {
        let op = self.bus.borrow().read_i8(self.regs.pc);

        if self.regs.status.contains(ProcessorStatus::ZERO_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn brk(&mut self, _length: u8, _cycles: u8) {
        self.unimplemented(&Instruction::BRK(1, 1));
    }

    fn bne(&mut self, length: u8, _cycles: u8) {
        let op = self.bus.borrow().read_i8(self.regs.pc);

        if !self.regs.status.contains(ProcessorStatus::ZERO_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bmi(&mut self, length: u8, _cycles: u8) {
        let op = self.bus.borrow().read_i8(self.regs.pc);

        if self.regs.status.contains(ProcessorStatus::NEGATIVE_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bpl(&mut self, length: u8, _cycles: u8) {
        let op = self.bus.borrow().read_i8(self.regs.pc);

        if !self.regs.status.contains(ProcessorStatus::NEGATIVE_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bvs(&mut self, length: u8, _cycles: u8) {
        let op = self.bus.borrow().read_i8(self.regs.pc);

        if self.regs.status.contains(ProcessorStatus::OVERFLOW_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bvc(&mut self, length: u8, _cycles: u8) {
        let op = self.bus.borrow().read_i8(self.regs.pc);

        if !self.regs.status.contains(ProcessorStatus::OVERFLOW_FLAG) {
            self.regs.pc = self.regs.pc.wrapping_add_signed(op as i16);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn bit(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let mut op = self.bus.borrow().read_u8(addr);

        let is_neg_set = op & (0x1 << 7) != 0x0;
        let is_of_set = op & (0x1 << 6) != 0x0;

        op &= self.regs.acc;

        self.regs.status.set_zero_flag(op);

        //
        // BIT instruction treats these 2 flags differently.
        // Bit 6 and 7 of the memory location BEOFRE and is stored in these 2 flags.
        //
        self.regs
            .status
            .set(ProcessorStatus::NEGATIVE_FLAG, is_neg_set);
        self.regs
            .status
            .set(ProcessorStatus::OVERFLOW_FLAG, is_of_set);

        self.regs.pc += length as u16 - 1;
    }

    fn clc(&mut self, _length: u8, _cycles: u8) {
        self.regs.status.set(ProcessorStatus::CARRY_FLAG, false);
    }

    fn cld(&mut self, _length: u8, _cycles: u8) {
        self.regs.status.set(ProcessorStatus::DECIMAL_MODE, false);
    }

    fn cli(&mut self, _length: u8, _cycles: u8) {
        self.regs
            .status
            .set(ProcessorStatus::INTERRUPT_DISABLE, false);
    }

    fn clv(&mut self, _length: u8, _cycles: u8) {
        self.regs.status.set(ProcessorStatus::OVERFLOW_FLAG, false);
    }

    fn cmp(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let op = self.bus.borrow().read_u8(addr);

        let (res, overflow) = self.regs.acc.overflowing_sub(op);

        self.regs
            .status
            .set_carry_flag(overflow)
            .set_zero_flag(res)
            .set_negative_flag(res);

        self.regs.pc += length as u16 - 1;
    }

    fn cpx(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let op = self.bus.borrow().read_u8(addr);

        let (res, overflow) = self.regs.idx_x.overflowing_sub(op);

        self.regs
            .status
            .set_carry_flag(overflow)
            .set_zero_flag(res)
            .set_negative_flag(res);

        self.regs.pc += length as u16 - 1;
    }

    fn cpy(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let op = self.bus.borrow().read_u8(addr);

        let (res, overflow) = self.regs.idx_y.overflowing_sub(op);

        self.regs
            .status
            .set_carry_flag(overflow)
            .set_zero_flag(res)
            .set_negative_flag(res);

        self.regs.pc += length as u16 - 1;
    }

    fn dec(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let mut op = self.bus.borrow().read_u8(addr);

        op -= 1;

        self.regs.status.set_zero_flag(op).set_negative_flag(op);

        self.bus.borrow_mut().write_u8(addr, op);

        self.regs.pc += length as u16 - 1;
    }

    fn dex(&mut self, length: u8, _cycles: u8) {
        self.regs.idx_x -= 1;

        self.regs
            .status
            .set_zero_flag(self.regs.idx_x)
            .set_negative_flag(self.regs.idx_x);

        self.regs.pc += length as u16 - 1;
    }

    fn dey(&mut self, length: u8, _cycles: u8) {
        self.regs.idx_y -= 1;

        self.regs
            .status
            .set_zero_flag(self.regs.idx_y)
            .set_negative_flag(self.regs.idx_y);

        self.regs.pc += length as u16 - 1;
    }

    fn eor(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let op = self.bus.borrow().read_u8(addr);

        self.regs.acc ^= op;

        self.regs
            .status
            .set_zero_flag(self.regs.acc)
            .set_negative_flag(self.regs.acc);

        self.regs.pc += length as u16 - 1;
    }

    fn inc(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let mut op = self.bus.borrow().read_u8(addr);

        op += 1;

        self.regs.status.set_zero_flag(op).set_negative_flag(op);

        self.bus.borrow_mut().write_u8(addr, op);

        self.regs.pc += length as u16 - 1;
    }

    fn inx(&mut self, length: u8, _cycles: u8) {
        self.regs.idx_x += 1;

        self.regs
            .status
            .set_zero_flag(self.regs.idx_x)
            .set_negative_flag(self.regs.idx_x);

        self.regs.pc += length as u16 - 1;
    }

    fn iny(&mut self, length: u8, _cycles: u8) {
        self.regs.idx_y += 1;

        self.regs
            .status
            .set_zero_flag(self.regs.idx_y)
            .set_negative_flag(self.regs.idx_y);

        self.regs.pc += length as u16 - 1;
    }

    fn jmp(&mut self, mode: &AddressingMode, _length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);

        self.regs.pc = addr;
    }

    fn jsr(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);

        self.regs.pc += length as u16 - 1;
        self.stack_push_u16(self.regs.pc);

        self.regs.pc = addr;
    }

    fn lda(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let op = self.bus.borrow().read_u8(addr);

        self.regs.acc = op;
        self.regs.status.set_zero_flag(op).set_negative_flag(op);

        self.regs.pc += length as u16 - 1;
    }

    fn ldx(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let op = self.bus.borrow().read_u8(addr);

        self.regs.idx_x = op;
        self.regs.status.set_zero_flag(op).set_negative_flag(op);

        self.regs.pc += length as u16 - 1;
    }

    fn ldy(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let op = self.bus.borrow().read_u8(addr);

        self.regs.idx_y = op;
        self.regs.status.set_zero_flag(op).set_negative_flag(op);

        self.regs.pc += length as u16 - 1;
    }

    fn lsr(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op: u8;
        if let AddressingMode::Accumulator = mode {
            op = self.regs.acc;
        } else {
            let addr = self.resolve_adressing(mode, cycles);
            op = self.bus.borrow().read_u8(addr);
        }

        // Put bit 0 into carry flag.
        let is_bit_set = op & 0x1 != 0x0;
        self.regs
            .status
            .set(ProcessorStatus::CARRY_FLAG, is_bit_set);

        op >>= 0x1;

        self.regs.status.set_negative_flag(op).set_zero_flag(op);

        if let AddressingMode::Accumulator = mode {
            self.regs.acc = op;
        } else {
            let addr = self.resolve_adressing(mode, cycles);
            self.bus.borrow_mut().write_u8(addr, op);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn nop(&mut self, _length: u8, _cycles: u8) {
        // *cracks open a cold one*
    }

    fn ora(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let op = self.bus.borrow().read_u8(addr);

        self.regs.acc |= op;

        self.regs
            .status
            .set_zero_flag(self.regs.acc)
            .set_negative_flag(self.regs.acc);

        self.regs.pc += length as u16 - 1;
    }

    fn pha(&mut self, _length: u8, _cycles: u8) {
        self.stack_push_u8(self.regs.acc);
    }

    fn php(&mut self, _length: u8, _cycles: u8) {
        self.stack_push_u8(self.regs.status.bits());
    }

    fn pla(&mut self, _length: u8, _cycles: u8) {
        self.regs.acc = self.stack_pop_u8();

        self.regs
            .status
            .set_zero_flag(self.regs.acc)
            .set_negative_flag(self.regs.acc);
    }

    fn plp(&mut self, _length: u8, _cycles: u8) {
        let new_status = self.stack_pop_u8();
        self.regs.status = ProcessorStatus::from_bits(new_status).unwrap();
    }

    fn rol(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op: u8;
        if let AddressingMode::Accumulator = mode {
            op = self.regs.acc;
        } else {
            let addr = self.resolve_adressing(mode, cycles);
            op = self.bus.borrow().read_u8(addr);
        }

        // Save current carry flag
        let is_current_carry_set = self.regs.status.contains(ProcessorStatus::CARRY_FLAG);

        let is_bit_set = op & (0x1 << 7) != 0x0;
        self.regs
            .status
            .set(ProcessorStatus::CARRY_FLAG, is_bit_set);

        op <<= 0x1;

        if is_current_carry_set {
            op |= 0x1;
        }

        self.regs.status.set_negative_flag(op).set_zero_flag(op);

        if let AddressingMode::Accumulator = mode {
            self.regs.acc = op;
        } else {
            let addr = self.resolve_adressing(mode, cycles);
            self.bus.borrow_mut().write_u8(addr, op);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn ror(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let mut op: u8;
        if let AddressingMode::Accumulator = mode {
            op = self.regs.acc;
        } else {
            let addr = self.resolve_adressing(mode, cycles);
            op = self.bus.borrow().read_u8(addr);
        }

        // Save current carry flag
        let is_current_carry_set = self.regs.status.contains(ProcessorStatus::CARRY_FLAG);

        let is_bit_set = op & 0x1 != 0x0;
        self.regs
            .status
            .set(ProcessorStatus::CARRY_FLAG, is_bit_set);

        op >>= 0x1;

        if is_current_carry_set {
            op |= 0x1 << 7;
        }

        self.regs.status.set_negative_flag(op).set_zero_flag(op);

        if let AddressingMode::Accumulator = mode {
            self.regs.acc = op;
        } else {
            let addr = self.resolve_adressing(mode, cycles);
            self.bus.borrow_mut().write_u8(addr, op);
        }

        self.regs.pc += length as u16 - 1;
    }

    fn rti(&mut self, _length: u8, _cycles: u8) {
        // Return from interrupt
        self.unimplemented(&Instruction::RTI(1, 1));
    }

    fn rts(&mut self, _length: u8, _cycles: u8) {
        self.regs.pc = self.stack_pop_u16();
    }

    fn sbc(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        let mut op = self.bus.borrow().read_u8(addr);

        let carry = self.regs.status.contains(ProcessorStatus::CARRY_FLAG);
        if !carry {
            op = op.wrapping_sub(0x1);
        }

        let (result, overflow) = self.regs.acc.overflowing_sub(op);

        self.regs
            .status
            .set_carry_flag(overflow)
            .set_zero_flag(result)
            .set_overflow_flag(result)
            .set_negative_flag(result);

        self.regs.acc = result;
        self.regs.pc += length as u16 - 1;
    }

    fn sec(&mut self, _length: u8, _cycles: u8) {
        self.regs.status.set(ProcessorStatus::CARRY_FLAG, true);
    }

    fn sed(&mut self, _length: u8, _cycles: u8) {
        self.regs.status.set(ProcessorStatus::DECIMAL_MODE, true);
    }

    fn sei(&mut self, _length: u8, _cycles: u8) {
        self.regs
            .status
            .set(ProcessorStatus::INTERRUPT_DISABLE, true);
    }

    fn sta(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        self.bus.borrow_mut().write_u8(addr, self.regs.acc);

        self.regs.pc += length as u16 - 1;
    }

    fn stx(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        self.bus.borrow_mut().write_u8(addr, self.regs.idx_x);

        self.regs.pc += length as u16 - 1;
    }

    fn sty(&mut self, mode: &AddressingMode, length: u8, cycles: u8) {
        let addr = self.resolve_adressing(mode, cycles);
        self.bus.borrow_mut().write_u8(addr, self.regs.idx_y);

        self.regs.pc += length as u16 - 1;
    }

    fn tax(&mut self, _length: u8, _cycles: u8) {
        self.regs.idx_x = self.regs.acc;

        self.regs
            .status
            .set_zero_flag(self.regs.idx_x)
            .set_negative_flag(self.regs.idx_x);
    }

    fn tay(&mut self, _length: u8, _cycles: u8) {
        self.regs.idx_y = self.regs.acc;

        self.regs
            .status
            .set_zero_flag(self.regs.idx_y)
            .set_negative_flag(self.regs.idx_y);
    }

    fn tsx(&mut self, _length: u8, _cycles: u8) {
        self.regs.idx_x = self.regs.sp;

        self.regs
            .status
            .set_zero_flag(self.regs.idx_x)
            .set_negative_flag(self.regs.idx_x);
    }

    fn txa(&mut self, _length: u8, _cycles: u8) {
        self.regs.acc = self.regs.idx_x;

        self.regs
            .status
            .set_zero_flag(self.regs.acc)
            .set_negative_flag(self.regs.acc);
    }

    fn txs(&mut self, _length: u8, _cycles: u8) {
        self.regs.sp = self.regs.idx_x;
    }

    fn tya(&mut self, _length: u8, _cycles: u8) {
        self.regs.acc = self.regs.idx_y;

        self.regs
            .status
            .set_zero_flag(self.regs.acc)
            .set_negative_flag(self.regs.acc);
    }

    fn unimplemented(&self, instruction: &Instruction) {
        panic!("Instruction unimplemented {:?}", instruction);
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
            status: ProcessorStatus::from_bits(0x34).unwrap(),
        }
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
        const OVERFLOW_FLAG = 0x1 << 5;
        const NEGATIVE_FLAG = 0x1 << 6;
    }
}

impl ProcessorStatus {
    fn set_carry_flag(&mut self, overflow: bool) -> &mut Self {
        if overflow {
            self.set(ProcessorStatus::CARRY_FLAG, true);
        }

        self
    }

    fn set_zero_flag(&mut self, result: u8) -> &mut Self {
        if result == 0 {
            self.set(ProcessorStatus::ZERO_FLAG, true);
        }

        self
    }

    fn set_overflow_flag(&mut self, result: u8) -> &mut Self {
        if result < 0x80 || result > 0xFF {
            self.set(ProcessorStatus::OVERFLOW_FLAG, true);
        }

        self
    }

    fn set_negative_flag(&mut self, result: u8) -> &mut Self {
        if result & (0x1 << 7) != 0x0 {
            self.set(ProcessorStatus::NEGATIVE_FLAG, true);
        }

        self
    }
}
