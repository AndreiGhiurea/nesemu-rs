mod addr_reg;
mod ctrl_reg;
mod mask_reg;
mod status_reg;

use addr_reg::AddressRegister;
use ctrl_reg::ControlRegister;
use mask_reg::MaskRegister;
use status_reg::StatusRegister;

use super::cartridge::Mirroring;
use super::renderer::frame::Frame;

#[rustfmt::skip]
pub static SYSTEM_PALETTE: [(u8,u8,u8); 64] = [
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
    mem: [u8; 0x800],       // 2 KB VRAM (nametables)
    palette: [u8; 0x20],    // 32 bytes palette RAM
    chr_rom: Vec<u8>,       // CHR ROM from cartridge
    ctrl: ControlRegister,
    mask: MaskRegister,
    addr: AddressRegister,
    status: StatusRegister,
    oam_addr: u8,
    oam: [u8; 64 * 4],
    data_latch: u8,

    scroll_x: u8,
    scroll_y: u8,
    scroll_latch: bool,     // false = first write (X), true = second write (Y)

    cycles: usize,
    scanlines: usize,

    nmi_occurred: Option<u8>,
    mirroring: Mirroring,

    frame: Frame,
    frame_ready: bool,
}

impl Ppu {
    pub fn new(mirroring: Mirroring) -> Ppu {
        Ppu {
            mem: [0; 0x800],
            palette: [0; 0x20],
            chr_rom: Vec::new(),
            ctrl: ControlRegister::default(),
            mask: MaskRegister::default(),
            addr: AddressRegister::default(),
            status: StatusRegister::default(),
            oam_addr: 0,
            oam: [0; 64 * 4],
            data_latch: 0,
            scroll_x: 0,
            scroll_y: 0,
            scroll_latch: false,
            cycles: 21,
            scanlines: 0,
            nmi_occurred: None,
            mirroring,
            frame: Frame::new(),
            frame_ready: false,
        }
    }
    
