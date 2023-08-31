use std::{cell::RefCell, default, fs, rc::Rc};

use super::{bus::Bus, Nes};

pub struct Cartridge {
    bus: Rc<RefCell<Bus>>,
}

#[derive(Default)]
struct NesHeader {
    signature: [u8; 4],
    prg_size: u8,
    chr_size: u8,
    flags_6: u8,
    flags_7: u8,
    flags_8: u8,
    flags_9: u8,
    flags_10: u8,
}

struct NesImage {
    header: NesHeader,
}

impl NesImage {
    fn parse(_bytes: &[u8]) -> Self {
        let header = NesHeader::default();
        NesImage { header }
    }
}

impl Cartridge {
    pub fn new(bus: Rc<RefCell<Bus>>) -> Cartridge {
        Cartridge { bus }
    }

    pub fn load(&mut self, path: &str) {
        let bytes = fs::read(path).expect(format!("Cannot fine file: {}", path).as_str());

        let prg_rom = &bytes[16..][..0x4000];

        let mut load_addr: u16 = 0x8000;
        for byte in prg_rom {
            self.bus.borrow_mut().write_u8(load_addr, *byte);
            load_addr += 0x1;
        }

        load_addr = 0xC000 - 0x1;
        for byte in prg_rom {
            load_addr += 0x1;
            self.bus.borrow_mut().write_u8(load_addr, *byte);
        }
    }
}
