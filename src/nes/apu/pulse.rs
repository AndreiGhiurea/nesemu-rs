/// NES APU Pulse Wave Channel
///
/// Used for both Pulse 1 and Pulse 2. Produces a square wave with
/// selectable duty cycle (12.5%, 25%, 50%, 75%).

/// Duty cycle lookup table — each entry is an 8-step waveform
const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [0, 0, 0, 0, 0, 0, 1, 1], // 25%
    [0, 0, 0, 0, 1, 1, 1, 1], // 50%
    [1, 1, 1, 1, 1, 1, 0, 0], // 75% (inverted 25%)
];

/// Length counter lookup table (shared across channels)
pub const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct PulseChannel {
    enabled: bool,

    // Duty
    duty: u8,
    duty_pos: u8,

    // Timer
    timer_period: u16,
    timer_value: u16,

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

    // Sweep unit
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_divider: u8,
    sweep_reload: bool,
    channel_id: u8, // 1 or 2, affects sweep negate behavior
}

impl PulseChannel {
    pub fn new(channel_id: u8) -> Self {
        PulseChannel {
            enabled: false,
            duty: 0,
            duty_pos: 0,
            timer_period: 0,
            timer_value: 0,
            length_counter: 0,
            length_halt: false,
            envelope_start: false,
            envelope_loop: false,
            constant_volume: false,
            envelope_period: 0,
            envelope_divider: 0,
            envelope_decay: 0,
            volume: 0,
            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_divider: 0,
            sweep_reload: false,
            channel_id,
        }
    }

    /// $4000/$4004 — Duty, length counter halt, constant volume, volume/envelope
    pub fn write_control(&mut self, value: u8) {
        self.duty = (value >> 6) & 0x03;
        self.length_halt = value & 0x20 != 0;
        self.envelope_loop = value & 0x20 != 0;
        self.constant_volume = value & 0x10 != 0;
        self.volume = value & 0x0F;
        self.envelope_period = value & 0x0F;
    }

    /// $4001/$4005 — Sweep unit
    pub fn write_sweep(&mut self, value: u8) {
        self.sweep_enabled = value & 0x80 != 0;
        self.sweep_period = (value >> 4) & 0x07;
        self.sweep_negate = value & 0x08 != 0;
        self.sweep_shift = value & 0x07;
        self.sweep_reload = true;
    }

    /// $4002/$4006 — Timer low 8 bits
    pub fn write_timer_lo(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x0700) | value as u16;
    }

    /// $4003/$4007 — Length counter load, timer high 3 bits
    pub fn write_timer_hi(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | ((value as u16 & 0x07) << 8);
        if self.enabled {
            self.length_counter = LENGTH_TABLE[(value >> 3) as usize];
        }
        self.duty_pos = 0;
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

    /// Called every APU cycle (every other CPU cycle)
    pub fn tick_timer(&mut self) {
        if self.timer_value == 0 {
            self.timer_value = self.timer_period;
            self.duty_pos = (self.duty_pos + 1) % 8;
        } else {
            self.timer_value -= 1;
        }
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

    /// Half frame: clock the sweep unit
    pub fn tick_sweep(&mut self) {
        let target = self.sweep_target_period();
        if self.sweep_divider == 0 && self.sweep_enabled && !self.is_sweep_muting() {
            self.timer_period = target;
        }

        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = self.sweep_period;
            self.sweep_reload = false;
        } else {
            self.sweep_divider -= 1;
        }
    }

    fn sweep_target_period(&self) -> u16 {
        let shift = self.timer_period >> self.sweep_shift;
        if self.sweep_negate {
            if self.channel_id == 1 {
                // Pulse 1: one's complement (subtract shift, then subtract 1)
                self.timer_period.wrapping_sub(shift).wrapping_sub(1)
            } else {
                // Pulse 2: two's complement (just subtract shift)
                self.timer_period.wrapping_sub(shift)
            }
        } else {
            self.timer_period.wrapping_add(shift)
        }
    }

    fn is_sweep_muting(&self) -> bool {
        self.timer_period < 8 || self.sweep_target_period() > 0x7FF
    }

    pub fn output(&self) -> u8 {
        if !self.enabled
            || self.length_counter == 0
            || DUTY_TABLE[self.duty as usize][self.duty_pos as usize] == 0
            || self.is_sweep_muting()
        {
            0
        } else if self.constant_volume {
            self.volume
        } else {
            self.envelope_decay
        }
    }
}
