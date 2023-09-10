use std::fmt;

use super::{
    instructions::{AddressingMode, Instruction, InstructionVariant},
    Addr, Cpu,
};

pub struct Trace;

impl Trace {
    fn print_adressing(cpu: &mut Cpu, instr: &Instruction) {
        let print_operand = !matches!(instr.variant, InstructionVariant::JSR)
            && !matches!(instr.variant, InstructionVariant::JMP);

        cpu.regs.pc += 1;

        match instr.mode {
            AddressingMode::Implied => {
                print!("{: <28}", " ");
            }
            AddressingMode::Accumulator => {
                print!("{: <28}", "A");
            }
            AddressingMode::Relative => {
                let op = cpu.bus.read_i8(cpu.regs.pc);
                let mut addr = cpu.regs.pc + 1;
                addr = addr.wrapping_add_signed(op as i16);
                print!("${: <27}", format!("{:02X}", addr));
            }
            AddressingMode::Immediate => {
                let op = cpu.bus.read_u8(cpu.regs.pc);
                print!("#${: <26}", format!("{:02X}", op));
            }
            AddressingMode::ZeroPage => {
                let immediate: u8 = cpu.bus.read_u8(cpu.regs.pc);
                if print_operand {
                    let (addr, _) = cpu.resolve_adressing(instr.mode);
                    let op = cpu.bus.read_u8(addr);
                    print!("${: <27}", format!("{:02X} = {:02X}", immediate, op));
                } else {
                    print!("${: <27}", format!("{:04X}", immediate));
                }
            }
            AddressingMode::ZeroPageX => {
                let immediate = cpu.bus.read_u8(cpu.regs.pc);
                if print_operand {
                    let (addr, _) = cpu.resolve_adressing(instr.mode);
                    let op = cpu.bus.read_u8(addr);
                    print!(
                        "${: <27}",
                        format!("{:02X},X @ {:02X} = {:02X}", immediate, addr as u8, op)
                    );
                } else {
                    print!("${: <27}", format!("{:02X},X", immediate));
                }
            }
            AddressingMode::ZeroPageY => {
                let immediate = cpu.bus.read_u8(cpu.regs.pc);
                if print_operand {
                    let (addr, _) = cpu.resolve_adressing(instr.mode);
                    let op = cpu.bus.read_u8(addr);
                    print!(
                        "${: <27}",
                        format!("{:02X},Y @ {:02X} = {:02X}", immediate, addr as u8, op)
                    );
                } else {
                    print!("${: <27}", format!("{:02X},Y", immediate));
                }
            }
            AddressingMode::Absolute => {
                let addr = cpu.bus.read_u16(cpu.regs.pc);
                if print_operand {
                    let op = cpu.bus.read_u8(addr);
                    print!("${: <27}", format!("{:04X} = {:02X}", addr, op));
                } else {
                    print!("${: <27}", format!("{:04X}", addr));
                }
            }
            AddressingMode::AbsoluteX => {
                let addr = cpu.bus.read_u16(cpu.regs.pc);
                let (final_addr, _) = cpu.resolve_adressing(instr.mode);
                let op = cpu.bus.read_u8(final_addr);
                print!(
                    "${: <27}",
                    format!("{:04X},X @ {:04X} = {:02X}", addr, final_addr, op)
                );
            }
            AddressingMode::AbsoluteY => {
                let addr = cpu.bus.read_u16(cpu.regs.pc);
                let (final_addr, _) = cpu.resolve_adressing(instr.mode);
                let op = cpu.bus.read_u8(final_addr);
                print!(
                    "${: <27}",
                    format!("{:04X},Y @ {:04X} = {:02X}", addr, final_addr, op)
                );
            }
            AddressingMode::IndirectX => {
                let immediate = cpu.bus.read_u8(cpu.regs.pc);
                let (addr, _) = cpu.resolve_adressing(instr.mode);
                let op = cpu.bus.read_u8(addr);
                print!(
                    "(${: <26}",
                    format!(
                        "{:02X},X) @ {:02X} = {:04X} = {:02X}",
                        immediate,
                        immediate.wrapping_add(cpu.regs.idx_x),
                        addr,
                        op
                    )
                );
            }
            AddressingMode::IndirectY => {
                let immediate = cpu.bus.read_u8(cpu.regs.pc);
                let (addr, _) = cpu.resolve_adressing(instr.mode);
                let op = cpu.bus.read_u8(addr);
                print!(
                    "(${: <26}",
                    format!(
                        "{:02X}),Y = {:04X} @ {:04X} = {:02X}",
                        immediate,
                        addr.wrapping_sub(cpu.regs.idx_y as u16),
                        addr,
                        op
                    )
                );
            }
            AddressingMode::Indirect => {
                let indirect_addr = cpu.bus.read_u16(cpu.regs.pc);
                let lb: u8;
                let hb: u8;

                if indirect_addr & 0x00FF == 0x00FF {
                    lb = cpu.bus.read_u8(indirect_addr);
                    hb = cpu.bus.read_u8(indirect_addr & 0xFF00);
                } else {
                    (lb, hb) = cpu.bus.read_u16(indirect_addr).to_le_bytes().into();
                }

                let addr = Addr::from_le_bytes([lb, hb]);
                print!(
                    "(${: <26}",
                    format!("{:04X}) = {:04X}", indirect_addr, addr)
                );
            }
        }

        cpu.regs.pc -= 1;
    }

    pub fn print_state(cpu: &mut Cpu, instr: &Instruction) {
        print!("{:04X}  ", cpu.regs.pc);

        let mut max_length = 3;
        for idx in 0..instr.length {
            print!("{:02X} ", cpu.bus.read_u8(cpu.regs.pc + idx as u16));
            max_length -= 1;
        }

        for _idx in 0..max_length {
            print!("   ");
        }

        let opcode = cpu.bus.read_u8(cpu.regs.pc);
        let is_unofficial = match instr.variant {
            InstructionVariant::NOP => opcode != 0xEA,
            InstructionVariant::LAX => true,
            InstructionVariant::SAX => true,
            InstructionVariant::SBC => opcode == 0xEB,
            InstructionVariant::DCP => true,
            InstructionVariant::ISB => true,
            InstructionVariant::SLO => true,
            InstructionVariant::RLA => true,
            InstructionVariant::SRE => true,
            InstructionVariant::RRA => true,
            _ => false,
        };

        if is_unofficial {
            print!("*{:?} ", instr.variant);
        } else {
            print!(" {:?} ", instr.variant);
        }

        Trace::print_adressing(cpu, instr);

        println!("{}", cpu);
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{: >3},{: >3} CYC:{}",
            self.regs.acc,
            self.regs.idx_x,
            self.regs.idx_y,
            self.regs.status.bits(),
            self.regs.sp,
            self.bus.get_ppu_tick().0,
            self.bus.get_ppu_tick().1,
            self.cycles
        )
    }
}
