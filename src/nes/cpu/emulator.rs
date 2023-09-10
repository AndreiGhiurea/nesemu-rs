use super::{
    instructions::{AddressingMode, Instruction},
    registers::ProcessorStatus,
    Cpu,
};

pub struct Emu;

impl Emu {
    pub fn adc(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, page_cross) = cpu.resolve_adressing(instr.mode);
        let op = cpu.bus.read_u8(addr);

        let carry = cpu.regs.status.contains(ProcessorStatus::CARRY_FLAG);
        let carry: u8 = if carry { 0x1 } else { 0x0 };

        let result: u16 = cpu.regs.acc as u16 + op as u16 + carry as u16;
        let carry = result > 0xFF;

        let ops_have_same_sign = (cpu.regs.acc ^ op) & 0x80 == 0x0;
        let result_has_same_sign = (cpu.regs.acc ^ result as u8) & 0x80 == 0x0;

        // Overflow flag is set if operands have the same sign and the result has a different sign.
        let overflow = ops_have_same_sign & !result_has_same_sign;

        // Overflow flag indicates signed overflow.
        // Carry indicates unsigned overflow.
        cpu.regs
            .status
            .set_carry_flag(carry)
            .set_zero_flag(result as u8)
            .set_overflow_flag(overflow)
            .set_negative_flag(result as u8);

        if page_cross {
            cpu.tick(1);
        }

