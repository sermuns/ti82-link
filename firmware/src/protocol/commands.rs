pub const MACHINE_ID_COMPUTER_TI82: u8 = 0x02;
pub const MACHINE_ID_COMPUTER_TI83: u8 = 0x03;
pub const MACHINE_ID_COMPUTER: u8 = MACHINE_ID_COMPUTER_TI83; // Use TI-83 for TI-82 Stats
pub const MACHINE_ID_TI82: u8 = 0x82;

pub const CMD_VAR: u8 = 0x06;
pub const CMD_CTS: u8 = 0x09;
pub const CMD_DATA: u8 = 0x15;
pub const CMD_SKIP_EXIT: u8 = 0x36;
pub const CMD_ACK: u8 = 0x56;
pub const CMD_ERR: u8 = 0x5A;
pub const CMD_SCR: u8 = 0x6D;
pub const CMD_EOT: u8 = 0x92;
pub const CMD_REQ: u8 = 0xA2;
pub const CMD_RTS: u8 = 0xC9;

pub const EXIT_CODE_EXIT: u8 = 0x01;
pub const EXIT_CODE_SKIP: u8 = 0x02;
pub const EXIT_CODE_OUT_OF_MEM: u8 = 0x03;
