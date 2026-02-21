mod emulator;
mod instructions;
mod registers;
mod trace;

use self::registers::ProcessorStatus;

use super::Bus;
use instructions::{AddressingMode, Instruction, InstructionVariant, INSTRUCTIONS};
use registers::Registers;

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
            cycles: 7,
        }
    }

    pub fn execute(&mut self) {
        let opcode = self.bus.read_u8(self.regs.pc);
        let instruction = Cpu::decode(opcode);

        // Trace::print_state(self, instruction);

        self.emulate(instruction);

        if self.bus.poll_nmi_status() {
            self.interrupt_nmi();
        }
    }

    pub fn reset(&mut self) {
        self.regs.sp -= 0x3;
        self.regs
            .status
            .set(ProcessorStatus::INTERRUPT_DISABLE, true);

        // Silence APU ($4015 = 0)
        // APU triangle phase is reset to 0 (i.e outputs a value of 15, the first setp of its waveform)
        // APU DPCM output ANDed with 1 (upper 6 bits cleared)
        // APU Frame Counter:
        //      2A03E, G, various clones: APU Frame Counter reset.
        //      2A03letterless: APU frame counter retains old value
    }

    pub fn power_up(&mut self) {
        // self.regs.status = ProcessorStatus::empty();
        // self.regs.status.set(ProcessorStatus::INTERRUPT_DISABLE, true);
        self.regs.acc = 0;
        self.regs.idx_x = 0;
        self.regs.idx_y = 0;
        self.regs.sp = 0xFD;
        // Frame IRQ Enabled
        self.bus.write_u8(0x4017, 0x00);
        // All Channel disabled
        self.bus.write_u8(0x4015, 0x00);
        // $4000 - $400F = $00
        // $4010 - $4013 = $00
        self.regs.pc = self.bus.read_u16(0xFFFC);
        // self.regs.pc=0xC000;
    }

    pub fn set_pc(&mut self, value: Addr) {
        self.regs.pc = value;
    }

    pub fn interrupt_nmi(&mut self) {
        self.stack_push_u16(self.regs.pc);

        let mut flags = self.regs.status;
        flags.set(ProcessorStatus::BREAK_CMD, false);
        flags.set(ProcessorStatus::BREAK_CMD2, true);
        self.stack_push_u8(flags.bits());

        self.regs
            .status
            .set(ProcessorStatus::INTERRUPT_DISABLE, true);

        self.tick(2);
        self.regs.pc = self.bus.read_u16(0xFFFA);
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

    pub fn is_page_cross(addr1: Addr, addr2: Addr) -> bool {
        addr1 & 0xFF00 != addr2 & 0xFF00
    }

    pub fn tick(&mut self, tick: u8) {
        self.cycles += tick as usize;
        self.bus.ppu_tick(tick * 3);
        // Tick APU once per CPU cycle
        for _ in 0..tick {
            self.bus.apu_tick();
        }
    }

    pub fn take_frame(&mut self) -> Option<super::renderer::frame::Frame> {
        self.bus.take_frame()
    }

    pub fn set_joypad_button(&mut self, button: super::joypad::JoypadButton, pressed: bool) {
        self.bus.joypad1_mut().set_button_pressed(button, pressed);
    }

    fn resolve_adressing(&mut self, mode: AddressingMode) -> (Addr, bool) {
        match mode {
            AddressingMode::Implied => (0xFFFF, false),
            AddressingMode::Relative => (self.regs.pc, false),
            AddressingMode::Immediate => (self.regs.pc, false),
            AddressingMode::ZeroPage => {
                let op = self.bus.read_u8(self.regs.pc);
                (Addr::from_le_bytes([op, 0x00]), false)
            }
            AddressingMode::ZeroPageX => {
                let mut op = self.bus.read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_x);
                (Addr::from_le_bytes([op, 0x00]), false)
            }
            AddressingMode::ZeroPageY => {
                let mut op = self.bus.read_u8(self.regs.pc);
                op = op.wrapping_add(self.regs.idx_y);
                (Addr::from_le_bytes([op, 0x00]), false)
            }
            AddressingMode::Absolute => (self.bus.read_u16(self.regs.pc), false),
            AddressingMode::AbsoluteX => {
                let addr = self.bus.read_u16(self.regs.pc);
                let final_addr = addr.wrapping_add(self.regs.idx_x as u16);

                (final_addr, Cpu::is_page_cross(addr, final_addr))
            }
            AddressingMode::AbsoluteY => {
                let addr = self.bus.read_u16(self.regs.pc);
                let final_addr = addr.wrapping_add(self.regs.idx_y as u16);

                (final_addr, Cpu::is_page_cross(addr, final_addr))
            }
            AddressingMode::IndirectX => {
                let op = self.bus.read_u8(self.regs.pc);
                let lb = self
                    .bus
                    .read_u8((op as Addr + self.regs.idx_x as Addr) & 0x00FF);
                let hb = self
                    .bus
                    .read_u8((op as Addr + self.regs.idx_x as Addr + 0x1) & 0x00FF);

                (Addr::from_le_bytes([lb, hb]), false)
            }
            AddressingMode::IndirectY => {
                let op = self.bus.read_u8(self.regs.pc);
                let lb = self.bus.read_u8(op as Addr);
                let hb = self.bus.read_u8((op as Addr + 0x1) & 0x00FF);

                let addr = Addr::from_le_bytes([lb, hb]);
                let final_addr = addr.wrapping_add(self.regs.idx_y as u16);

                (final_addr, Cpu::is_page_cross(addr, final_addr))
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

                (Addr::from_le_bytes([lb, hb]), false)
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

        self.tick(instruction.cycles);

        match instruction.variant {
            // These instructions modify the PC directly, no need to add the length.
            InstructionVariant::JMP | InstructionVariant::JSR | InstructionVariant::RTS => {}
            _ => {
                self.regs.pc += instruction.length as u16 - 1;
            }
        }
    }
}
