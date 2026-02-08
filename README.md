# TI-82 Link Tools

A Rust-based toolset for communicating with TI-82 calculators via the 2.5mm link port.

## Project Structure

```
ti82-tools/
├── firmware/     - Arduino Nano firmware (Rust/AVR, no_std)
└── cli/          - PC command-line tool (Rust/std)
```

Both are workspace members sharing version, edition, and license configuration.

## Hardware Setup

### Required Components
- Arduino Nano (ATmega328p with new bootloader)
- TI-82 calculator with 2.5mm link cable
- 2x 10kΩ pull-up resistors

### Wiring
```
TI-82 2.5mm Jack → Arduino Nano
──────────────────────────────────
Tip (Red/D0)    → A0 ──[10kΩ]── 5V
Ring (White/D1) → A1 ──[10kΩ]── 5V  
Sleeve (GND)    → GND
```

## Building & Flashing Firmware

The firmware requires AVR target. Build from the firmware directory:

```bash
cd firmware
cargo build --release  # Uses AVR target from .cargo/config.toml
cargo run --release    # Builds and flashes to Arduino
```

Or from workspace root:
```bash
cargo build -p ti82-bridge --release --target avr-none
```

The firmware uses:
- **OpenDrain mode** for proper D-BUS protocol implementation
- **No unsafe code** - uses arduino-hal abstractions
- **57600 baud** serial communication
- **Nightly Rust** (workspace-level rust-toolchain.toml)

## Building & Using the CLI

```bash
# From workspace root
cargo build -p ti82-cli --release
cargo run -p ti82-cli -- --help

# Check bridge status
cargo run -p ti82-cli -- status

# Get variable A from calculator (displays parsed value)
cargo run -p ti82-cli -- get A

# Verbose mode - see all protocol messages and raw bytes
cargo run -p ti82-cli -- -v get A

# Custom port
cargo run -p ti82-cli -- -p /dev/ttyACM0 get B
```

### CLI Features

- **Auto-parsing**: Decodes TI-82 9-byte floating point format to readable numbers
- **Verbose mode** (`-v`): Shows all protocol messages and raw hex bytes
- **Smart timeout**: 30-second max wait, exits after 2 seconds of inactivity
- **Clean output**: Just shows "Variable A: 3.14159" unless verbose

Example output:
```bash
$ cargo run -p ti82-cli -- get A
Variable A: 42.0

$ cargo run -p ti82-cli -- -v get A
← Requesting variable: A
← Sending REQ...
← Waiting for ACK...
← Waiting for VAR...
← Variable size: 9 bytes
← Sending ACK...
← Sending CTS...
← Waiting for ACK...
← Waiting for DATA...
← Received 9 bytes:
← 00 82 42 00 00 00 00 00 00
← Sending final ACK...
← Transfer complete!
← OK
Variable A: 42.0
Raw bytes: 00 82 42 00 00 00 00 00 00
```

## Serial Output Issue

If you don't see serial output when transferring variables:

1. **Arduino resets on serial connection** - Output sent before your terminal connects is lost
2. **Solution**: Use the CLI tool which stays connected and waits for all output
3. **Alternative**: Use `screen` or `minicom` and keep it open continuously

Example with screen:
```bash
screen /dev/ttyUSB0 57600
# Keep this open, then trigger transfer from calculator
```

## Protocol

The firmware implements the TI-82 D-BUS protocol:
- 2-wire bidirectional communication
- LSB-first byte transmission  
- 16-bit checksums
- 8-step REQ/ACK/VAR/CTS/DATA exchange sequence

Currently supports:
- ✅ Real number variables (A-Z)
- ⏳ Lists, matrices, programs (future)

## License

WTFPL
