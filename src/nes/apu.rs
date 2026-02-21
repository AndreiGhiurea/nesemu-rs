/// NES Audio Processing Unit (APU)
///
/// Manages 5 audio channels, a frame counter for timing, and
/// mixes all channels into an audio sample buffer for SDL2.
///
/// Audio pipeline:
/// 1. Each channel produces raw output every CPU/APU cycle (~1.79 MHz)
/// 2. Non-linear mixer combines all channels
/// 3. Decimation with weighted averaging downsamples to 44.1 kHz
/// 4. NES hardware-accurate filter chain:
///    - 1st-order high-pass @ ~37 Hz  (capacitor coupling in NES)
///    - 1st-order high-pass @ ~440 Hz (AC coupling on output)
///    - 1st-order low-pass  @ ~14 kHz (anti-aliasing in NES DAC)

pub mod pulse;
pub mod triangle;
pub mod noise;
pub mod dmc;

use std::sync::{Arc, Mutex};

use pulse::PulseChannel;
use triangle::TriangleChannel;
use noise::NoiseChannel;
use dmc::DmcChannel;

/// NES CPU clock rate (~1.789773 MHz NTSC)
const CPU_FREQ: f64 = 1_789_773.0;
/// Target audio sample rate
pub const SAMPLE_RATE: u32 = 44_100;

/// Pulse output lookup table for non-linear mixing
fn pulse_table() -> [f32; 31] {
    let mut table = [0.0f32; 31];
    for n in 1..31 {
        table[n] = 95.52 / (8128.0 / n as f32 + 100.0);
    }
    table
}

/// TND (triangle/noise/dmc) output lookup table for non-linear mixing
fn tnd_table() -> [f32; 203] {
    let mut table = [0.0f32; 203];
    for n in 1..203 {
        table[n] = 163.67 / (24329.0 / n as f32 + 100.0);
    }
    table
}

/// First-order IIR filter (used for both high-pass and low-pass)
struct FirstOrderFilter {
    b0: f64,
    b1: f64,
    a1: f64,
    prev_in: f64,
    prev_out: f64,
}

impl FirstOrderFilter {
    /// Create a high-pass filter for the given cutoff frequency and sample rate
    fn high_pass(cutoff_hz: f64, sample_rate: f64) -> Self {
        let rc = 1.0 / (2.0 * std::f64::consts::PI * cutoff_hz);
        let dt = 1.0 / sample_rate;
        let alpha = rc / (rc + dt);
        FirstOrderFilter {
            b0: alpha,
            b1: -alpha,
            a1: -alpha,
            prev_in: 0.0,
            prev_out: 0.0,
        }
    }

    /// Create a low-pass filter for the given cutoff frequency and sample rate
    fn low_pass(cutoff_hz: f64, sample_rate: f64) -> Self {
        let rc = 1.0 / (2.0 * std::f64::consts::PI * cutoff_hz);
        let dt = 1.0 / sample_rate;
        let alpha = dt / (rc + dt);
        FirstOrderFilter {
            b0: alpha,
            b1: 0.0,
            a1: -(1.0 - alpha),
            prev_in: 0.0,
            prev_out: 0.0,
        }
    }

    fn process(&mut self, input: f64) -> f64 {
        let output = self.b0 * input + self.b1 * self.prev_in - self.a1 * self.prev_out;
        self.prev_in = input;
        self.prev_out = output;
        output
    }
}

pub struct Apu {
    pulse1: PulseChannel,
    pulse2: PulseChannel,
    triangle: TriangleChannel,
    noise: NoiseChannel,
    dmc: DmcChannel,

    // Frame counter
    frame_counter_mode: u8,    // 0 = 4-step, 1 = 5-step
    frame_counter_value: usize,
    irq_inhibit: bool,
    pub frame_interrupt: bool,

    // Timing
    cpu_cycles: usize,
    even_cycle: bool,

    // Decimation accumulator (averages ~40.6 raw samples per output sample)
    sample_counter: f64,
    sample_period: f64,
    sample_sum: f64,
    sample_count: u32,

    // Audio buffer shared with SDL2 callback
    audio_buffer: Arc<Mutex<Vec<f32>>>,

    // Lookup tables
    pulse_table: [f32; 31],
    tnd_table: [f32; 203],

