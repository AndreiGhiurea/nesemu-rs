use crate::nes::cpu::Addr;

pub struct ControlRegister {
    nametable_addr: Addr,
    sprite_pattern_addr: Addr,
    backgroung_pattern_addr: Addr,
    vram_increment: VramIncrement,
    sprite_size: SpriteSize,
    ppu_select: PpuSelect,
    generate_nmi: bool,
}

enum VramIncrement {
    Add1,
    Add32,
}

enum SpriteSize {
    Size8x8,
    Size8x16,
}

enum PpuSelect {
    ReadBackdrop,
    OutputColor,
}

impl Default for ControlRegister {
    fn default() -> Self {
        ControlRegister {
            nametable_addr: 0,
            sprite_pattern_addr: 0,
            backgroung_pattern_addr: 0,
            vram_increment: VramIncrement::Add1,
            sprite_size: SpriteSize::Size8x8,
            ppu_select: PpuSelect::ReadBackdrop,
            generate_nmi: false,
        }
    }
}

impl ControlRegister {
    pub fn update(&mut self, arg: u8) {
        // 7  bit  0
        // ---- ----
        // VPHB SINN
        // |||| ||||
        // |||| ||++- Base nametable address
        // |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
        // |||| |+--- VRAM address increment per CPU read/write of PPUDATA
        // |||| |     (0: add 1, going across; 1: add 32, going down)
        // |||| +---- Sprite pattern table address for 8x8 sprites
        // ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
        // |||+------ Background pattern table address (0: $0000; 1: $1000)
        // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels – see PPU OAM#Byte 1)
        // |+-------- PPU master/slave select
        // |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
        // +--------- Generate an NMI at the start of the
        //            vertical blanking interval (0: off; 1: on)
        self.nametable_addr = match arg & 0b11 {
            0 => 0x2000 as Addr,
            1 => 0x2400 as Addr,
            2 => 0x2800 as Addr,
            3 => 0x2C00 as Addr,
            _ => panic!("Unexpected error, all bits besides 0 and 1 should've been 0"),
        };

        self.vram_increment = if arg & 0b100 != 0 {
            VramIncrement::Add32
        } else {
            VramIncrement::Add1
        };

        self.sprite_pattern_addr = if arg & 0b1000 != 0 {
            0x1000 as Addr
        } else {
            0x0000 as Addr
        };

        self.backgroung_pattern_addr = if arg & 0b1_0000 != 0 {
            0x1000 as Addr
        } else {
            0x0000 as Addr
        };

        self.sprite_size = if arg & 0b10_0000 != 0 {
            SpriteSize::Size8x16
        } else {
            SpriteSize::Size8x8
        };

        self.ppu_select = if arg & 0b100_0000 != 0 {
            PpuSelect::OutputColor
        } else {
            PpuSelect::ReadBackdrop
        };

        self.generate_nmi = arg & 0b1000_0000 != 0;

        let scroll_x = if arg & 0b1 != 0 { 256 } else { 0 };
        let scroll_y = if arg & 0b10 != 0 { 240 } else { 0 };
    }
}