    pub fn tick(&mut self, tick: u8) -> bool {
        self.cycles += tick as usize;
        if self.cycles >= 341 {
            self.scanlines += 1;
            self.cycles %= 341;

            if self.scanlines == 241 {
                self.status.set_vblank(true);
                if self.ctrl.get_generate_nmi() {
                    self.nmi_occurred = Some(1);
                }

                // Render the frame at the start of vblank
                self.render_frame();
                self.frame_ready = true;
            }

            if self.scanlines >= 262 {
                self.scanlines = 0;
                self.status.set_vblank(false);
                self.status.set_sprite0_hit(false);
                self.status.set_sprite_overflow(false);
                self.nmi_occurred = None;
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
        self.chr_rom = chr.clone();
    }

    pub fn ctrl(&mut self, arg: u8) {
        let before_nmi_status = self.ctrl.get_generate_nmi();
        self.ctrl.update(arg);
        if !before_nmi_status && self.ctrl.get_generate_nmi() && self.status.get_vblank() {
            self.nmi_occurred = Some(1);
        }
    }

    pub fn mask(&mut self, arg: u8) {
        self.mask.update(arg);
    }

    pub fn status(&mut self) -> u8 {
        let result = self.status.get();
        // Reading status clears vblank flag
        self.status.set_vblank(false);
        // Reading status also resets the address latch
        self.addr.reset_latch();
        self.scroll_latch = false;
        result
    }

    pub fn scroll(&mut self, value: u8) {
        if !self.scroll_latch {
            self.scroll_x = value;
        } else {
            self.scroll_y = value;
        }
        self.scroll_latch = !self.scroll_latch;
    }

    pub fn addr(&mut self, value: u8) {
        self.addr.write_byte(value);
    }

    /// Read from VRAM through the internal address bus.
    /// Maps the address to the correct memory region.
    fn vram_read(&self, addr: u16) -> u8 {
        let addr = addr & 0x3FFF; // Mirror above 0x3FFF
        match addr {
            // Pattern tables — read from CHR ROM
            0x0000..=0x1FFF => {
                if (addr as usize) < self.chr_rom.len() {
                    self.chr_rom[addr as usize]
                } else {
                    0
                }
            }
            // Nametables
            0x2000..=0x3EFF => {
                let mirrored = self.mirror_nametable_addr(addr);
                self.mem[mirrored]
            }
            // Palette
            0x3F00..=0x3FFF => {
                let mut palette_addr = (addr - 0x3F00) & 0x1F;
                // Mirror background palette entries
                if palette_addr == 0x10 || palette_addr == 0x14 || palette_addr == 0x18 || palette_addr == 0x1C {
                    palette_addr -= 0x10;
                }
                self.palette[palette_addr as usize]
            }
            _ => 0,
        }
    }

    /// Write to VRAM through the internal address bus.
    fn vram_write(&mut self, addr: u16, value: u8) {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => {
                // CHR ROM — typically not writable, but CHR RAM would be
                // For now, ignore writes to pattern tables
            }
            0x2000..=0x3EFF => {
                let mirrored = self.mirror_nametable_addr(addr);
                self.mem[mirrored] = value;
            }
            0x3F00..=0x3FFF => {
                let mut palette_addr = (addr - 0x3F00) & 0x1F;
                if palette_addr == 0x10 || palette_addr == 0x14 || palette_addr == 0x18 || palette_addr == 0x1C {
                    palette_addr -= 0x10;
                }
                self.palette[palette_addr as usize] = value;
            }
            _ => {}
        }
    }

    /// Convert a nametable address (0x2000-0x3EFF) into a VRAM index (0x000-0x7FF)
    /// based on the cartridge mirroring mode.
    fn mirror_nametable_addr(&self, addr: u16) -> usize {
        let addr = (addr - 0x2000) & 0x0FFF; // Wrap to 0x000-0xFFF range
        let nametable = addr / 0x400;         // Which nametable (0-3)
        let offset = addr % 0x400;            // Offset within nametable

        let mirrored_table = match self.mirroring {
            Mirroring::Horizontal => {
                // NT0 and NT1 map to VRAM 0x000, NT2 and NT3 map to VRAM 0x400
                match nametable {
                    0 | 1 => 0,
                    2 | 3 => 1,
                    _ => 0,
                }
            }
            Mirroring::Vertical => {
                // NT0 and NT2 map to VRAM 0x000, NT1 and NT3 map to VRAM 0x400
                match nametable {
                    0 | 2 => 0,
                    1 | 3 => 1,
                    _ => 0,
                }
            }
        };

        (mirrored_table * 0x400 + offset as usize) as usize
    }

    pub fn data_read(&mut self) -> u8 {
        let addr = self.addr.get();
        let is_palette = matches!(addr, 0x3F00..=0x3FFF);

        if is_palette {
            // Palette reads are not buffered
            self.data_latch = self.vram_read(addr - 0x1000); // Latch gets the nametable byte "underneath"
            let result = self.vram_read(addr);
            self.addr.increment(self.ctrl.get_vram_increment());
            result
        } else {
            // Non-palette reads are buffered (dummy read)
            let previous_data = self.data_latch;
            self.data_latch = self.vram_read(addr);
            self.addr.increment(self.ctrl.get_vram_increment());
            previous_data
        }
    }

    pub fn data_write(&mut self, value: u8) {
        let addr = self.addr.get();
        self.vram_write(addr, value);
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

    pub fn get_nmi_occurred(&mut self) -> bool {
        self.nmi_occurred.take().is_some()
    }

    pub fn take_frame(&mut self) -> Option<Frame> {
        if self.frame_ready {
            self.frame_ready = false;
            let frame = std::mem::replace(&mut self.frame, Frame::new());
            Some(frame)
        } else {
            None
        }
    }

    // ─── Rendering ───────────────────────────────────────────────────────

    fn render_frame(&mut self) {
        self.render_background();
        self.render_sprites();
    }

    fn render_background(&mut self) {
        let bg_pattern_base = self.ctrl.get_background_pattern_addr();
        let nametable_base: u16 = self.ctrl.get_nametable_addr();

        // Render the 32x30 tile grid
        for tile_row in 0..30u16 {
            for tile_col in 0..32u16 {
                let nametable_addr = nametable_base + tile_row * 32 + tile_col;
                let tile_index = self.vram_read(nametable_addr) as u16;

                // Get the attribute byte for this tile's 2x2 metatile region
                let attr_table_addr = nametable_base + 0x3C0 
                    + (tile_row / 4) * 8 
                    + (tile_col / 4);
                let attr_byte = self.vram_read(attr_table_addr);
                
                // Each attribute byte covers a 4x4 tile area.
                // Which 2x2 quadrant is this tile in?
                let quadrant_x = (tile_col % 4) / 2;
                let quadrant_y = (tile_row % 4) / 2;
                let shift = (quadrant_y * 2 + quadrant_x) * 2;
                let palette_idx = ((attr_byte >> shift) & 0x03) as u16;

                // Render 8x8 pixel tile
                let tile_addr = bg_pattern_base + tile_index * 16;
                for row in 0..8u16 {
                    let lo = self.vram_read(tile_addr + row);
                    let hi = self.vram_read(tile_addr + row + 8);

                    for col in (0..8u16).rev() {
                        let bit0 = (lo >> col) & 1;
                        let bit1 = (hi >> col) & 1;
                        let color_idx = (bit1 << 1) | bit0;

                        let palette_entry = if color_idx == 0 {
                            // Universal background color
                            self.vram_read(0x3F00)
                        } else {
                            self.vram_read(0x3F00 + palette_idx * 4 + color_idx as u16)
                        };

                        let color = SYSTEM_PALETTE[(palette_entry & 0x3F) as usize];
                        let pixel_x = tile_col as usize * 8 + (7 - col as usize);
                        let pixel_y = tile_row as usize * 8 + row as usize;
                        self.frame.set_pixel(pixel_x, pixel_y, color);
                    }
                }
            }
        }
    }

    fn render_sprites(&mut self) {
        let sprite_pattern_base = self.ctrl.get_sprite_pattern_addr();

        // Sprites with lower OAM indices have higher priority (drawn on top).
        // Iterate in reverse so lower-index sprites overwrite higher-index ones.
        for i in (0..64).rev() {
            let oam_base = i * 4;
            let y_pos = self.oam[oam_base] as u16;
            let tile_index = self.oam[oam_base + 1] as u16;
            let attributes = self.oam[oam_base + 2];
            let x_pos = self.oam[oam_base + 3] as u16;

            // Skip sprites at Y=0 or off-screen
            if y_pos == 0 || y_pos >= 0xEF {
                continue;
            }

            let palette_idx = (attributes & 0x03) as u16;
            let flip_h = attributes & 0x40 != 0;
            let flip_v = attributes & 0x80 != 0;
            let behind_bg = attributes & 0x20 != 0;

            let tile_addr = sprite_pattern_base + tile_index * 16;

            for row in 0..8u16 {
                let actual_row = if flip_v { 7 - row } else { row };
                let lo = self.vram_read(tile_addr + actual_row);
                let hi = self.vram_read(tile_addr + actual_row + 8);

                for col in (0..8u16).rev() {
                    let actual_col = if flip_h { 7 - col } else { col };
                    let bit0 = (lo >> actual_col) & 1;
                    let bit1 = (hi >> actual_col) & 1;
                    let color_idx = (bit1 << 1) | bit0;

                    // Transparent pixel
                    if color_idx == 0 {
                        continue;
                    }

                    let palette_entry = self.vram_read(0x3F10 + palette_idx * 4 + color_idx as u16);
                    let color = SYSTEM_PALETTE[(palette_entry & 0x3F) as usize];

                    let pixel_x = (x_pos + 7 - col) as usize;
                    let pixel_y = (y_pos + 1 + row) as usize; // OAM Y is off by 1

                    if pixel_x < 256 && pixel_y < 240 {
                        if !behind_bg {
                            self.frame.set_pixel(pixel_x, pixel_y, color);
                        }
                        // TODO: behind_bg sprites should only show through transparent BG pixels
                    }
                }
            }
        }
    }
}
