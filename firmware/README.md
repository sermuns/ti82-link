# TI-82 Bridge

Arduino Nano firmware for communicating with TI-82 calculators via D-BUS protocol.

## Hardware Setup

### Connections

Connect your TI-82's 2.5mm jack to the Arduino Nano:

```
TI-82 2.5mm Jack        Arduino Nano
─────────────────────────────────────
Tip (Red/D0)      ─────  A0 (PC0)
Ring (White/D1)   ─────  A1 (PC1)
Sleeve (GND)      ─────  GND

Pull-up resistors (10kΩ each):
A0 ──[10kΩ]── 5V
A1 ──[10kΩ]── 5V
```

**Note:** The Arduino Nano has internal pull-ups (~20-50kΩ), but external 10kΩ resistors are recommended for reliability.

## Building and Flashing

### Prerequisites
- Rust nightly toolchain
- `ravedude` for flashing
- Arduino Nano (new bootloader)

### Build
```bash
cargo build --release
```

### Flash to Arduino
```bash
cargo run --release
```

The `ravedude` tool will automatically detect your Arduino and flash the firmware.

## Usage

### Serial Protocol

Connect to the Arduino at **57600 baud**. The firmware accepts single-character commands:

#### Commands

- **`R<name>`** - Request a real number variable from the TI-82
  - Example: `RA` requests variable "A"
  - The Arduino will send the REQ packet and receive the variable data
  
- **`S`** - Status check
  - Returns `STATUS: IDLE` or `STATUS: BUSY`

### Example Session

```
$ screen /dev/ttyUSB0 57600

TI-82 Bridge Ready
D0=A0(PC0), D1=A1(PC1)
Commands: REQ:<name>

> RA
Requesting variable: A
Sending REQ...
Waiting for ACK...
Waiting for VAR...
Variable size: 9 bytes
Sending ACK...
Sending CTS...
Waiting for ACK...
Waiting for DATA...
Received 9 bytes:
00 80 12 56 34 78 85 25 55 
Sending final ACK...
Transfer complete!
OK
```

## Protocol Implementation

### D-BUS Protocol

The TI-82 uses a proprietary two-wire bidirectional protocol:

- **Bit 0**: Red line (D0) changes first
- **Bit 1**: White line (D1) changes first
- **Byte format**: LSB first, 8 bits
- **Acknowledgement**: Receiver acknowledges each bit on opposite line

### TI-82 Packet Format

```
[Machine ID] [Command ID] [Length Low] [Length High] [Data...] [Checksum Low] [Checksum High]
```

### Supported Operations

- ✅ Request real number variables (REQ command)
- ✅ Receive variables from calculator
- ❌ Send variables to calculator (TI-82 does not respond to RTS in silent mode)
- ❌ Directory listing (not supported on TI-82)

## Current Limitations

- Only real number variables supported (type 0x00)
- TI-82 has limited silent linking support
- No variable upload (user must manually initiate receive on calculator)
- Timeouts use 1ms polling (may be slow for fast transfers)

## Next Steps

1. **PC CLI Tool**: Build a Rust CLI tool using `clap` and `serialport` crate
2. **Add more variable types**: Lists, programs, matrices
3. **Optimize timing**: Use tighter polling loops for bit reception
4. **Better error handling**: Parse and report TI-82 error codes

## Documentation

Based on the TI Link Protocol Guide from ticalc.org:
- Hardware: D-BUS open-collector protocol
- Packet format: Machine ID + Command + Length + Data + Checksum
- Commands: REQ (0xA2), ACK (0x56), VAR (0x06), CTS (0x09), DATA (0x15)

## License

MIT OR Apache-2.0
