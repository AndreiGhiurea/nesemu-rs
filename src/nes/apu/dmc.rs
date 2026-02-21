/// NES APU Delta Modulation Channel (DMC)
///
/// Plays 1-bit delta-encoded samples from memory (DPCM).
/// Has a 7-bit output level counter and a memory reader that
/// fetches sample bytes from the cartridge.

/// Rate lookup table (CPU cycles per sample bit)
const DMC_RATE_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214,
    190, 160, 142, 128, 106, 84, 72, 54,
];

pub struct DmcChannel {
    enabled: bool,

    // Timer
    rate: u16,
    timer_value: u16,

    // Output
    output_level: u8,

    // Sample
    sample_address: u16,
    sample_length: u16,
    current_address: u16,
    bytes_remaining: u16,

    // Shift register
    sample_buffer: Option<u8>,
    shift_register: u8,
    bits_remaining: u8,
    silence: bool,

    // Flags
    irq_enabled: bool,
    loop_flag: bool,
    pub interrupt_flag: bool,
}

impl DmcChannel {
    pub fn new() -> Self {
        DmcChannel {
            enabled: false,
            rate: DMC_RATE_TABLE[0],
            timer_value: 0,
            output_level: 0,
            sample_address: 0xC000,
            sample_length: 1,
            current_address: 0xC000,
            bytes_remaining: 0,
            sample_buffer: None,
            shift_register: 0,
            bits_remaining: 0,
            silence: true,
            irq_enabled: false,
            loop_flag: false,
            interrupt_flag: false,
        }
    }

    /// $4010 — Flags and rate
    pub fn write_control(&mut self, value: u8) {
        self.irq_enabled = value & 0x80 != 0;
        self.loop_flag = value & 0x40 != 0;
        self.rate = DMC_RATE_TABLE[(value & 0x0F) as usize];

        if !self.irq_enabled {
            self.interrupt_flag = false;
        }
    }

    /// $4011 — Direct load (7-bit output level)
    pub fn write_direct_load(&mut self, value: u8) {
        self.output_level = value & 0x7F;
    }

    /// $4012 — Sample address (address = $C000 + value * 64)
    pub fn write_sample_address(&mut self, value: u8) {
        self.sample_address = 0xC000 + (value as u16) * 64;
    }

    /// $4013 — Sample length (length = value * 16 + 1)
    pub fn write_sample_length(&mut self, value: u8) {
        self.sample_length = (value as u16) * 16 + 1;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.bytes_remaining = 0;
        } else if self.bytes_remaining == 0 {
            self.restart();
        }
        self.interrupt_flag = false;
    }

    pub fn bytes_remaining(&self) -> u16 {
        self.bytes_remaining
    }

    fn restart(&mut self) {
        self.current_address = self.sample_address;
        self.bytes_remaining = self.sample_length;
    }

    /// Called every CPU cycle. Returns Some(address) if a memory read is needed.
    pub fn tick_timer(&mut self) -> Option<u16> {
        let mut memory_read_addr = None;

        if self.timer_value == 0 {
            self.timer_value = self.rate;

            // Clock the output unit
            if !self.silence {
                if self.shift_register & 1 != 0 {
                    if self.output_level <= 125 {
                        self.output_level += 2;
                    }
                } else if self.output_level >= 2 {
                    self.output_level -= 2;
                }
                self.shift_register >>= 1;
            }

            self.bits_remaining = self.bits_remaining.saturating_sub(1);

            if self.bits_remaining == 0 {
                self.bits_remaining = 8;

                if let Some(buffer) = self.sample_buffer.take() {
                    self.silence = false;
                    self.shift_register = buffer;
                } else {
                    self.silence = true;
                }
            }

            // Fill sample buffer if empty
            if self.sample_buffer.is_none() && self.bytes_remaining > 0 {
                memory_read_addr = Some(self.current_address);
                // Address wraps around after $FFFF
                self.current_address = self.current_address.wrapping_add(1) | 0x8000;
                self.bytes_remaining -= 1;

                if self.bytes_remaining == 0 {
                    if self.loop_flag {
                        self.restart();
                    } else if self.irq_enabled {
                        self.interrupt_flag = true;
                    }
                }
            }
        } else {
            self.timer_value -= 1;
        }

        memory_read_addr
    }

    pub fn fill_sample_buffer(&mut self, value: u8) {
        self.sample_buffer = Some(value);
    }

    pub fn output(&self) -> u8 {
        self.output_level
    }
}
