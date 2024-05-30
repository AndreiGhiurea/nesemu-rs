mod bus;
mod cartridge;
mod cpu;
mod ppu;
mod renderer;

use bus::Bus;
use cartridge::Cartridge;
use cpu::Cpu;
use ppu::Ppu;

pub struct Nes {
    cpu: Cpu,
}

impl Nes {
    pub fn new(rom_path: &str) -> Result<Nes, String> {
        let cartridge = Cartridge::new(rom_path)?;
        let ppu = Ppu::new();
        let bus = Bus::new(cartridge, ppu);
        let cpu = Cpu::new(bus);

        Ok(Nes { cpu })
    }

    pub fn run(&mut self) {
        self.cpu.power_up();

        loop {
            self.cpu.execute();
        }
    }
}
