pub struct MaskRegister {
    greyscale: bool,
    show_leftmost_background: bool,
    show_leftmost_sprites: bool,
    show_background: bool,
    show_sprites: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,
}

impl Default for MaskRegister {
    fn default() -> Self {
        MaskRegister {
            greyscale: false,
            show_leftmost_background: false,
            show_leftmost_sprites: false,
            show_background: false,
            show_sprites: false,
            emphasize_red: false,
            emphasize_green: false,
            emphasize_blue: false,
        }
    }
}

impl MaskRegister {
    pub fn update(&mut self, arg: u8) {
        // 7  bit  0
        // ---- ----
        // BGRs bMmG
        // |||| ||||
        // |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
        // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
        // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
        // |||| +---- 1: Show background
        // |||+------ 1: Show sprites
        // ||+------- Emphasize red (green on PAL/Dendy)
        // |+-------- Emphasize green (red on PAL/Dendy)
        // +--------- Emphasize blue
        self.greyscale = arg & 0b1 != 0;
        self.show_leftmost_background = arg & 0b10 != 0;
        self.show_leftmost_sprites = arg & 0b100 != 0;
        self.show_background = arg & 0b1000 != 0;
        self.show_sprites = arg & 0b1_0000 != 0;
        self.emphasize_red = arg & 0b10_0000 != 0;
        self.emphasize_green = arg & 0b100_0000 != 0;
        self.emphasize_blue = arg & 0b1000_0000 != 0;
    }
}
