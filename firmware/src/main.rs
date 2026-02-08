#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;

mod dbus;
mod protocol;

use arduino_hal::port::PinOps;
use dbus::{DBus, Packet};
use protocol::*;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Initialize USB serial at 57600 baud
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let d0 = pins.d2.into_opendrain_high();
    let d1 = pins.d3.into_opendrain_high();
    let mut dbus = DBus::new(d0, d1);

    ufmt::uwriteln!(&mut serial, "TI-82 Bridge Ready\r").unwrap_infallible();
    ufmt::uwriteln!(&mut serial, "D0=D2, D1=D3\r").unwrap_infallible();
    ufmt::uwriteln!(&mut serial, "Commands: M<type><name> or S or R<name>\r").unwrap_infallible();

    let mut led = pins.d13.into_output();

    loop {
        led.toggle();
        arduino_hal::delay_ms(500);

        let byte = nb::block!(serial.read()).unwrap_infallible();

        match byte {
            b'R' => {
                // Request variable (receive FROM calculator)
                let name_char = nb::block!(serial.read()).unwrap_infallible();

                ufmt::uwriteln!(&mut serial, "Requesting variable: {}\r", name_char as char)
                    .unwrap_infallible();

                if request_variable(&mut dbus, &mut serial, name_char).is_ok() {
                    ufmt::uwriteln!(&mut serial, "OK\r").unwrap_infallible();
                } else {
                    ufmt::uwriteln!(&mut serial, "ERROR\r").unwrap_infallible();
                }
            }
            b'S' => {
                // Status check
                let (d0_state, d1_state) = dbus.get_pin_states();
                if dbus.is_idle() {
                    ufmt::uwriteln!(
                        &mut serial,
                        "STATUS: IDLE (D0={} D1={})\r",
                        d0_state,
                        d1_state
                    )
                    .unwrap_infallible();
                } else {
                    ufmt::uwriteln!(
                        &mut serial,
                        "STATUS: BUSY (D0={} D1={})\r",
                        d0_state,
                        d1_state
                    )
                    .unwrap_infallible();
                }
            }
            b'M' => {
                // Manual send variable (send TO calculator)
                // Format: M<type><name/data>
                // Types: R=real, L=list, S=string, P=program
                let type_char = nb::block!(serial.read()).unwrap_infallible();

                match type_char {
                    b'R' => {
                        // Real: MR<name><4 bytes i32 little-endian>
                        let name_char = nb::block!(serial.read()).unwrap_infallible();

                        // Read 4 bytes for i32 value (little-endian)
                        let mut value_bytes = [0u8; 4];
                        for i in 0..4 {
                            value_bytes[i] = nb::block!(serial.read()).unwrap_infallible();
                        }
                        let value = i32::from_le_bytes(value_bytes);

                        ufmt::uwriteln!(
                            &mut serial,
                            "Sending real {}={}\r",
                            name_char as char,
                            value
                        )
                        .unwrap_infallible();

                        if send_real(&mut dbus, &mut serial, name_char, value).is_ok() {
                            ufmt::uwriteln!(&mut serial, "OK\r").unwrap_infallible();
                        } else {
                            ufmt::uwriteln!(&mut serial, "ERROR\r").unwrap_infallible();
                        }
                    }
                    _ => {
                        ufmt::uwriteln!(&mut serial, "Unknown type: {}\r", type_char as char)
                            .unwrap_infallible();
                    }
                }
            }
            _ => {
                ufmt::uwriteln!(&mut serial, "Unknown command\r").unwrap_infallible();
            }
        }
    }
}