    // NES hardware-accurate filter chain
    hp_37hz: FirstOrderFilter,    // Capacitor coupling
    hp_90hz: FirstOrderFilter,    // AC coupling on output
    lp_14khz: FirstOrderFilter,   // DAC anti-aliasing

    // Local sample batch to avoid per-sample mutex locks
    sample_batch: Vec<f32>,
}

impl Apu {
    pub fn new(audio_buffer: Arc<Mutex<Vec<f32>>>) -> Self {
        let sr = SAMPLE_RATE as f64;
        Apu {
            pulse1: PulseChannel::new(1),
            pulse2: PulseChannel::new(2),
            triangle: TriangleChannel::new(),
            noise: NoiseChannel::new(),
            dmc: DmcChannel::new(),
            frame_counter_mode: 0,
            frame_counter_value: 0,
            irq_inhibit: false,
            frame_interrupt: false,
            cpu_cycles: 0,
            even_cycle: false,
            sample_counter: 0.0,
            sample_period: CPU_FREQ / sr,
            sample_sum: 0.0,
            sample_count: 0,
            audio_buffer,
            pulse_table: pulse_table(),
            tnd_table: tnd_table(),
            hp_37hz: FirstOrderFilter::high_pass(37.0, sr),
            hp_90hz: FirstOrderFilter::high_pass(90.0, sr),
            lp_14khz: FirstOrderFilter::low_pass(14000.0, sr),
            sample_batch: Vec::with_capacity(256),
        }
    }

    /// Called every CPU cycle.
    /// Returns Some(address) if the DMC needs a memory read.
    pub fn tick(&mut self) -> Option<u16> {
        self.cpu_cycles += 1;

        // Triangle timer ticks at CPU rate
        self.triangle.tick_timer();

        // DMC timer ticks at CPU rate
        let dmc_read = self.dmc.tick_timer();

        // Pulse, noise tick at APU rate (every other CPU cycle)
        self.even_cycle = !self.even_cycle;
        if self.even_cycle {
            self.pulse1.tick_timer();
            self.pulse2.tick_timer();
            self.noise.tick_timer();
            self.clock_frame_counter();
        }

        // Accumulate raw mix for decimation averaging
        // (~40.6 raw samples averaged per output sample: 1789773/44100)
        let raw = self.mix() as f64;
        self.sample_sum += raw;
        self.sample_count += 1;

        // Generate output sample at the target sample rate
        self.sample_counter += 1.0;
        if self.sample_counter >= self.sample_period {
            self.sample_counter -= self.sample_period;

            // Decimation: average all accumulated samples since last output
            let averaged = if self.sample_count > 0 {
                self.sample_sum / self.sample_count as f64
            } else {
                0.0
            };
            self.sample_sum = 0.0;
            self.sample_count = 0;

            // Apply NES hardware-accurate filter chain
            let filtered = self.hp_37hz.process(averaged);
            let filtered = self.hp_90hz.process(filtered);
            let filtered = self.lp_14khz.process(filtered);

            // Scale and soft-clip
            let output = (filtered * 1.8).clamp(-1.0, 1.0) as f32;

            // Batch locally, flush every 128 samples to minimize mutex locks
            self.sample_batch.push(output);
            if self.sample_batch.len() >= 128 {
                if let Ok(mut buf) = self.audio_buffer.lock() {
                    if buf.len() < 4096 {
                        let space = 4096 - buf.len();
                        let to_push = self.sample_batch.len().min(space);
                        buf.extend_from_slice(&self.sample_batch[..to_push]);
                    }
                }
                self.sample_batch.clear();
            }
        }

        dmc_read
    }

    /// Feed a byte read from memory into the DMC sample buffer
    pub fn dmc_fill_buffer(&mut self, value: u8) {
        self.dmc.fill_sample_buffer(value);
    }

    fn clock_frame_counter(&mut self) {
        let step = self.frame_counter_value;
        self.frame_counter_value += 1;

        if self.frame_counter_mode == 0 {
            // 4-step mode
            match step {
                3728 => self.quarter_frame(),
                7456 => { self.quarter_frame(); self.half_frame(); }
                11185 => self.quarter_frame(),
                14914 => {
                    self.quarter_frame();
                    self.half_frame();
                    if !self.irq_inhibit {
                        self.frame_interrupt = true;
                    }
                    self.frame_counter_value = 0;
                }
                _ => {}
            }
        } else {
            // 5-step mode
            match step {
                3728 => self.quarter_frame(),
                7456 => { self.quarter_frame(); self.half_frame(); }
                11185 => self.quarter_frame(),
                18640 => {
                    self.quarter_frame();
                    self.half_frame();
                    self.frame_counter_value = 0;
                }
                _ => {}
            }
        }
    }

