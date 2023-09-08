mod ctrl_reg;
mod mask_reg;

use ctrl_reg::ControlRegister;
use mask_reg::MaskRegister;

const PATTERN_TABLE_SIZE: usize = 0x1000;
const NAMETABLE_SIZE: usize = 0x400;

pub struct Ppu {
    mem: [u8; 0x4000],
    ctrl: ControlRegister,
    mask: MaskRegister,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            mem: [0x0; 0x4000],
            ctrl: ControlRegister::default(),
            mask: MaskRegister::default(),
        }
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
        0x0
    }
}
