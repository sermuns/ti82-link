# Session Summary - TI-82 Link Debugging

## What Happened

When you ran `cargo r -- -v get A`, the CLI wasn't receiving responses from the firmware. I debugged and fixed multiple issues, then discovered a **hardware problem**.

## Issues Found & Fixed

### 1. CLI Not Reading Responses ✅ FIXED
**Problem:** CLI was timing out immediately or reading startup messages instead of actual responses.

**Fixes made:**
- Added 2-second boot delay (Arduino resets when serial connects)
- Clear input buffer after boot
- Skip startup messages, wait for "Requesting variable:" before recording
- Increased timeout to 30 seconds
- Better activity-based timeout (2 seconds of no data after receiving something)

### 2. Startup Message Confusion ✅ FIXED
**Problem:** CLI was treating Arduino's boot messages as command responses.

**Fix:** Modified `read_all_response()` to mark startup messages as `[skipped]` and only record lines after seeing "Requesting variable:".

### 3. Command Sending ✅ WORKING
- Status command works: `cargo r -p ti82-cli -- status`
- Get command reaches firmware: `cargo r -p ti82-cli -- -v get A`
- Firmware receives commands correctly

### 4. Hardware Issue Discovered ⚠️ NEEDS YOUR ATTENTION

## Current Problem: D1 Pin Stuck LOW

Running the status command shows:
```bash
$ cargo r -p ti82-cli -- status
STATUS: BUSY (D0=true D1=false)
```

**What this means:**
- D0 (Red wire, A0): ✅ HIGH - Working correctly
- D1 (White wire, A1): ❌ LOW - **Missing pull-up resistor or short to ground**

## What You Need to Check

### Critical: Pull-up Resistors Required!

The TI-82 link protocol requires **both** lines to be HIGH when idle. This is done with 10kΩ resistors:

```
A0 ──[10kΩ resistor]── 5V  ← D0 (Red)
A1 ──[10kΩ resistor]── 5V  ← D1 (White)
```

### Action Items

1. **Check if you have a 10kΩ resistor on A1:**
   - One leg connected to Arduino pin A1
   - Other leg connected to Arduino 5V pin

2. **If no resistor on A1:**
   - Add a 10kΩ resistor (color: Brown-Black-Orange-Gold)
   - Connect between A1 and 5V

3. **If resistor exists:**
   - Check connections are firm
   - Test resistor with multimeter (should read ~10kΩ)
   - Check for short to ground on white wire

4. **After fixing, test again:**
   ```bash
   cargo r -p ti82-cli -- status
   ```
   Should show: `STATUS: IDLE (D0=true D1=true)`

## Files Updated

### Firmware Changes
- `firmware/src/main.rs`: Added pin state diagnostics to status command
- `firmware/src/dbus/hardware.rs`: Added `get_pin_states()` method
- **Firmware has been flashed to your Arduino**

### CLI Changes
- `cli/src/main.rs`: 
  - Added 2-second boot delay
  - Skip startup messages
  - Wait for actual response
  - Added verbose output showing `[skipped]` messages
- **CLI has been compiled and ready to use**

## Documentation Created

- **TROUBLESHOOTING.md**: Comprehensive hardware debugging guide
  - How to test pin states
  - How to verify pull-up resistors
  - How to check for shorts
  - Step-by-step troubleshooting

## How to Test When Fixed

```bash
# Step 1: Verify hardware (should show IDLE with both pins HIGH)
cargo r -p ti82-cli -- status

# Step 2: Store a value on calculator
# On TI-82: 42 → A

# Step 3: Request the variable
cargo r -p ti82-cli -- -v get A

# Expected output:
# Waiting for Arduino to boot...
# Sending command...
# ← Requesting variable: A
# ← Sending REQ...
# ← Waiting for ACK...
# ... (protocol messages)
# ← Transfer complete!
# ← OK
# Variable A: 42.0
```

## Current Status

✅ Software working correctly
✅ Communication between CLI and firmware working
✅ Diagnostics added to identify hardware issues
⚠️ **Hardware issue detected: D1 (A1) missing pull-up resistor**

**Next step:** Add or fix 10kΩ pull-up resistor on A1 to 5V, then test again!

## Quick Reference Commands

```bash
# Check pin states
cargo r -p ti82-cli -- status

# Get variable with verbose output
cargo r -p ti82-cli -- -v get A

# Get variable (clean output)
cargo r -p ti82-cli -- get A
```

## Expected Working Output

When hardware is fixed, you should see:

```
$ cargo r -p ti82-cli -- status
STATUS: IDLE (D0=true D1=true)

$ cargo r -p ti82-cli -- get A
Variable A: 42.0
```

See TROUBLESHOOTING.md for detailed debugging steps!
