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

    pub fn fetch_immediate(&self) -> u8 {
        1
    }

    pub fn fetch_zp(&self) -> u8 {
        1
    }

    pub fn fetch_zp_x(&self, idx: u8) -> u8 {
        1
    }

    pub fn fetch_zp_y(&self, idx: u8) -> u8 {
        1
    }

    pub fn fetch_absolute(&self) -> u8 {
        1
    }

    pub fn fetch_absolute_x(&self, idx: u8) -> u8 {
        1
    }

    pub fn fetch_absolute_y(&self, idx: u8) -> u8 {
        1
    }

    pub fn fetch_indirect_x(&self, idx: u8) -> u8 {
        1
    }

    pub fn fetch_indirect_y(&self, idx: u8) -> u8 {
        1
    }

    pub fn fetch_indirect(&self) -> u8 {
        1
    }

    pub fn read_u16(&self, address: Addr) -> u16 {
        1
    }

    pub fn read_u8(&self, address: Addr) -> u8 {
        1
    }

    pub fn write_u8(&self, address: Addr, value: u8) {}

    pub fn write_u16(&self, address: Addr, value: u16) {}
}
