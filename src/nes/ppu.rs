mod addr_reg;
mod ctrl_reg;
mod mask_reg;

use addr_reg::AddressRegister;
use ctrl_reg::ControlRegister;
use mask_reg::MaskRegister;

pub struct Ppu {
    mem: [u8; 0x4000],
    ctrl: ControlRegister,
    mask: MaskRegister,
    addr: AddressRegister,
    oam_addr: u8,
    oam: [u8; 64 * 4],
    data_latch: u8,

    cycles: usize,
    scanlines: usize,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            mem: [0; 0x4000],
            ctrl: ControlRegister::default(),
            mask: MaskRegister::default(),
            addr: AddressRegister::default(),
            oam_addr: 0,
            oam: [0; 64 * 4],
            data_latch: 0,
            cycles: 21,
            scanlines: 0,
        }
    }

    pub fn tick(&mut self, tick: u8) {
        self.cycles += tick as usize;
        if self.cycles >= 341 {
            self.scanlines += 1;
            self.cycles %= 341;
        }
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
        self.ctrl.update(arg);
    }

    pub fn mask(&mut self, arg: u8) {
        self.mask.update(arg);
    }

    pub fn status(&self) -> u8 {
        // 7  bit  0
        // ---- ----
        // VSO. ....
        // |||| ||||
        // |||+-++++- PPU open bus. Returns stale PPU bus contents.
        // ||+------- Sprite overflow. The intent was for this flag to be set
        // ||         whenever more than eight sprites appear on a scanline, but a
        // ||         hardware bug causes the actual behavior to be more complicated
        // ||         and generate false positives as well as false negatives; see
        // ||         PPU sprite evaluation. This flag is set during sprite
        // ||         evaluation and cleared at dot 1 (the second dot) of the
        // ||         pre-render line.
        // |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
        // |          a nonzero background pixel; cleared at dot 1 of the pre-render
        // |          line.  Used for raster timing.
        // +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
        //         Set at dot 1 of line 241 (the line *after* the post-render
        //         line); cleared after reading $2002 and at dot 1 of the
        //         pre-render line.
        todo!("PPU Status not implemented");
    }

    pub fn scroll(&mut self) {
        todo!("PPU Scroll not implemented");
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
        self.oam_addr += 1;
    }

    pub fn oam_dma(&mut self, data: &[u8; 256]) {
        self.oam.copy_from_slice(data);
    }
}
