# TI-82 Link

Send variables to TI-82 Stats calculators via Arduino Nano.

## Hardware

**Components:**
- Arduino Nano (ATmega328p)
- 2x 10kΩ resistors
- TI-82 Stats with 2.5mm link cable

**Wiring:**
```
TI-82 Jack      Arduino Nano
────────────────────────────
Tip (D0)    →   D2 + 10kΩ to 5V
Ring (D1)   →   D3 + 10kΩ to 5V
Sleeve      →   GND
```

## Setup

**Flash Arduino:**
```bash
cd firmware
cargo run --release
```

## Usage

**Send variable:**
```bash
cargo run -p ti82-cli -- send A 42
cargo run -p ti82-cli -- send B -- -123  # negative numbers
```

**Steps:**
1. Put calculator in RECEIVE mode: `[2nd][LINK] → RECEIVE`
2. Calculator shows "Waiting..."
3. Run CLI command within 5 seconds
4. Variable appears on calculator

**Note:** If variable exists, calculator shows overwrite prompt (65 second timeout).

## What Works

- Sending real numbers to variables A-Z
- Values: integers, negatives, zero
- Handles overwrite dialogs (60s timeout)

## License

WTFPL
