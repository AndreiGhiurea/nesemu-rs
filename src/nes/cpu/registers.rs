use bitflags::bitflags;

pub struct Registers {
    pub pc: u16,                 // Program counter
    pub sp: u8,                  // Stack pointer
    pub acc: u8,                 // Accumulator
    pub idx_x: u8,               // Index register X
    pub idx_y: u8,               // Index register Y
    pub status: ProcessorStatus, // Bitfield with various flags
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

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
    pub struct ProcessorStatus: u8 {
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
    pub fn set_carry_flag(&mut self, overflow: bool) -> &mut Self {
        self.set(ProcessorStatus::CARRY_FLAG, overflow);

        self
    }

    pub fn set_zero_flag(&mut self, result: u8) -> &mut Self {
        self.set(ProcessorStatus::ZERO_FLAG, result == 0x0);

        self
    }

    pub fn set_overflow_flag(&mut self, overflow: bool) -> &mut Self {
        self.set(ProcessorStatus::OVERFLOW_FLAG, overflow);

        self
    }

    pub fn set_negative_flag(&mut self, result: u8) -> &mut Self {
        self.set(ProcessorStatus::NEGATIVE_FLAG, result & (0x1 << 7) != 0x0);

        self
    }
}
