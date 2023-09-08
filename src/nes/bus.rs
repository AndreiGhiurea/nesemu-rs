const MEMORY_SIZE: usize = 0x10000;
use super::{cartridge::Cartridge, cpu::Addr, ppu::Ppu};

pub struct Bus {
    mem: [u8; MEMORY_SIZE],
    rom: Cartridge,
    ppu: Ppu,
}

impl Bus {
    pub fn new(rom: Cartridge, ppu: Ppu) -> Bus {
        let mut bus = Bus {
            mem: [0x0; MEMORY_SIZE],
            rom,
            ppu,
        };

        bus.load_rom();
        bus
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

    pub fn load_rom(&mut self) {
        let mut load_addr: usize = 0x8000;
        for byte in self.rom.prg_rom.clone() {
            self.write_u8(load_addr as u16, byte);
            self.write_u8(load_addr as u16 + 0x4000, byte);

            load_addr += 0x1;
        }
    }
}
