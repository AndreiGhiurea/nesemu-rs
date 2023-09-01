mod bus;
mod cartridge;
mod cpu;

use bus::Bus;
use cartridge::Cartridge;
use cpu::Cpu;
use std::{cell::RefCell, rc::Rc};

pub struct Nes {
    bus: Rc<RefCell<Bus>>,
    cpu: Cpu,
    cartridge: Cartridge,
}

impl Nes {
    pub fn new() -> Nes {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let cpu = Cpu::new(bus.clone());
        let cartridge = Cartridge::new(bus.clone());
        Nes {
            bus,
            cpu,
            cartridge,
        }
    }

    pub fn run(&mut self) {
        self.cartridge.load("testroms/nestest.nes");
        self.cpu.set_pc(0xC000);

        loop {
            self.cpu.execute();
        }
    }
}