fn request_variable<W, D0, D1>(
    dbus: &mut DBus<D0, D1>,
    serial: &mut W,
    name_char: u8,
) -> Result<(), dbus::hardware::DBusError>
where
    W: ufmt::uWrite,
    D0: PinOps,
    D1: PinOps,
{
    let var_header = VariableHeader::new_real(name_char, 0);
    let req_packet = Packet::with_data(MACHINE_ID_COMPUTER, CMD_REQ, &var_header.to_bytes());

    ufmt::uwriteln!(serial, "Sending REQ...\r").ok();
    req_packet.send(dbus)?;

    ufmt::uwriteln!(serial, "Waiting for ACK...\r").ok();
    let ack1 = Packet::receive(dbus, 5000)?;
    if ack1.command_id != CMD_ACK {
        ufmt::uwriteln!(serial, "Expected ACK, got: {:02X}\r", ack1.command_id).ok();
        return Err(dbus::hardware::DBusError::LinkError);
    }

    ufmt::uwriteln!(serial, "Waiting for VAR...\r").ok();
    let var_packet = Packet::receive(dbus, 5000)?;
    if var_packet.command_id != CMD_VAR {
        ufmt::uwriteln!(serial, "Expected VAR, got: {:02X}\r", var_packet.command_id).ok();
        return Err(dbus::hardware::DBusError::LinkError);
    }

    let Some(actual_header) = VariableHeader::from_bytes(&var_packet.data) else {
        return Err(dbus::hardware::DBusError::LinkError);
    };
    ufmt::uwriteln!(serial, "Variable size: {} bytes\r", actual_header.data_size).ok();

    ufmt::uwriteln!(serial, "Sending ACK...\r").ok();
    let ack_packet = Packet::new(MACHINE_ID_COMPUTER, CMD_ACK);
    ack_packet.send(dbus)?;

    ufmt::uwriteln!(serial, "Sending CTS...\r").ok();
    let cts_packet = Packet::new(MACHINE_ID_COMPUTER, CMD_CTS);
    cts_packet.send(dbus)?;

    ufmt::uwriteln!(serial, "Waiting for ACK...\r").ok();
    let ack2 = Packet::receive(dbus, 5000)?;
    if ack2.command_id != CMD_ACK {
        ufmt::uwriteln!(serial, "Expected ACK, got: {:02X}\r", ack2.command_id).ok();
        return Err(dbus::hardware::DBusError::LinkError);
    }

    ufmt::uwriteln!(serial, "Waiting for DATA...\r").ok();
    let data_packet = Packet::receive(dbus, 5000)?;
    if data_packet.command_id != CMD_DATA {
        ufmt::uwriteln!(
            serial,
            "Expected DATA, got: {:02X}\r",
            data_packet.command_id
        )
        .ok();
        return Err(dbus::hardware::DBusError::LinkError);
    }

    ufmt::uwriteln!(serial, "Received {} bytes:\r", data_packet.data.len()).ok();
    for (i, &byte) in data_packet.data.iter().enumerate() {
        if i % 16 == 0 {
            ufmt::uwrite!(serial, "\r").ok();
        }
        ufmt::uwrite!(serial, "{:02X} ", byte).ok();
    }
    ufmt::uwriteln!(serial, "\r").ok();

    ufmt::uwriteln!(serial, "Sending final ACK...\r").ok();
    let ack_final = Packet::new(MACHINE_ID_COMPUTER, CMD_ACK);
    ack_final.send(dbus)?;

    ufmt::uwriteln!(serial, "Transfer complete!\r").ok();

    Ok(())
}

fn send_real<W, D0, D1>(
    dbus: &mut DBus<D0, D1>,
    serial: &mut W,
    name_char: u8,
    value: i32,
) -> Result<(), dbus::hardware::DBusError>
where
    W: ufmt::uWrite,
    D0: PinOps,
    D1: PinOps,
{
    // Encode the value using integer encoding (no floats!)
    let real_data = encode_integer(value);

    // Step 1: Send VAR packet
    {
        let var_header = VariableHeader::new_real(name_char, 9);
        let header_bytes = var_header.to_bytes();
        let var_packet = Packet::with_data(MACHINE_ID_COMPUTER, CMD_VAR, &header_bytes);
        ufmt::uwriteln!(serial, "S1: Sending VAR\r").ok();
        var_packet.send(dbus)?;
    }

    // Step 2: Wait for ACK
    {
        ufmt::uwriteln!(serial, "S2: Wait ACK\r").ok();
        let ack1 = Packet::receive(dbus, 10000)?;
        if ack1.command_id != CMD_ACK {
            ufmt::uwriteln!(serial, "ERR: expected ACK (0x56)\r").ok();
            return Err(dbus::hardware::DBusError::LinkError);
        }
    }

    // Step 3: Wait for CTS (long timeout for overwrite dialog)
    {
        ufmt::uwriteln!(serial, "S3: Wait CTS\r").ok();
        let cts = Packet::receive(dbus, 65535)?;
        if cts.command_id != CMD_CTS {
            ufmt::uwriteln!(serial, "ERR: got {:02X}\r", cts.command_id).ok();
            return Err(dbus::hardware::DBusError::LinkError);
        }
    }
    arduino_hal::delay_ms(50);

    // Step 4: Send ACK for CTS
    {
        ufmt::uwriteln!(serial, "S4: Send ACK\r").ok();
        let ack2 = Packet::new(MACHINE_ID_COMPUTER, CMD_ACK);
        ack2.send(dbus)?;
    }
    arduino_hal::delay_ms(50);

    // Step 5: Send DATA packet
    {
        ufmt::uwriteln!(serial, "S5: Send DATA\r").ok();
        let data_packet = Packet::with_data(MACHINE_ID_COMPUTER, CMD_DATA, &real_data);
        data_packet.send(dbus)?;
    }

    // Step 6: Wait for final ACK
    {
        ufmt::uwriteln!(serial, "S6: Wait ACK\r").ok();
        let ack3 = Packet::receive(dbus, 5000)?;
        if ack3.command_id != CMD_ACK {
            ufmt::uwriteln!(serial, "ERR: got {:02X}\r", ack3.command_id).ok();
            return Err(dbus::hardware::DBusError::LinkError);
        }
    }
    arduino_hal::delay_ms(50);

    // Step 7: Send EOT
    {
        ufmt::uwriteln!(serial, "S7: Send EOT\r").ok();
        let eot = Packet::new(MACHINE_ID_COMPUTER, CMD_EOT);
        eot.send(dbus)?;
    }

    // Step 8: Wait for EOT ACK
    {
        ufmt::uwriteln!(serial, "S8: Wait ACK\r").ok();
        let ack_eot = Packet::receive(dbus, 5000)?;
        if ack_eot.command_id != CMD_ACK {
            ufmt::uwriteln!(serial, "ERR: got {:02X}\r", ack_eot.command_id).ok();
            return Err(dbus::hardware::DBusError::LinkError);
        }
    }

    ufmt::uwriteln!(serial, "Transfer complete!\r").ok();

    Ok(())
}
