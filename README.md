# TI-82 Link

Bidirectional communication with TI-82 Stats calculators via Arduino Nano.

## ⚠️ WARNING: EXPERIMENTAL CODE ⚠️

**This is vibe-coded experimental slop. DO NOT TRUST for anything important.**

- Developed through trial-and-error without deep protocol understanding
- Minimal error handling and testing
- Protocol implementation tuned by vibes and magic numbers
- Code refactoring has caused regressions
- Works by luck more than design

**Use at your own risk.** See `ISSUES.md` for known problems. This is a hobbyist toy for calculator enthusiasts who understand they're working with experimental, unreliable code.

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

**Send variable to calculator:**
```bash
cargo run -p ti82-cli -- send A 42
cargo run -p ti82-cli -- send B -- -123  # negative numbers
```

Steps:
1. Put calculator in RECEIVE mode: `[2nd][LINK] → RECEIVE`
2. Calculator shows "Waiting..."
3. Run CLI command within 5 seconds
4. Variable appears on calculator

**Get variable from calculator (SILENT - no SEND mode needed):**
```bash
cargo run -p ti82-cli -- get A
```

Steps:
1. Store a value on calculator: `42 → A`
2. Calculator on home screen (not in any menu)
3. Run CLI command - it silently fetches the value!
4. Value displays on PC

**Note:** If sending variable and it exists on calculator, calculator shows overwrite prompt (10 minute timeout).

## What Works

- Sending real numbers to variables A-Z
- Receiving real numbers from variables A-Z
- Values: integers, negatives, zero, decimals
- Handles overwrite dialogs (10 minute timeout)

## License

WTFPL
