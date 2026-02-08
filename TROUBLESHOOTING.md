# TI-82 Link Troubleshooting Guide

## Current Issue: D1 Pin Stuck LOW

### Diagnostic Output
```bash
$ cargo r -p ti82-cli -- status
STATUS: BUSY (D0=true D1=false)
```

This indicates:
- ✅ D0 (A0, Red wire) is HIGH - Working correctly
- ❌ D1 (A1, White wire) is LOW - **Hardware problem**

### Required Hardware Setup

Both D0 and D1 MUST be HIGH when idle. This requires:

```
TI-82 Link Cable → Arduino Nano
─────────────────────────────────────────
Tip (Red/D0)    → A0 ──[10kΩ resistor]── 5V
Ring (White/D1) → A1 ──[10kΩ resistor]── 5V
Sleeve (GND)    → GND
```

### Why Pull-up Resistors Are Critical

The TI-82 uses an **open-drain protocol**:
- Devices can pull lines LOW (to ground)
- Devices release lines to HIGH-IMPEDANCE (floating)
- **Pull-up resistors** pull floating lines to HIGH (5V)
- Without pull-ups, floating lines read as LOW or random values

### Troubleshooting Steps

#### Step 1: Check Pin States
```bash
cargo r -p ti82-cli -- status
```

**Expected (idle, both HIGH):**
```
STATUS: IDLE (D0=true D1=true)
```

**Problem examples:**
```
STATUS: BUSY (D0=true D1=false)   ← D1 missing pull-up or shorted
STATUS: BUSY (D0=false D1=true)   ← D0 missing pull-up or shorted
STATUS: BUSY (D0=false D1=false)  ← Both missing pull-ups
```

#### Step 2: Verify Pull-up Resistors

**For D1 (A1) which is currently LOW:**

1. **Check resistor is connected:**
   - One leg to Arduino A1 pin
   - Other leg to Arduino 5V pin
   - Value: 10kΩ (color bands: Brown-Black-Orange)

2. **Test resistor with multimeter:**
   - Disconnect from circuit
   - Measure resistance: Should read ~10kΩ
   - If infinite resistance → broken resistor
   - If very low resistance → wrong resistor or shorted

3. **Check connections:**
   - Resistor firmly seated in A1 pin hole
   - Resistor firmly seated in 5V pin hole
   - No loose wires

#### Step 3: Check for Shorts to Ground

1. **Disconnect calculator cable**
2. **Run status check:**
   ```bash
   cargo r -p ti82-cli -- status
   ```
   - If still LOW → Arduino/resistor problem
   - If now HIGH → Cable/calculator problem

3. **Check with multimeter:**
   - Arduino disconnected from USB
   - Measure resistance between A1 and GND
   - Should read: 10kΩ (pull-up resistor)
   - If reads 0Ω → Short to ground

#### Step 4: Test Individual Wires

**With calculator disconnected:**

1. **Add only the pull-up resistors (no calculator wires)**
2. **Check status:**
   ```bash
   cargo r -p ti82-cli -- status
   ```
   Should show: `STATUS: IDLE (D0=true D1=true)`

3. **Connect D0 (red) wire only:**
   - Check status again
   - Should still be IDLE (both HIGH)

4. **Connect D1 (white) wire:**
   - Check status again
   - If now BUSY → Problem with white wire or calculator

5. **Connect GND (sleeve):**
   - Check status
   - Should still be IDLE

#### Step 5: Verify Calculator Connection

With calculator OFF and disconnected:

1. **TI-82 link cable pinout:**
   ```
   Looking at 2.5mm plug (side view):
   
        Tip    ←  D0 (Red)
        Ring   ←  D1 (White)
        Sleeve ←  GND (Black/Shield)
   ```

2. **Use multimeter continuity mode:**
   - Tip to Red wire: Should beep
   - Ring to White wire: Should beep
   - Sleeve to GND: Should beep

### Common Issues and Fixes

#### Issue: "STATUS: BUSY (D0=true D1=false)"
**Fix:** Add or fix 10kΩ pull-up resistor on A1 to 5V

#### Issue: "STATUS: BUSY (D0=false D1=true)"
**Fix:** Add or fix 10kΩ pull-up resistor on A0 to 5V

#### Issue: "STATUS: BUSY (D0=false D1=false)"
**Fix:** Add or fix BOTH 10kΩ pull-up resistors

#### Issue: "STATUS: IDLE but ERROR when requesting variable"
**Possible causes:**
1. Calculator not turned on
2. Calculator in wrong mode (try home screen)
3. Variable doesn't exist
4. Timing issue (calculator too slow to respond)

#### Issue: "Timeout after 30 seconds"
**Fix:** Check if calculator shows "Transmitting" screen
- If YES → Communication working, wait longer or check protocol
- If NO → Calculator not detecting request, check wiring

### Testing Checklist

- [ ] Both pull-up resistors (10kΩ) installed
- [ ] A0 → 10kΩ → 5V
- [ ] A1 → 10kΩ → 5V
- [ ] Status shows IDLE with both pins HIGH
- [ ] Red wire to A0
- [ ] White wire to A1
- [ ] GND wire to GND
- [ ] Calculator turned on
- [ ] Variable stored (e.g., 42 → A)
- [ ] Calculator on home screen (not in menu)

### Successful Test

Once hardware is fixed:

```bash
# 1. Check status
$ cargo r -p ti82-cli -- status
STATUS: IDLE (D0=true D1=true)

# 2. Store value on calculator
TI-82: 42 → A

# 3. Request variable
$ cargo r -p ti82-cli -- -v get A
Waiting for Arduino to boot...
Sending command...
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
```

### Need More Help?

If D1 stays LOW after checking all above:
1. Try a different Arduino pin (e.g., A2 instead of A1)
2. Try a different pull-up resistor
3. Check Arduino A1 pin isn't damaged
4. Verify 5V pin is actually 5V (use multimeter)
