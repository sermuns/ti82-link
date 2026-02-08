use super::hardware::{DBus, DBusError};
use arduino_hal::port::PinOps;
use heapless::Vec;

pub struct Packet {
    pub machine_id: u8,
    pub command_id: u8,
    pub data: Vec<u8, 256>,
}

impl Packet {
    pub fn new(machine_id: u8, command_id: u8) -> Self {
        Self {
            machine_id,
            command_id,
            data: Vec::new(),
        }
    }

    pub fn with_data(machine_id: u8, command_id: u8, data: &[u8]) -> Self {
        let mut vec = Vec::new();
        vec.extend_from_slice(data).ok();
        Self {
            machine_id,
            command_id,
            data: vec,
        }
    }

    pub fn send<D0, D1>(&self, dbus: &mut DBus<D0, D1>) -> Result<(), DBusError>
    where
        D0: PinOps,
        D1: PinOps,
    {
        dbus.send_byte(self.machine_id)?;
        dbus.send_byte(self.command_id)?;

        let len = self.data.len() as u16;
        dbus.send_byte((len & 0xFF) as u8)?;
        dbus.send_byte((len >> 8) as u8)?;

        // Check if this is a command that has no data
        let has_data = match self.command_id {
            0x56 => false, // ACK
            0x09 => false, // CTS (Clear To Send)
            0x92 => false, // EOT
            0x5A => false, // ERR
            _ => !self.data.is_empty(),
        };

        if has_data {
            for &byte in self.data.iter() {
                dbus.send_byte(byte)?;
            }

            let checksum = Self::calculate_checksum(&self.data);
            dbus.send_byte((checksum & 0xFF) as u8)?;
            dbus.send_byte((checksum >> 8) as u8)?;
        }

        Ok(())
    }

    pub fn receive<D0, D1>(dbus: &mut DBus<D0, D1>, timeout_ms: u32) -> Result<Self, DBusError>
    where
        D0: PinOps,
        D1: PinOps,
    {
        let machine_id = dbus.receive_byte(timeout_ms)?;
        let command_id = dbus.receive_byte(timeout_ms)?;

        let len_low = dbus.receive_byte(timeout_ms)?;
        let len_high = dbus.receive_byte(timeout_ms)?;
        let len = (len_high as u16) << 8 | (len_low as u16);

        // Check if this is a command that has no data
        // ACK (0x56), CTS (0x09), EOT (0x92), and some error commands have no data
        let has_data = match command_id {
            0x56 => false, // ACK
            0x09 => false, // CTS (Clear To Send)
            0x92 => false, // EOT
            0x5A => false, // ERR
            _ => len > 0,
        };

        let mut data = Vec::new();
        if has_data {
            for _ in 0..len {
                let byte = dbus.receive_byte(timeout_ms)?;
                data.push(byte).ok();
            }

            let checksum_low = dbus.receive_byte(timeout_ms)?;
            let checksum_high = dbus.receive_byte(timeout_ms)?;
            let received_checksum = (checksum_high as u16) << 8 | (checksum_low as u16);

            let calculated_checksum = Self::calculate_checksum(&data);
            if received_checksum != calculated_checksum {
                return Err(DBusError::LinkError);
            }
        }

        Ok(Self {
            machine_id,
            command_id,
            data,
        })
    }

    fn calculate_checksum(data: &[u8]) -> u16 {
        data.iter()
            .fold(0u16, |acc, &byte| acc.wrapping_add(byte as u16))
    }
}
