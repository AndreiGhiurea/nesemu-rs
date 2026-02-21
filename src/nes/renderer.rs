pub mod frame;

use std::sync::{Arc, Mutex};

use super::apu::SAMPLE_RATE;
use super::joypad::JoypadButton;
use frame::Frame;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::{self, render::Canvas, video::Window, EventPump};

/// SDL2 audio callback that reads from the shared sample buffer
struct NesAudioCallback {
    buffer: Arc<Mutex<Vec<f32>>>,
    last_sample: f32,
}

impl AudioCallback for NesAudioCallback {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        if let Ok(mut buf) = self.buffer.lock() {
            let available = buf.len().min(out.len());
            for i in 0..available {
                out[i] = buf[i];
                self.last_sample = buf[i];
            }
            // On underrun, hold last sample value to avoid clicks
            for i in available..out.len() {
                out[i] = self.last_sample;
                // Gently fade toward silence to avoid sustained DC
                self.last_sample *= 0.999;
            }
            buf.drain(0..available);
        } else {
            for sample in out.iter_mut() {
                *sample = self.last_sample;
                self.last_sample *= 0.999;
            }
        }
    }
}

pub struct Renderer {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    // Keep the audio device alive (dropping it stops audio)
    _audio_device: sdl2::audio::AudioDevice<NesAudioCallback>,
}

impl Renderer {
    pub fn new(audio_buffer: Arc<Mutex<Vec<f32>>>) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();

        // Video setup
        let window = video_subsystem
            .window("NES Emulator", 256 * 3, 240 * 3)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        canvas.set_scale(3.0, 3.0).unwrap();

        // Audio setup
        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1),  // Mono
            samples: Some(1024), // Buffer size (1024 = ~23ms at 44.1kHz)
        };

        let audio_device = audio_subsystem
            .open_playback(None, &desired_spec, |_spec| {
                NesAudioCallback {
                    buffer: audio_buffer,
                    last_sample: 0.0,
                }
            })
            .unwrap();

        // Start audio playback
        audio_device.resume();

        let event_pump = sdl_context.event_pump().unwrap();

        Renderer {
            canvas,
            event_pump,
            _audio_device: audio_device,
        }
    }

    pub fn render_frame(&mut self, frame: &Frame) {
        let creator = self.canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
            .unwrap();

        texture.update(None, &frame.data, 256 * 3).unwrap();
        self.canvas.copy(&texture, None, None).unwrap();
        self.canvas.present();
    }

    /// Poll SDL2 events. Returns `None` if the user wants to quit,
    /// or `Some(Vec)` of (button, pressed) pairs for joypad updates.
    pub fn poll_events(&mut self) -> Option<Vec<(JoypadButton, bool)>> {
        let mut key_events = Vec::new();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return None,

                Event::KeyDown { keycode: Some(key), .. } => {
                    if let Some(button) = keycode_to_button(key) {
                        key_events.push((button, true));
                    }
                }

                Event::KeyUp { keycode: Some(key), .. } => {
                    if let Some(button) = keycode_to_button(key) {
                        key_events.push((button, false));
                    }
                }

                _ => {}
            }
        }

        Some(key_events)
    }
}

fn keycode_to_button(keycode: Keycode) -> Option<JoypadButton> {
    match keycode {
        Keycode::Z      => Some(JoypadButton::A),
        Keycode::X      => Some(JoypadButton::B),
        Keycode::Space  => Some(JoypadButton::SELECT),
        Keycode::Return => Some(JoypadButton::START),
        Keycode::Up     => Some(JoypadButton::UP),
        Keycode::Down   => Some(JoypadButton::DOWN),
        Keycode::Left   => Some(JoypadButton::LEFT),
        Keycode::Right  => Some(JoypadButton::RIGHT),
        _ => None,
    }
}
