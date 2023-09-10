const MEMORY_SIZE: usize = 0x10000;
use core::panic;

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

        bus.init();
        bus
    }

    fn init(&mut self) {
        let mut load_addr: usize = 0x8000;
        for byte in self.rom.prg_rom.clone() {
            self.write_u8(load_addr as u16, byte);
            self.write_u8(load_addr as u16 + 0x4000, byte);
            load_addr += 0x1;
        }

        self.ppu.load_chr(&self.rom.chr_rom);

        // println!("{}", self.rom.mapper);
    }

    pub fn ppu_tick(&mut self, tick: u8) {
        self.ppu.tick(tick);
    }

    pub fn get_ppu_tick(&self) -> (usize, usize) {
        (self.ppu.get_scanlines(), self.ppu.get_cycles())
    }

    fn handle_ppu_read(&mut self, idx: u8) -> u8 {
        match idx {
            0 => panic!("Read on PPUCTRL is invalid"),
            1 => panic!("Read on PPUMASK is invalid"),
            2 => self.ppu.status(),
            4 => panic!("Read on OAMADDR is invalid"),
            5 => self.ppu.oam_data_read(),
            6 => panic!("Read on PPUSCROLL is invalid"),
            7 => panic!("Read on PPUADDR is invalid"),
            8 => self.ppu.data_read(),
            _ => panic!("This should be impossible"),
        }
    }

    fn handle_ppu_write(&mut self, idx: u8, value: u8) {
        match idx {
            0 => self.ppu.ctrl(value),
            1 => self.ppu.mask(value),
            2 => panic!("Write on PPUSTATUS is invalid"),
            4 => self.ppu.oam_addr(value),
            5 => self.ppu.oam_data_write(value),
            6 => self.ppu.scroll(),
            7 => self.ppu.addr(value),
            8 => self.ppu.data_write(value),
            _ => panic!("This should be impossible"),
        }
    }

    fn handle_oam_dma(&mut self, value: u8) {
        let addr = Addr::from_le_bytes([0x00, value]) as usize;

        let mut data: [u8; 256] = [0; 256];
        data.copy_from_slice(&self.mem[addr..=(addr + 0xFF)]);
        self.ppu.oam_dma(&data);
    }

    pub fn read_u16(&mut self, address: Addr) -> u16 {
        u16::from_le_bytes([self.read_u8(address), self.read_u8(address + 1)])
    }

    pub fn read_u8(&mut self, address: Addr) -> u8 {
        let address = match address {
            // Handle internal RAM mirroring
            0x0000..=0x1FFF => address % 0x800,
            // Handle PPU mappped I/O
            0x2000..=0x3FFF => return self.handle_ppu_read(address as u8 % 8),
            // Handle OAM DMA
            0x4014 => panic!("Read on OMA DMA is invalid"),
            // Handle APU
            0x4000..=0x4017 => todo!("Implement APU"),
            0x4018..=0x401F => todo!("APU and I/O functionallity"),
            // Cartridge space, leave this as is.
            _ => address,
        };

        self.mem[address as usize]
    }

    pub fn read_i8(&mut self, address: Addr) -> i8 {
        self.read_u8(address) as i8
    }

    pub fn write_u8(&mut self, address: Addr, value: u8) {
        let address = match address {
            // Handle internal RAM mirroring
            0x0000..=0x1FFF => address % 0x800,
            // Handle PPU mappped I/O
            0x2000..=0x3FFF => return self.handle_ppu_write(address as u8 % 8, value),
            // Handle OAM DMA
            0x4014 => return self.handle_oam_dma(value),
            // Handle APU
            0x4000..=0x4017 => todo!("Implement APU"),
            0x4018..=0x401F => todo!("APU and I/O functionallity"),
            // Cartridge space, leave this as is.
            _ => address,
        };

        self.mem[address as usize] = value;
    }
}
