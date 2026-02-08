# Known Issues

## Program Receive Not Working (get-program command)

**Status:** Broken  
**Date:** 2026-02-08

### Symptoms
```bash
$ cargo r -- -v get-program COSRULE
← Requesting program
← Sending REQ...
← Waiting for ACK...
← Waiting for VAR...
← Program size: 297  # Correct size detected
← Sending ACK...
← Sending CTS...
← Waiting for ACK...
← Waiting for DATA...
← ERROR
Timeout after 30 seconds
Program 'COSRULE': No data received
```

### Analysis
- The REQ/ACK/VAR/ACK/CTS/ACK sequence completes successfully
- Program size is correctly detected (297 bytes)
- Communication fails at the DATA packet step
- Calculator may be rejecting the request or protocol difference for programs vs real variables

### What Works
- ✅ Real variable receive (silent fetch): `cargo r -- get A` works perfectly
- ✅ Real variable send: `cargo r -- send A 42` works
- ❌ Program receive: Fails at DATA packet

### Possible Causes
1. **Type ID mismatch**: Program type ID (0x05) may need special handling
2. **Silent fetch limitation**: Programs might require explicit SEND mode on calculator
3. **Protocol difference**: Programs may use different packet structure than real variables
4. **Timeout issue**: DATA packet for programs may take longer than 5000ms
5. **CTS behavior**: Calculator may not respond to CTS the same way for programs

### Debug Output Analysis
The corrupted output `297OSRULE` suggests the program name is bleeding into the size field display, indicating a potential parsing or buffer issue in the firmware's serial output.

### Investigation Needed
- [ ] Test with calculator in explicit SEND mode (TRANSMIT → Program)
- [ ] Increase DATA packet timeout for programs
- [ ] Compare TI-82 link protocol spec for real vs program variables
- [ ] Capture actual D-BUS traffic to see what calculator sends
- [ ] Check if program type needs different VAR header structure

### Workaround
None currently. Program receive functionality is non-functional.

### Related Code
- `firmware/src/main.rs`: `request_program()` function (lines ~216-295)
- `cli/src/main.rs`: `GetProgram` command handler (lines ~209-257)

---

## Disclaimer

This codebase is **experimental vibe-coded slop**. It was developed through iterative experimentation without deep understanding of the TI-82 link protocol. The real variable functionality works by luck more than design.

**DO NOT TRUST THIS CODE FOR:**
- Production use
- Critical data transfer
- Any application where data integrity matters

**Known quality issues:**
- Minimal error handling
- No comprehensive testing
- Protocol implementation based on trial-and-error
- Magic numbers and timeouts tuned by vibes
- Code refactoring broke existing functionality (learned to avoid premature abstraction)

**Use at your own risk.** This is a hobbyist project for calculator enthusiasts who understand they're working with experimental code.
