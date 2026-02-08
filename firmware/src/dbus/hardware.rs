use arduino_hal::port::{mode, Pin, PinOps};

pub struct DBus<D0, D1>
where
    D0: PinOps,
    D1: PinOps,
{
    d0: Pin<mode::OpenDrain, D0>,
    d1: Pin<mode::OpenDrain, D1>,
}

#[derive(Debug)]
pub enum DBusError {
    Timeout,
    LinkError,
}

impl<D0, D1> DBus<D0, D1>
where
    D0: PinOps,
    D1: PinOps,
{
    pub fn new(d0: Pin<mode::OpenDrain, D0>, d1: Pin<mode::OpenDrain, D1>) -> Self {
        Self { d0, d1 }
    }

    pub fn send_bit(&mut self, bit: bool) -> Result<(), DBusError> {
        if bit {
            self.d1.set_low();
            self.wait_for_d0_low(2000)?;
            self.d1.set_high();
            self.wait_for_d0_high(2000)?;
        } else {
            self.d0.set_low();
            self.wait_for_d1_low(2000)?;
            self.d0.set_high();
            self.wait_for_d1_high(2000)?;
        }
        Ok(())
    }

    pub fn receive_bit(&mut self, timeout_ms: u16) -> Result<bool, DBusError> {
        for _ in 0..timeout_ms {
            let d0_high = self.d0.is_high();
            let d1_high = self.d1.is_high();

            if d1_high && !d0_high {
                self.d1.set_low();
                self.wait_for_d0_high(timeout_ms)?;
                self.d1.set_high();
                return Ok(false);
            }

            if d0_high && !d1_high {
                self.d0.set_low();
                self.wait_for_d1_high(timeout_ms)?;
                self.d0.set_high();
                return Ok(true);
            }

            arduino_hal::delay_ms(1);
        }
        Err(DBusError::Timeout)
    }

    pub fn send_byte(&mut self, byte: u8) -> Result<(), DBusError> {
        for i in 0..8 {
            let bit = (byte >> i) & 1 == 1;
            self.send_bit(bit)?;
        }
        Ok(())
    }

    pub fn receive_byte(&mut self, timeout_ms: u16) -> Result<u8, DBusError> {
        let mut byte = 0u8;
        for i in 0..8 {
            if self.receive_bit(timeout_ms)? {
                byte |= 1 << i;
            }
        }
        Ok(byte)
    }

    pub fn is_idle(&self) -> bool {
        self.d0.is_high() && self.d1.is_high()
    }

    pub fn get_pin_states(&self) -> (bool, bool) {
        (self.d0.is_high(), self.d1.is_high())
    }

    pub fn test_d1_low(&mut self) {
        self.d1.set_low();
    }

    pub fn test_d1_high(&mut self) {
        self.d1.set_high();
    }

    fn wait_for_d0_low(&self, timeout_ms: u16) -> Result<(), DBusError> {
        for _ in 0..timeout_ms {
            if self.d0.is_low() {
                return Ok(());
            }
            arduino_hal::delay_ms(1);
        }
        Err(DBusError::Timeout)
    }

    fn wait_for_d0_high(&self, timeout_ms: u16) -> Result<(), DBusError> {
        for _ in 0..timeout_ms {
            if self.d0.is_high() {
                return Ok(());
            }
            arduino_hal::delay_ms(1);
        }
        Err(DBusError::Timeout)
    }

    fn wait_for_d1_low(&self, timeout_ms: u16) -> Result<(), DBusError> {
        for _ in 0..timeout_ms {
            if self.d1.is_low() {
                return Ok(());
            }
            arduino_hal::delay_ms(1);
        }
        Err(DBusError::Timeout)
    }

    fn wait_for_d1_high(&self, timeout_ms: u16) -> Result<(), DBusError> {
        for _ in 0..timeout_ms {
            if self.d1.is_high() {
                return Ok(());
            }
            arduino_hal::delay_ms(1);
        }
        Err(DBusError::Timeout)
    }
}
