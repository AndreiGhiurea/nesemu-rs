mod addr_reg;
mod ctrl_reg;
mod mask_reg;
mod status_reg;

use addr_reg::AddressRegister;
use ctrl_reg::ControlRegister;
use mask_reg::MaskRegister;
use status_reg::StatusRegister;
use super::renderer::Renderer;

#[rustfmt::skip]
pub static SYSTEM_PALLETE: [(u8,u8,u8); 64] = [
   (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
   (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
   (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
   (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
   (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
   (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
   (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
   (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
   (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
   (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
   (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
   (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
   (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];

pub struct Ppu {
    mem: [u8; 0x4000],
    ctrl: ControlRegister,
    mask: MaskRegister,
    addr: AddressRegister,
    status: StatusRegister,
    oam_addr: u8,
    oam: [u8; 64 * 4],
    data_latch: u8,

    cycles: usize,
    scanlines: usize,

    nmi_occured: Option<u8>,
    renderer: Renderer
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            mem: [0; 0x4000],
            ctrl: ControlRegister::default(),
            mask: MaskRegister::default(),
            addr: AddressRegister::default(),
            status: StatusRegister::default(),
            oam_addr: 0,
            oam: [0; 64 * 4],
            data_latch: 0,
            cycles: 21,
            scanlines: 0,
            nmi_occured: None,
            renderer: Renderer::new()
        }
    }
    
    pub fn tick(&mut self, tick: u8) -> bool {
        self.cycles += tick as usize;
        if self.cycles >= 341 {
            self.scanlines += 1;
            self.cycles %= 341;

            if self.scanlines == 241 && self.ctrl.get_generate_nmi() {
                self.status.set_vblank(true);
                self.nmi_occured = Some(1);
            }

            if self.scanlines >= 262 {
                self.scanlines = 0;
                self.status.set_vblank(false);
                return true;
            }
        }

        false
    }

    pub fn get_scanlines(&self) -> usize {
        self.scanlines
    }

    pub fn get_cycles(&self) -> usize {
        self.cycles
    }

    pub fn load_chr(&mut self, chr: &Vec<u8>) {
        let load_addr: usize = 0x0;
        for byte in chr {
            self.mem[load_addr] = *byte;
        }
    }

    pub fn ctrl(&mut self, arg: u8) {
        let before_nmi_status = self.ctrl.get_generate_nmi();
        self.ctrl.update(arg);
        if !before_nmi_status && self.ctrl.get_generate_nmi() && self.status.get_vblank() {
            self.nmi_occured = Some(1);
        }
    }

    pub fn mask(&mut self, arg: u8) {
        self.mask.update(arg);
    }

    pub fn status(&self) -> u8 {
        self.status.get()
    }

    pub fn scroll(&mut self) {
        
    }

    pub fn addr(&mut self, value: u8) {
        self.addr.write_byte(value);
    }

    pub fn data_read(&mut self) -> u8 {
        let addr = self.addr.get();
        let needs_dummy = !matches!(addr, 0x3F00..=0x3FFF);

        if needs_dummy {
            // Simulate a dummy read when reading from ROM or RAM.
            let previous_data = self.data_latch;
            self.data_latch = self.mem[addr as usize];

            self.addr.increment(self.ctrl.get_vram_increment());

            previous_data
        } else {
            self.data_latch = self.mem[addr as usize];
            self.addr.increment(self.ctrl.get_vram_increment());

            self.data_latch
        }
    }

    pub fn data_write(&mut self, value: u8) {
        let addr = self.addr.get();
        self.mem[addr as usize] = value;

        self.addr.increment(self.ctrl.get_vram_increment());
    }

    pub fn oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    pub fn oam_data_read(&self) -> u8 {
        self.oam[self.oam_addr as usize]
    }

    pub fn oam_data_write(&mut self, value: u8) {
        self.oam[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn oam_dma(&mut self, data: &[u8; 256]) {
        self.oam.copy_from_slice(data);
    }

    pub fn get_nmi_occured(&mut self) -> bool {
        self.nmi_occured.take().is_some()
    }
}