    fn quarter_frame(&mut self) {
        self.pulse1.tick_envelope();
        self.pulse2.tick_envelope();
        self.triangle.tick_linear_counter();
        self.noise.tick_envelope();
    }

    fn half_frame(&mut self) {
        self.pulse1.tick_length_counter();
        self.pulse1.tick_sweep();
        self.pulse2.tick_length_counter();
        self.pulse2.tick_sweep();
        self.triangle.tick_length_counter();
        self.noise.tick_length_counter();
    }

    /// Mix all channels using the NES non-linear mixing formula
    fn mix(&self) -> f32 {
        let p1 = self.pulse1.output() as usize;
        let p2 = self.pulse2.output() as usize;
        let t = self.triangle.output() as usize;
        let n = self.noise.output() as usize;
        let d = self.dmc.output() as usize;

        let pulse_out = self.pulse_table[p1 + p2];
        let tnd_idx = (3 * t + 2 * n + d).min(202);
        let tnd_out = self.tnd_table[tnd_idx];

        pulse_out + tnd_out
    }

    // ─── Register writes ─────────────────────────────────────────────────

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            // Pulse 1
            0x4000 => self.pulse1.write_control(value),
            0x4001 => self.pulse1.write_sweep(value),
            0x4002 => self.pulse1.write_timer_lo(value),
            0x4003 => self.pulse1.write_timer_hi(value),

            // Pulse 2
            0x4004 => self.pulse2.write_control(value),
            0x4005 => self.pulse2.write_sweep(value),
            0x4006 => self.pulse2.write_timer_lo(value),
            0x4007 => self.pulse2.write_timer_hi(value),

            // Triangle
            0x4008 => self.triangle.write_linear_counter(value),
            0x4009 => {} // Unused
            0x400A => self.triangle.write_timer_lo(value),
            0x400B => self.triangle.write_timer_hi(value),

            // Noise
            0x400C => self.noise.write_control(value),
            0x400D => {} // Unused
            0x400E => self.noise.write_mode_period(value),
            0x400F => self.noise.write_length(value),

            // DMC
            0x4010 => self.dmc.write_control(value),
            0x4011 => self.dmc.write_direct_load(value),
            0x4012 => self.dmc.write_sample_address(value),
            0x4013 => self.dmc.write_sample_length(value),

            // Status
            0x4015 => self.write_status(value),

            // Frame counter
            0x4017 => self.write_frame_counter(value),

            _ => {}
        }
    }

    /// $4015 write — Enable/disable channels
    fn write_status(&mut self, value: u8) {
        self.pulse1.set_enabled(value & 0x01 != 0);
        self.pulse2.set_enabled(value & 0x02 != 0);
        self.triangle.set_enabled(value & 0x04 != 0);
        self.noise.set_enabled(value & 0x08 != 0);
        self.dmc.set_enabled(value & 0x10 != 0);
        self.dmc.interrupt_flag = false;
    }

    /// $4015 read — Channel status
    pub fn read_status(&mut self) -> u8 {
        let mut status = 0u8;
        if self.pulse1.length_counter() > 0 { status |= 0x01; }
        if self.pulse2.length_counter() > 0 { status |= 0x02; }
        if self.triangle.length_counter() > 0 { status |= 0x04; }
        if self.noise.length_counter() > 0 { status |= 0x08; }
        if self.dmc.bytes_remaining() > 0 { status |= 0x10; }
        if self.frame_interrupt { status |= 0x40; }
        if self.dmc.interrupt_flag { status |= 0x80; }

        // Reading status clears frame interrupt flag
        self.frame_interrupt = false;
        status
    }

    /// $4017 write — Frame counter mode and IRQ inhibit
    fn write_frame_counter(&mut self, value: u8) {
        self.frame_counter_mode = if value & 0x80 != 0 { 1 } else { 0 };
        self.irq_inhibit = value & 0x40 != 0;

        if self.irq_inhibit {
            self.frame_interrupt = false;
        }

        // If mode 1, immediately clock quarter and half frame
        if self.frame_counter_mode == 1 {
            self.quarter_frame();
            self.half_frame();
        }

        self.frame_counter_value = 0;
    }
}
