use core::panic;
use std::sync::{Arc, Mutex};

use super::{apu::Apu, cartridge::Cartridge, cpu::Addr, joypad::Joypad, ppu::Ppu};

pub struct Bus {
    mem: [u8; 0x800],  // 2 KB internal RAM
    rom: Cartridge,
    ppu: Ppu,
    apu: Apu,
    joypad1: Joypad,
}

impl Bus {
    pub fn new(rom: Cartridge, ppu: Ppu, audio_buffer: Arc<Mutex<Vec<f32>>>) -> Bus {
        let mut bus = Bus {
            mem: [0x0; 0x800],
            rom,
            ppu,
            apu: Apu::new(audio_buffer),
            joypad1: Joypad::new(),
        };

        bus.init();
        bus
    }

    fn init(&mut self) {
        println!("Mapper: {}", self.rom.mapper);
        self.ppu.load_chr(&self.rom.chr_rom);
    }

    pub fn ppu_tick(&mut self, tick: u8) {
        self.ppu.tick(tick);
    }

    /// Tick the APU once per CPU cycle.
    pub fn apu_tick(&mut self) {
        if let Some(dmc_addr) = self.apu.tick() {
            // DMC needs to read a byte from memory
            let value = self.read_u8(dmc_addr);
            self.apu.dmc_fill_buffer(value);
        }
    }

    pub fn get_ppu_tick(&self) -> (usize, usize) {
        (self.ppu.get_scanlines(), self.ppu.get_cycles())
    }

    pub fn take_frame(&mut self) -> Option<super::renderer::frame::Frame> {
        self.ppu.take_frame()
    }

    pub fn joypad1_mut(&mut self) -> &mut Joypad {
        &mut self.joypad1
    }

    fn handle_ppu_read(&mut self, idx: u8) -> u8 {
        match idx {
            0 => panic!("Read on PPUCTRL is invalid"),
            1 => panic!("Read on PPUMASK is invalid"),
            2 => self.ppu.status(),
            3 => panic!("Read on OAMADDR is invalid"),
            4 => self.ppu.oam_data_read(),
            5 => panic!("Read on PPUSCROLL is invalid"),
            6 => panic!("Read on PPUADDR is invalid"),
            7 => self.ppu.data_read(),
            _ => panic!("This should be impossible"),
        }
    }

    fn handle_ppu_write(&mut self, idx: u8, value: u8) {
        match idx {
            0 => self.ppu.ctrl(value),
            1 => self.ppu.mask(value),
            2 => panic!("Write on PPUSTATUS is invalid"),
            3 => self.ppu.oam_addr(value),
            4 => self.ppu.oam_data_write(value),
            5 => self.ppu.scroll(value),
            6 => self.ppu.addr(value),
            7 => self.ppu.data_write(value),
            _ => panic!("This should be impossible"),
        }
    }

    fn handle_oam_dma(&mut self, value: u8) {
        let addr = Addr::from_le_bytes([0x00, value]) as usize;

        let mut data: [u8; 256] = [0; 256];
        for i in 0..256 {
            data[i] = self.read_u8((addr + i) as u16);
        }
        self.ppu.oam_dma(&data);
    }

    pub fn read_u16(&mut self, address: Addr) -> u16 {
        u16::from_le_bytes([self.read_u8(address), self.read_u8(address + 1)])
    }

    pub fn read_u8(&mut self, address: Addr) -> u8 {
        match address {
            // Internal RAM (mirrored every 0x800 bytes)
            0x0000..=0x1FFF => self.mem[(address & 0x07FF) as usize],
            // PPU mapped I/O (mirrored every 8 bytes)
            0x2000..=0x3FFF => self.handle_ppu_read((address & 0x07) as u8),
            // OAM DMA
            0x4014 => panic!("Read on OAM DMA is invalid"),
            // APU Status
            0x4015 => self.apu.read_status(),
            // Joypad 1
            0x4016 => self.joypad1.read(),
            // Joypad 2 (not implemented yet)
            0x4017 => 0,
            // APU and I/O
            0x4000..=0x4013 => 0, // APU registers are write-only (except $4015)
            0x4018..=0x401F => 0, // Test mode
            // Cartridge space: PRG ROM
            0x8000..=0xFFFF => self.read_prg_rom(address),
            _ => 0,
        }
    }

    pub fn read_i8(&mut self, address: Addr) -> i8 {
        self.read_u8(address) as i8
    }

    fn read_prg_rom(&self, address: Addr) -> u8 {
        let offset = (address - 0x8000) as usize;
        let prg_len = self.rom.prg_rom.len();
        
        if prg_len == 0 {
            return 0;
        }

        self.rom.prg_rom[offset % prg_len]
    }

    pub fn write_u8(&mut self, address: Addr, value: u8) {
        match address {
            // Internal RAM (mirrored every 0x800 bytes)
            0x0000..=0x1FFF => self.mem[(address & 0x07FF) as usize] = value,
            // PPU mapped I/O
            0x2000..=0x3FFF => self.handle_ppu_write((address & 0x07) as u8, value),
            // OAM DMA
            0x4014 => self.handle_oam_dma(value),
            // Joypad 1 strobe
            0x4016 => self.joypad1.write(value),
            // APU registers ($4000-$4013, $4015, $4017)
            0x4000..=0x4013 | 0x4015 | 0x4017 => self.apu.write_register(address, value),
            // Remaining I/O
            0x4018..=0x401F => {} // Test mode
            // Cartridge space
            _ => {}
        }
    }
    
    pub fn poll_nmi_status(&mut self) -> bool {
        self.ppu.get_nmi_occurred()
    }
}