        cpu.regs.acc = result as u8;
    }

    pub fn and(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, page_cross) = cpu.resolve_adressing(instr.mode);
        let op = cpu.bus.read_u8(addr);

        cpu.regs.acc &= op;

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.acc)
            .set_negative_flag(cpu.regs.acc);

        if page_cross {
            cpu.tick(1);
        }
    }

    pub fn asl(cpu: &mut Cpu, instr: &Instruction) {
        let mut op: u8;
        if let AddressingMode::Accumulator = instr.mode {
            op = cpu.regs.acc;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            op = cpu.bus.read_u8(addr);
        }

        // Put bit 7 into carry flag.
        let is_bit_set = op & (0x1 << 7) != 0;
        cpu.regs.status.set(ProcessorStatus::CARRY_FLAG, is_bit_set);

        op <<= 0x1;

        cpu.regs.status.set_negative_flag(op).set_zero_flag(op);

        if let AddressingMode::Accumulator = instr.mode {
            cpu.regs.acc = op;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            cpu.bus.write_u8(addr, op);
        }
    }

    fn branch(cpu: &mut Cpu, condition: bool) {
        if condition {
            cpu.tick(1); // +1 on success

            let op = cpu.bus.read_i8(cpu.regs.pc);
            let new_pc = cpu.regs.pc.wrapping_add_signed(op as i16);

            // Add +1 on PC when verifying page cross account for operand skipping.
            if Cpu::is_page_cross(new_pc + 1, cpu.regs.pc.wrapping_add(1)) {
                cpu.tick(1);
            }

            cpu.regs.pc = new_pc;
        }
    }

    pub fn bcc(cpu: &mut Cpu, _instr: &Instruction) {
        Emu::branch(cpu, !cpu.regs.status.contains(ProcessorStatus::CARRY_FLAG));
    }

    pub fn bcs(cpu: &mut Cpu, _instr: &Instruction) {
        Emu::branch(cpu, cpu.regs.status.contains(ProcessorStatus::CARRY_FLAG));
    }

    pub fn beq(cpu: &mut Cpu, _instr: &Instruction) {
        Emu::branch(cpu, cpu.regs.status.contains(ProcessorStatus::ZERO_FLAG));
    }

    pub fn brk(cpu: &mut Cpu, instr: &Instruction) {
        let mut flags = cpu.regs.status;

        // This instructions pushes bit B flags as 1.
        flags.set(ProcessorStatus::BREAK_CMD, true);
        flags.set(ProcessorStatus::BREAK_CMD2, true);

        panic!("Instruction not implemented {:?}", instr.variant);
    }

    pub fn bne(cpu: &mut Cpu, _instr: &Instruction) {
        Emu::branch(cpu, !cpu.regs.status.contains(ProcessorStatus::ZERO_FLAG));
    }

    pub fn bmi(cpu: &mut Cpu, _instr: &Instruction) {
        Emu::branch(
            cpu,
            cpu.regs.status.contains(ProcessorStatus::NEGATIVE_FLAG),
        );
    }

    pub fn bpl(cpu: &mut Cpu, _instr: &Instruction) {
        Emu::branch(
            cpu,
            !cpu.regs.status.contains(ProcessorStatus::NEGATIVE_FLAG),
        );
    }

    pub fn bvs(cpu: &mut Cpu, _instr: &Instruction) {
        Emu::branch(
            cpu,
            cpu.regs.status.contains(ProcessorStatus::OVERFLOW_FLAG),
        );
    }

    pub fn bvc(cpu: &mut Cpu, _instr: &Instruction) {
        Emu::branch(
            cpu,
            !cpu.regs.status.contains(ProcessorStatus::OVERFLOW_FLAG),
        );
    }

    pub fn bit(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);
        let mut op = cpu.bus.read_u8(addr);

        let is_neg_set = op & (0x1 << 7) != 0x0;
        let is_of_set = op & (0x1 << 6) != 0x0;

        op &= cpu.regs.acc;

        cpu.regs.status.set_zero_flag(op);

        //
        // BIT instruction treats these 2 flags differently.
        // Bit 6 and 7 of the memory location BEOFRE and is stored in these 2 flags.
        //
        cpu.regs
            .status
            .set(ProcessorStatus::NEGATIVE_FLAG, is_neg_set);
        cpu.regs
            .status
            .set(ProcessorStatus::OVERFLOW_FLAG, is_of_set);
    }

    pub fn clc(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.status.set(ProcessorStatus::CARRY_FLAG, false);
    }

    pub fn cld(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.status.set(ProcessorStatus::DECIMAL_MODE, false);
    }

    pub fn cli(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs
            .status
            .set(ProcessorStatus::INTERRUPT_DISABLE, false);
    }

    pub fn clv(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.status.set(ProcessorStatus::OVERFLOW_FLAG, false);
    }

    pub fn cmp(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, page_cross) = cpu.resolve_adressing(instr.mode);
        let op = cpu.bus.read_u8(addr);

        let res = cpu.regs.acc.wrapping_sub(op);

        cpu.regs
            .status
            .set_carry_flag(cpu.regs.acc >= op)
            .set_zero_flag(res)
            .set_negative_flag(res);

        if page_cross {
            cpu.tick(1);
        }
    }

    pub fn cpx(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);
        let op = cpu.bus.read_u8(addr);

        let res = cpu.regs.idx_x.wrapping_sub(op);

        cpu.regs
            .status
            .set_carry_flag(cpu.regs.idx_x >= op)
            .set_zero_flag(res)
            .set_negative_flag(res);
    }

    pub fn cpy(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);
        let op = cpu.bus.read_u8(addr);

        let res = cpu.regs.idx_y.wrapping_sub(op);

        cpu.regs
            .status
            .set_carry_flag(cpu.regs.idx_y >= op)
            .set_zero_flag(res)
            .set_negative_flag(res);
    }

    pub fn dec(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);
        let mut op = cpu.bus.read_u8(addr);

        op = op.wrapping_sub(1);

        cpu.regs.status.set_zero_flag(op).set_negative_flag(op);

        cpu.bus.write_u8(addr, op);
    }

    pub fn dex(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.idx_x = cpu.regs.idx_x.wrapping_sub(1);

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.idx_x)
            .set_negative_flag(cpu.regs.idx_x);
    }

    pub fn dey(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.idx_y = cpu.regs.idx_y.wrapping_sub(1);

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.idx_y)
            .set_negative_flag(cpu.regs.idx_y);
    }

    pub fn eor(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, page_cross) = cpu.resolve_adressing(instr.mode);
        let op = cpu.bus.read_u8(addr);

        cpu.regs.acc ^= op;

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.acc)
            .set_negative_flag(cpu.regs.acc);

        if page_cross {
            cpu.tick(1);
        }
    }

    pub fn inc(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);
        let mut op = cpu.bus.read_u8(addr);

        op = op.wrapping_add(1);

        cpu.regs.status.set_zero_flag(op).set_negative_flag(op);

        cpu.bus.write_u8(addr, op);
    }

    pub fn inx(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.idx_x = cpu.regs.idx_x.wrapping_add(1);

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.idx_x)
            .set_negative_flag(cpu.regs.idx_x);
    }

    pub fn iny(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.idx_y = cpu.regs.idx_y.wrapping_add(1);

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.idx_y)
            .set_negative_flag(cpu.regs.idx_y);
    }

    pub fn jmp(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);

        cpu.regs.pc = addr;
    }

    pub fn jsr(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);

        cpu.regs.pc += instr.length as u16 - 2;
        cpu.stack_push_u16(cpu.regs.pc);

        cpu.regs.pc = addr;
    }

    pub fn lda(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, page_cross) = cpu.resolve_adressing(instr.mode);
        let op = cpu.bus.read_u8(addr);

        cpu.regs.acc = op;
        cpu.regs.status.set_zero_flag(op).set_negative_flag(op);

        if page_cross {
            cpu.tick(1);
        }
    }

    pub fn ldx(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, page_cross) = cpu.resolve_adressing(instr.mode);
        let op = cpu.bus.read_u8(addr);

        cpu.regs.idx_x = op;
        cpu.regs.status.set_zero_flag(op).set_negative_flag(op);

        if page_cross {
            cpu.tick(1);
        }
    }

    pub fn ldy(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, page_cross) = cpu.resolve_adressing(instr.mode);
        let op = cpu.bus.read_u8(addr);

        cpu.regs.idx_y = op;
        cpu.regs.status.set_zero_flag(op).set_negative_flag(op);

        if page_cross {
            cpu.tick(1);
        }
    }

    pub fn lsr(cpu: &mut Cpu, instr: &Instruction) {
        let mut op: u8;
        if let AddressingMode::Accumulator = instr.mode {
            op = cpu.regs.acc;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            op = cpu.bus.read_u8(addr);
        }

        // Put bit 0 into carry flag.
        let is_bit_set = op & 0x1 != 0x0;
        cpu.regs.status.set(ProcessorStatus::CARRY_FLAG, is_bit_set);

        op >>= 0x1;

        cpu.regs.status.set_negative_flag(op).set_zero_flag(op);

        if let AddressingMode::Accumulator = instr.mode {
            cpu.regs.acc = op;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            cpu.bus.write_u8(addr, op);
        }
    }

    pub fn nop(cpu: &mut Cpu, instr: &Instruction) {
        let (_, page_cross) = cpu.resolve_adressing(instr.mode);
        if page_cross {
            cpu.tick(1);
        }
        // *cracks open a cold one*
    }

    pub fn ora(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, page_cross) = cpu.resolve_adressing(instr.mode);
        let op = cpu.bus.read_u8(addr);

        cpu.regs.acc |= op;

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.acc)
            .set_negative_flag(cpu.regs.acc);

        if page_cross {
            cpu.tick(1);
        }
    }

    pub fn pha(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.stack_push_u8(cpu.regs.acc);
    }

    pub fn php(cpu: &mut Cpu, _instr: &Instruction) {
        let mut flags = cpu.regs.status;

        // This instructions pushes bit B flags as 1.
        flags.set(ProcessorStatus::BREAK_CMD, true);
        flags.set(ProcessorStatus::BREAK_CMD2, true);

        cpu.stack_push_u8(flags.bits());
    }

    pub fn pla(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.acc = cpu.stack_pop_u8();

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.acc)
            .set_negative_flag(cpu.regs.acc);
    }

    pub fn plp(cpu: &mut Cpu, _instr: &Instruction) {
        let new_status = cpu.stack_pop_u8();
        cpu.regs.status = ProcessorStatus::from_bits(new_status).unwrap();

        // This instructions discards the B flags, as they're not techincally present in the CPU.
        cpu.regs.status.set(ProcessorStatus::BREAK_CMD, false);
        cpu.regs.status.set(ProcessorStatus::BREAK_CMD2, true);
    }

    pub fn rol(cpu: &mut Cpu, instr: &Instruction) {
        let mut op: u8;
        if let AddressingMode::Accumulator = instr.mode {
            op = cpu.regs.acc;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            op = cpu.bus.read_u8(addr);
        }

        // Save current carry flag
        let is_current_carry_set = cpu.regs.status.contains(ProcessorStatus::CARRY_FLAG);

        let is_bit_set = op & (0x1 << 7) != 0x0;
        cpu.regs.status.set(ProcessorStatus::CARRY_FLAG, is_bit_set);

        op <<= 0x1;

        if is_current_carry_set {
            op |= 0x1;
        }

        cpu.regs.status.set_negative_flag(op).set_zero_flag(op);

        if let AddressingMode::Accumulator = instr.mode {
            cpu.regs.acc = op;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            cpu.bus.write_u8(addr, op);
        }
    }

    pub fn ror(cpu: &mut Cpu, instr: &Instruction) {
        let mut op: u8;
        if let AddressingMode::Accumulator = instr.mode {
            op = cpu.regs.acc;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            op = cpu.bus.read_u8(addr);
        }

        // Save current carry flag
        let is_current_carry_set = cpu.regs.status.contains(ProcessorStatus::CARRY_FLAG);

        let is_bit_set = op & 0x1 != 0x0;
        cpu.regs.status.set(ProcessorStatus::CARRY_FLAG, is_bit_set);

        op >>= 0x1;

        if is_current_carry_set {
            op |= 0x1 << 7;
        }

        cpu.regs.status.set_negative_flag(op).set_zero_flag(op);

        if let AddressingMode::Accumulator = instr.mode {
            cpu.regs.acc = op;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            cpu.bus.write_u8(addr, op);
        }
    }

    pub fn rti(cpu: &mut Cpu, _instr: &Instruction) {
        // Returrn from interrupt.
        let flags = cpu.stack_pop_u8();
        cpu.regs.status = ProcessorStatus::from_bits(flags).unwrap();

        // This instructions discards the B flags, as they're not techincally present in the CPU.
        cpu.regs.status.set(ProcessorStatus::BREAK_CMD, false);
        cpu.regs.status.set(ProcessorStatus::BREAK_CMD2, true);

        cpu.regs.pc = cpu.stack_pop_u16();
    }

    pub fn rts(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.pc = cpu.stack_pop_u16() + 1;
    }

    pub fn sbc(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, page_cross) = cpu.resolve_adressing(instr.mode);

        // Same implementation as ADC but with negated operator.
        let op = !cpu.bus.read_u8(addr);

        let carry = cpu.regs.status.contains(ProcessorStatus::CARRY_FLAG);
        let carry: u8 = if carry { 0x1 } else { 0x0 };

        let result: u16 = cpu.regs.acc as u16 + op as u16 + carry as u16;
        let carry = result > 0xFF;

        let ops_have_same_sign = (cpu.regs.acc ^ op) & 0x80 == 0x0;
        let result_has_same_sign = (cpu.regs.acc ^ result as u8) & 0x80 == 0x0;

        // Overflow flag is set if operands have the same sign and the result has a different sign.
        let overflow = ops_have_same_sign & !result_has_same_sign;

        // Overflow flag indicates signed overflow.
        // Carry indicates unsigned overflow.
        cpu.regs
            .status
            .set_carry_flag(carry)
            .set_zero_flag(result as u8)
            .set_overflow_flag(overflow)
            .set_negative_flag(result as u8);

        cpu.regs.acc = result as u8;

        if page_cross {
            cpu.tick(1);
        }
    }

    pub fn sec(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.status.set(ProcessorStatus::CARRY_FLAG, true);
    }

    pub fn sed(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.status.set(ProcessorStatus::DECIMAL_MODE, true);
    }

    pub fn sei(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs
            .status
            .set(ProcessorStatus::INTERRUPT_DISABLE, true);
    }

    pub fn sta(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);
        cpu.bus.write_u8(addr, cpu.regs.acc);
    }

    pub fn stx(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);
        cpu.bus.write_u8(addr, cpu.regs.idx_x);
    }

    pub fn sty(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);
        cpu.bus.write_u8(addr, cpu.regs.idx_y);
    }

    pub fn tax(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.idx_x = cpu.regs.acc;

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.idx_x)
            .set_negative_flag(cpu.regs.idx_x);
    }

    pub fn tay(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.idx_y = cpu.regs.acc;

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.idx_y)
            .set_negative_flag(cpu.regs.idx_y);
    }

    pub fn tsx(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.idx_x = cpu.regs.sp;

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.idx_x)
            .set_negative_flag(cpu.regs.idx_x);
    }

    pub fn txa(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.acc = cpu.regs.idx_x;

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.acc)
            .set_negative_flag(cpu.regs.acc);
    }

    pub fn txs(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.sp = cpu.regs.idx_x;
    }

    pub fn tya(cpu: &mut Cpu, _instr: &Instruction) {
        cpu.regs.acc = cpu.regs.idx_y;

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.acc)
            .set_negative_flag(cpu.regs.acc);
    }

    pub fn lax(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, page_cross) = cpu.resolve_adressing(instr.mode);
        let op = cpu.bus.read_u8(addr);

        cpu.regs.idx_x = op;
        cpu.regs.acc = op;
        cpu.regs.status.set_zero_flag(op).set_negative_flag(op);

        if page_cross {
            cpu.tick(1);
        }
    }

    pub fn sax(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);
        let op = cpu.regs.acc & cpu.regs.idx_x;

        cpu.bus.write_u8(addr, op);
    }

    pub fn dcp(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);
        let mut op = cpu.bus.read_u8(addr);

        op = op.wrapping_sub(1);
        let res = cpu.regs.acc.wrapping_sub(op);

        cpu.bus.write_u8(addr, op);

        cpu.regs
            .status
            .set_carry_flag(cpu.regs.acc >= op)
            .set_zero_flag(res)
            .set_negative_flag(res);
    }

    pub fn isb(cpu: &mut Cpu, instr: &Instruction) {
        let (addr, _) = cpu.resolve_adressing(instr.mode);
        let mut op = cpu.bus.read_u8(addr);

        op = op.wrapping_add(1);
        cpu.bus.write_u8(addr, op);

        // Same implementation as ADC but with negated operator.
        let op = !op;

        let carry = cpu.regs.status.contains(ProcessorStatus::CARRY_FLAG);
        let carry: u8 = if carry { 0x1 } else { 0x0 };

        let result: u16 = cpu.regs.acc as u16 + op as u16 + carry as u16;
        let carry = result > 0xFF;

        let ops_have_same_sign = (cpu.regs.acc ^ op) & 0x80 == 0x0;
        let result_has_same_sign = (cpu.regs.acc ^ result as u8) & 0x80 == 0x0;

        // Overflow flag is set if operands have the same sign and the result has a different sign.
        let overflow = ops_have_same_sign & !result_has_same_sign;

        // Overflow flag indicates signed overflow.
        // Carry indicates unsigned overflow.
        cpu.regs
            .status
            .set_carry_flag(carry)
            .set_zero_flag(result as u8)
            .set_overflow_flag(overflow)
            .set_negative_flag(result as u8);

        cpu.regs.acc = result as u8;
    }

    pub fn slo(cpu: &mut Cpu, instr: &Instruction) {
        let mut op: u8;
        if let AddressingMode::Accumulator = instr.mode {
            op = cpu.regs.acc;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            op = cpu.bus.read_u8(addr);
        }

        // Put bit 7 into carry flag.
        let is_bit_set = op & (0x1 << 7) != 0;
        cpu.regs.status.set(ProcessorStatus::CARRY_FLAG, is_bit_set);

        op <<= 0x1;

        cpu.regs.status.set_negative_flag(op).set_zero_flag(op);

        if let AddressingMode::Accumulator = instr.mode {
            cpu.regs.acc = op;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            cpu.bus.write_u8(addr, op);
        }

        cpu.regs.acc |= op;

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.acc)
            .set_negative_flag(cpu.regs.acc);
    }

    pub fn rla(cpu: &mut Cpu, instr: &Instruction) {
        let mut op: u8;
        if let AddressingMode::Accumulator = instr.mode {
            op = cpu.regs.acc;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            op = cpu.bus.read_u8(addr);
        }

        // Save current carry flag
        let is_current_carry_set = cpu.regs.status.contains(ProcessorStatus::CARRY_FLAG);

        let is_bit_set = op & (0x1 << 7) != 0x0;
        cpu.regs.status.set(ProcessorStatus::CARRY_FLAG, is_bit_set);

        op <<= 0x1;

        if is_current_carry_set {
            op |= 0x1;
        }

        cpu.regs.status.set_negative_flag(op).set_zero_flag(op);

        if let AddressingMode::Accumulator = instr.mode {
            cpu.regs.acc = op;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            cpu.bus.write_u8(addr, op);
        }

        cpu.regs.acc &= op;

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.acc)
            .set_negative_flag(cpu.regs.acc);
    }

    pub fn sre(cpu: &mut Cpu, instr: &Instruction) {
        let mut op: u8;
        if let AddressingMode::Accumulator = instr.mode {
            op = cpu.regs.acc;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            op = cpu.bus.read_u8(addr);
        }

        // Put bit 0 into carry flag.
        let is_bit_set = op & 0x1 != 0x0;
        cpu.regs.status.set(ProcessorStatus::CARRY_FLAG, is_bit_set);

        op >>= 0x1;

        cpu.regs.status.set_negative_flag(op).set_zero_flag(op);

        if let AddressingMode::Accumulator = instr.mode {
            cpu.regs.acc = op;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            cpu.bus.write_u8(addr, op);
        }

        cpu.regs.acc ^= op;

        cpu.regs
            .status
            .set_zero_flag(cpu.regs.acc)
            .set_negative_flag(cpu.regs.acc);
    }

    pub fn rra(cpu: &mut Cpu, instr: &Instruction) {
        let mut op: u8;
        if let AddressingMode::Accumulator = instr.mode {
            op = cpu.regs.acc;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            op = cpu.bus.read_u8(addr);
        }

        // Save current carry flag
        let is_current_carry_set = cpu.regs.status.contains(ProcessorStatus::CARRY_FLAG);

        let is_bit_set = op & 0x1 != 0x0;
        cpu.regs.status.set(ProcessorStatus::CARRY_FLAG, is_bit_set);

        op >>= 0x1;

        if is_current_carry_set {
            op |= 0x1 << 7;
        }

        cpu.regs.status.set_negative_flag(op).set_zero_flag(op);

        if let AddressingMode::Accumulator = instr.mode {
            cpu.regs.acc = op;
        } else {
            let (addr, _) = cpu.resolve_adressing(instr.mode);
            cpu.bus.write_u8(addr, op);
        }

        let carry = cpu.regs.status.contains(ProcessorStatus::CARRY_FLAG);
        let carry: u8 = if carry { 0x1 } else { 0x0 };

        let result: u16 = cpu.regs.acc as u16 + op as u16 + carry as u16;
        let carry = result > 0xFF;

        let ops_have_same_sign = (cpu.regs.acc ^ op) & 0x80 == 0x0;
        let result_has_same_sign = (cpu.regs.acc ^ result as u8) & 0x80 == 0x0;

        // Overflow flag is set if operands have the same sign and the result has a different sign.
        let overflow = ops_have_same_sign & !result_has_same_sign;

        // Overflow flag indicates signed overflow.
        // Carry indicates unsigned overflow.
        cpu.regs
            .status
            .set_carry_flag(carry)
            .set_zero_flag(result as u8)
            .set_overflow_flag(overflow)
            .set_negative_flag(result as u8);

        cpu.regs.acc = result as u8;
    }
}
