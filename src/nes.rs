mod apu;
mod bus;
mod cartridge;
mod cpu;
mod joypad;
mod ppu;
mod renderer;

use std::sync::{Arc, Mutex};

use bus::Bus;
use cartridge::Cartridge;
use cpu::Cpu;
use ppu::Ppu;
use renderer::Renderer;

pub struct Nes {
    cpu: Cpu,
    renderer: Renderer,
}

impl Nes {
    pub fn new(rom_path: &str) -> Result<Nes, String> {
        let cartridge = Cartridge::new(rom_path)?;
        let mirroring = cartridge.mirroring;
        let ppu = Ppu::new(mirroring);

        // Shared audio buffer between APU and SDL2 audio callback
        let audio_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(44100)));

        let bus = Bus::new(cartridge, ppu, audio_buffer.clone());
        let cpu = Cpu::new(bus);
        let renderer = Renderer::new(audio_buffer);

        Ok(Nes { cpu, renderer })
    }

    pub fn run(&mut self) {
        use std::time::{Duration, Instant};

        const FRAME_DURATION: Duration = Duration::from_nanos(16_666_667); // ~60 FPS

        self.cpu.power_up();

        loop {
            let frame_start = Instant::now();

            // Run CPU until the PPU produces a frame
            loop {
                self.cpu.execute();

                if let Some(frame) = self.cpu.take_frame() {
                    self.renderer.render_frame(&frame);
                    break;
                }
            }

            // Poll SDL events and handle input
            match self.renderer.poll_events() {
                None => return, // Quit requested
                Some(key_events) => {
                    for (button, pressed) in key_events {
                        self.cpu.set_joypad_button(button, pressed);
                    }
                }
            }

            // Frame timing — sleep if we finished early to maintain ~60 FPS
            let elapsed = frame_start.elapsed();
            if elapsed < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - elapsed);
            }
        }
    }
}
