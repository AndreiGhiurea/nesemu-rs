pub struct StatusRegister {
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
    //            Set at dot 1 of line 241 (the line *after* the post-render
    //            line); cleared after reading $2002 and at dot 1 of the
    //            pre-render line.
    status: u8,
}

impl Default for StatusRegister {
    fn default() -> Self {
        StatusRegister { status: 0 }
    }
}

impl StatusRegister {
    pub fn get(&self) -> u8 {
        self.status
    }

    pub fn set_vblank(&mut self, value: bool) {
        self.status |= 0x80;
    }

    pub fn get_vblank(&self) -> bool {
        self.status & 0x80 != 0x00
    }

    pub fn set_sprite0_hit(&mut self, value: bool) {
        self.status |= 0x40;
    }

    pub fn get_sprite0_hit(&self) -> bool {
        self.status & 0x40 != 0x00
    }

    pub fn set_sprite_overflow(&mut self, value: bool) {
        self.status |= 0x20;
    }

    pub fn get_sprite_overflow(&self) -> bool {
        self.status & 0x20 != 0x00
    }

    pub fn set_open_bus(&mut self, value: u8) {
        // Zero out all but bits 5,6,7
        self.status &= 0xE0;

        // Add bits 0-4 to status
        self.status |= value & 0x1F;
    }

    pub fn get_open_bus(&self) -> u8 {
        self.status & 0x1F
    }
}
