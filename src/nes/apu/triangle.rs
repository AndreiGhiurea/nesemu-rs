/// NES APU Triangle Wave Channel
///
/// Produces a triangle waveform using a 32-step sequence.
/// No volume control — it's either on or off. Commonly used for bass lines.

use super::pulse::LENGTH_TABLE;

/// The triangle channel's output sequence (32 steps)
const TRIANGLE_TABLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
];

pub struct TriangleChannel {
    enabled: bool,

    // Timer
    timer_period: u16,
    timer_value: u16,

    // Sequencer
    sequence_pos: u8,

    // Length counter
    length_counter: u8,
    length_halt: bool, // Also serves as linear counter control flag

    // Linear counter
    linear_counter: u8,
    linear_counter_reload: u8,
    linear_counter_reload_flag: bool,
}

impl TriangleChannel {
    pub fn new() -> Self {
        TriangleChannel {
            enabled: false,
            timer_period: 0,
            timer_value: 0,
            sequence_pos: 0,
            length_counter: 0,
            length_halt: false,
            linear_counter: 0,
            linear_counter_reload: 0,
            linear_counter_reload_flag: false,
        }
    }

    /// $4008 — Linear counter setup
    pub fn write_linear_counter(&mut self, value: u8) {
        self.length_halt = value & 0x80 != 0;
        self.linear_counter_reload = value & 0x7F;
    }

    /// $400A — Timer low 8 bits
    pub fn write_timer_lo(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x0700) | value as u16;
    }

    /// $400B — Length counter load, timer high 3 bits
    pub fn write_timer_hi(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | ((value as u16 & 0x07) << 8);
        if self.enabled {
            self.length_counter = LENGTH_TABLE[(value >> 3) as usize];
        }
        self.linear_counter_reload_flag = true;
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

    /// Called every CPU cycle (triangle timer ticks at CPU rate, not APU rate)
    pub fn tick_timer(&mut self) {
        if self.timer_value == 0 {
            self.timer_value = self.timer_period;
            // Only advance sequencer if both counters are non-zero
            if self.length_counter > 0 && self.linear_counter > 0 {
                self.sequence_pos = (self.sequence_pos + 1) % 32;
            }
        } else {
            self.timer_value -= 1;
        }
    }

    /// Quarter frame: clock the linear counter
    pub fn tick_linear_counter(&mut self) {
        if self.linear_counter_reload_flag {
            self.linear_counter = self.linear_counter_reload;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.length_halt {
            self.linear_counter_reload_flag = false;
        }
    }

    /// Half frame: clock the length counter
    pub fn tick_length_counter(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled || self.length_counter == 0 || self.linear_counter == 0 {
            0
        } else {
            TRIANGLE_TABLE[self.sequence_pos as usize]
        }
    }
}
