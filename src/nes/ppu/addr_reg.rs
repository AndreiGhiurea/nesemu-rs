use crate::nes::cpu::Addr;

pub struct AddressRegister {
    state: AddressState,
    upper: u8,
    lower: u8,
}

enum AddressState {
    WaitingForUpperByte,
    WaitingForLowerByte,
}

impl Default for AddressRegister {
    fn default() -> Self {
        AddressRegister {
            state: AddressState::WaitingForUpperByte,
            upper: 0x0,
            lower: 0x0,
        }
    }
}

impl AddressRegister {
    pub fn write_byte(&mut self, value: u8) {
        // Upper bytes goes first.
        match self.state {
            AddressState::WaitingForUpperByte => {
                self.upper = value;
                self.state = AddressState::WaitingForLowerByte;
            }
            AddressState::WaitingForLowerByte => {
                self.lower = value;
                self.state = AddressState::WaitingForUpperByte;
            }
        }
    }

    pub fn get(&self) -> Addr {
        Addr::from_le_bytes([self.lower, self.upper])
    }

    pub fn increment(&mut self, increment: u8) {
        let mut addr = self.get().wrapping_add(increment as u16);

        // Mirror down address above 0x3FFF
        if addr > 0x3FFF {
            addr &= 0x3FFF;
        }

        (self.lower, self.upper) = addr.to_le_bytes().into();
    }
}
