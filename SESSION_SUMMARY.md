# Session Summary - TI-82 Link Project

## Current Status (2026-02-08)

### What Works ✅
- **Real variable silent fetch**: `cargo run -p ti82-cli -- get A` works perfectly
- **Real variable send**: `cargo run -p ti82-cli -- send A 42` works
- Hardware is properly connected (D0 and D1 both HIGH when idle)
- Firmware and CLI are working correctly for real number operations

### What's Broken ❌
- **Program receive**: `cargo run -p ti82-cli -- get-program COSRULE` fails
  - Protocol completes REQ/ACK/VAR/ACK/CTS/ACK sequence
  - Program size detected correctly (297 bytes)
  - **Fails at DATA packet step** - times out waiting for DATA
  - See `ISSUES.md` for detailed analysis

## Latest Session Work

### Attempted: Add Program Receive Functionality
1. Added `get-program` CLI command
2. Added `request_program()` firmware function (separate from working `request_variable()`)
3. Tested - **doesn't work**
4. Debugged output shows protocol mostly working but fails at final DATA step

### Key Learning: Don't Break Working Code
- Initial attempt refactored `request_variable()` into generic function
- This broke the working real variable functionality
- **Solution**: Reverted changes, added `request_program()` as separate function
- Real variable fetch still works after revert

### Files Modified (Committed)
- `firmware/src/main.rs`: Added 'P' command handler + `request_program()` function
- `cli/src/main.rs`: Added `GetProgram` command with hex dump display
- `ISSUES.md`: Created - documents broken program receive with full debug output
- `README.md`: Added strong warning about experimental code quality

## Repository Structure

```
ti82-tools/
├── firmware/           # Arduino Rust (no_std)
│   ├── src/
│   │   ├── main.rs                    # Commands: R, P, S, MR
│   │   ├── dbus/
│   │   │   ├── hardware.rs            # Low-level D-BUS protocol
│   │   │   ├── packet.rs              # Packet send/receive
│   │   │   └── mod.rs
│   │   └── protocol/
│   │       ├── commands.rs            # CMD_VAR, CMD_REQ, etc.
│   │       ├── variables.rs           # VariableHeader struct
│   │       ├── encoding.rs            # BCD encoding/decoding
│   │       └── mod.rs
│   └── Cargo.toml
└── cli/                # PC Rust tool (std)
    ├── src/main.rs     # Commands: status, get, get-program, send
    └── Cargo.toml
```

## Hardware Setup (Working)

**Wiring:**
```
TI-82 Jack      Arduino Nano
────────────────────────────
Tip (D0)    →   D2 + 10kΩ to 5V
Ring (D1)   →   D3 + 10kΩ to 5V  
Sleeve      →   GND
```

**Status Check:**
```bash
$ cargo run -p ti82-cli -- status
STATUS: IDLE (D0=true D1=true)  # Both HIGH = correct
```

## Quick Command Reference

```bash
# Flash firmware
cd firmware && cargo run --release

# Check hardware status
cargo run -p ti82-cli -- status

# Get real variable (silent - no SEND mode needed)
cargo run -p ti82-cli -- get A

# Send real variable (calculator must be in RECEIVE mode)
cargo run -p ti82-cli -- send A 42

# Get program (BROKEN - documented in ISSUES.md)
cargo run -p ti82-cli -- get-program HELLO
```

## Protocol Details

### Working: Real Number Receive (Silent Fetch)
1. PC sends REQ packet with real variable header (type 0x00)
2. Calculator responds with ACK
3. Calculator sends VAR packet with variable info
4. PC sends ACK
5. PC sends CTS (Clear To Send)
6. Calculator sends ACK
7. Calculator sends DATA packet (9 bytes for real numbers)
8. PC sends final ACK
9. **Works perfectly** - fetches silently from home screen

### Broken: Program Receive
1. PC sends REQ packet with program header (type 0x05)
2. Calculator responds with ACK
3. Calculator sends VAR packet with program info ✅
4. PC sends ACK ✅
5. PC sends CTS ✅
6. Calculator sends ACK ✅
7. **Calculator never sends DATA packet** ❌ - timeout after 30s
8. Unknown why - see ISSUES.md for investigation ideas

## Important Files for Next Session

- **ISSUES.md** - Current problem documentation
- **README.md** - Usage instructions + experimental code warning
- **TROUBLESHOOTING.md** - Hardware debugging guide
- **firmware/src/main.rs** - Arduino firmware with R, P, S, MR commands
- **cli/src/main.rs** - PC tool commands

## Git Status

```
Branch: main
Last commit: 1dbeceb "wip: add broken program receive functionality"
Status: Pushed to origin
```

## Next Steps (If Continuing)

To fix program receive:
1. Test with calculator in explicit SEND mode (not silent fetch)
2. Increase DATA timeout from 5000ms to longer
3. Check TI-82 link protocol spec for program vs real differences
4. Capture actual D-BUS traffic to see what calculator does
5. Try different type IDs or header structures for programs

## Important Notes

⚠️ **This is experimental vibe-coded slop**
- Minimal error handling
- Protocol tuned by trial and error
- Works by luck more than design
- DO NOT TRUST for important data

📝 **Commit Style**
- Lowercase, imperative: "fix decimal parsing bug"
- No descriptions, just one line
- Example: `git commit -m "add program receive functionality"`

🔧 **Testing After Changes**
Always verify real variable fetch still works:
```bash
cargo run -p ti82-cli -- get A
```
If this breaks, you've introduced a regression!
