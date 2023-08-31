const MEMORY_SIZE: usize = 0x10000;
use super::cpu::Addr;

pub struct Bus {
    mem: [u8; MEMORY_SIZE],
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            mem: [0x0; MEMORY_SIZE],
        }
    }

    pub fn read_u16(&self, address: Addr) -> u16 {
        u16::from_le_bytes([self.mem[address as usize], self.mem[address as usize + 1]])
    }

    pub fn read_u8(&self, address: Addr) -> u8 {
        self.mem[address as usize]
    }

    pub fn read_i8(&self, address: Addr) -> i8 {
        self.mem[address as usize] as i8
    }

    pub fn write_u8(&mut self, address: Addr, value: u8) {
        self.mem[address as usize] = value;
    }

    pub fn write_u16(&mut self, address: Addr, value: u16) {
        let bytes = value.to_le_bytes();
        self.mem[address as usize] = bytes[0];
        self.mem[address as usize + 1] = bytes[1];
    }
}
