use super::cpu::Addr;

pub struct Ppu {
    mem: [u8; 0x4000],
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

impl Ppu {
    pub fn new() -> Ppu {
        Ppu { mem: [0x0; 0x4000] }
    }

    pub fn ppuctrl(arg: u8) {
        let base_nametable_addr = match arg & 0b11 {
            0b00 => 0x2000 as Addr,
            0b01 => 0x2400 as Addr,
            0b10 => 0x2800 as Addr,
            0b11 => 0x2C00 as Addr,
            _ => panic!("Unexpected error, all bits besides 0 and 1 should've been 0"),
        };

        let vram_increment = if arg & 0b100 != 0 {
            VramIncrement::Add32
        } else {
            VramIncrement::Add1
        };

        let sprite_pattern_table = if arg & 0b1000 != 0 {
            0x1000 as Addr
        } else {
            0x0000 as Addr
        };

        let background_pattern_table = if arg & 0b1_0000 != 0 {
            0x1000 as Addr
        } else {
            0x0000 as Addr
        };

        let sprite_size = if arg & 0b10_0000 != 0 {
            SpriteSize::Size8x16
        } else {
            SpriteSize::Size8x8
        };

        let ppu_select = if arg & 0b100_0000 != 0 {
            PpuSelect::OutputColor
        } else {
            PpuSelect::ReadBackdrop
        };

        let generate_nmi = arg & 0b1000_0000 != 0;

        let scroll_x = if arg & 0b1 != 0 { 256 } else { 0 };
        let scroll_y = if arg & 0b10 != 0 { 240 } else { 0 };
    }
}
