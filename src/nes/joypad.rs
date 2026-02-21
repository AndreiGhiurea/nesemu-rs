use bitflags::bitflags;

bitflags! {
    /// NES controller button flags.
    /// Buttons are read in this order when the controller is polled.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct JoypadButton: u8 {
        const A      = 0b0000_0001;
        const B      = 0b0000_0010;
        const SELECT = 0b0000_0100;
        const START  = 0b0000_1000;
        const UP     = 0b0001_0000;
        const DOWN   = 0b0010_0000;
        const LEFT   = 0b0100_0000;
        const RIGHT  = 0b1000_0000;
    }
}

/// Emulates the NES standard controller.
///
/// Protocol:
/// 1. CPU writes 1 then 0 to $4016 to latch the current button state.
/// 2. Each subsequent read of $4016 returns the next button (bit 0),
///    in order: A, B, Select, Start, Up, Down, Left, Right.
pub struct Joypad {
    strobe: bool,
    button_index: u8,
    button_status: JoypadButton,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            button_index: 0,
            button_status: JoypadButton::empty(),
        }
    }

    pub fn write(&mut self, value: u8) {
        self.strobe = value & 1 == 1;
        if self.strobe {
            self.button_index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        // After 8 reads, always return 1
        if self.button_index > 7 {
            return 1;
        }

        let result = if self.button_status.bits() & (1 << self.button_index) != 0 {
            1
        } else {
            0
        };

        if !self.strobe {
            self.button_index += 1;
        }

        result
    }

    pub fn set_button_pressed(&mut self, button: JoypadButton, pressed: bool) {
        self.button_status.set(button, pressed);
    }
}
