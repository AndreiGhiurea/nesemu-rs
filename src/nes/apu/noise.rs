/// NES APU Noise Channel
///
/// Generates pseudo-random noise using a 15-bit linear feedback shift register (LFSR).
/// Two modes: long (bit 1 feedback) and short (bit 6 feedback) for different timbres.

use super::pulse::LENGTH_TABLE;

/// Timer period lookup table for the noise channel
const NOISE_PERIOD_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

pub struct NoiseChannel {
    enabled: bool,

    // Timer
    timer_period: u16,
    timer_value: u16,

    // LFSR
    shift_register: u16,
    mode: bool, // false = long mode (bit 1), true = short mode (bit 6)

    // Length counter
    length_counter: u8,
    length_halt: bool,

    // Envelope
    envelope_start: bool,
    envelope_loop: bool,
    constant_volume: bool,
    envelope_period: u8,
    envelope_divider: u8,
    envelope_decay: u8,
    volume: u8,
}

impl NoiseChannel {
    pub fn new() -> Self {
        NoiseChannel {
            enabled: false,
            timer_period: 0,
            timer_value: 0,
            shift_register: 1, // Must be non-zero
            mode: false,
            length_counter: 0,
            length_halt: false,
            envelope_start: false,
            envelope_loop: false,
            constant_volume: false,
            envelope_period: 0,
            envelope_divider: 0,
            envelope_decay: 0,
            volume: 0,
        }
    }

    /// $400C — Envelope and length counter halt
    pub fn write_control(&mut self, value: u8) {
        self.length_halt = value & 0x20 != 0;
        self.envelope_loop = value & 0x20 != 0;
        self.constant_volume = value & 0x10 != 0;
        self.volume = value & 0x0F;
        self.envelope_period = value & 0x0F;
    }

    /// $400E — Mode and period
    pub fn write_mode_period(&mut self, value: u8) {
        self.mode = value & 0x80 != 0;
        self.timer_period = NOISE_PERIOD_TABLE[(value & 0x0F) as usize];
    }

    /// $400F — Length counter load
    pub fn write_length(&mut self, value: u8) {
        if self.enabled {
            self.length_counter = LENGTH_TABLE[(value >> 3) as usize];
        }
        self.envelope_start = true;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }

    pub fn length_counter(&self) -> u8 {
        self.length_counter
    }

    /// Called every APU cycle
    pub fn tick_timer(&mut self) {
        if self.timer_value == 0 {
            self.timer_value = self.timer_period;
            self.clock_shift_register();
        } else {
            self.timer_value -= 1;
        }
    }

    fn clock_shift_register(&mut self) {
        let feedback_bit = if self.mode { 6 } else { 1 };
        let feedback = (self.shift_register & 1) ^ ((self.shift_register >> feedback_bit) & 1);
        self.shift_register >>= 1;
        self.shift_register |= feedback << 14;
    }

    /// Quarter frame: clock the envelope
    pub fn tick_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.envelope_decay = 15;
            self.envelope_divider = self.envelope_period;
        } else if self.envelope_divider == 0 {
            self.envelope_divider = self.envelope_period;
            if self.envelope_decay > 0 {
                self.envelope_decay -= 1;
            } else if self.envelope_loop {
                self.envelope_decay = 15;
            }
        } else {
            self.envelope_divider -= 1;
        }
    }

    /// Half frame: clock the length counter
    pub fn tick_length_counter(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled || self.length_counter == 0 || self.shift_register & 1 != 0 {
            0
        } else if self.constant_volume {
            self.volume
        } else {
            self.envelope_decay
        }
    }
}
