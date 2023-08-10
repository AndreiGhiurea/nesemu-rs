mod bus;
mod cpu;

use bus::Bus;
use cpu::Cpu;
use std::rc::Rc;

pub struct Nes {
    bus: Rc<Bus>,
    cpu: Cpu,
}

impl Nes {
    pub fn new() -> Nes {
        let bus = Rc::new(Bus::new());
        let cpu = Cpu::new(bus.clone());
        Nes { cpu, bus }
    }

    pub fn run(&mut self) {
        self.cpu.execute();
    }
}
