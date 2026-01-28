# Hardware Troubleshooting Guide - Pico 2W EC800K HTTP Proxy

## Problem: Firmware Compiled But Not Working on Hardware

### Symptoms
- ✅ Code compiles successfully
- ❌ LED doesn't turn on after flashing
- ❌ WiFi AP "PicoLTE" not visible
- ❌ Pico appears to reboot but nothing happens

---

## Quick Diagnostics

### 1. Check You Have the RIGHT Hardware

**CRITICAL:** This project requires **Raspberry Pi Pico 2 W** (RP2350A/B with WiFi)

❌ **NOT Compatible:**
- Raspberry Pi Pico (RP2040, no WiFi)
- Raspberry Pi Pico W (RP2040 with WiFi - wrong chip!)
- Raspberry Pi Pico 2 (RP2350, no WiFi)

✅ **Compatible:**
- Raspberry Pi Pico 2 W (RP2350A with WiFi)
- Pimoroni Pico Plus 2 W (RP2350B with WiFi)

**How to verify:** Look for "Pico 2 W" printed on the board with a WiFi antenna connector.

---

## Step-by-Step Troubleshooting

### Step 1: Verify Firmware File Format

The firmware must be in `.uf2` format for direct flashing to Pico.

**Check your build output:**
```bash
# Your build creates an ELF file, not UF2
# Location: target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway
```

**Convert ELF to UF2:**
```bash
# Install elf2uf2-rs if not already installed
cargo install elf2uf2-rs

# Convert the firmware
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway pico2w-wifi-gateway.uf2

# Now flash pico2w-wifi-gateway.uf2 to the Pico
```

---

### Step 2: Proper Flashing Procedure

1. **Disconnect Pico from USB**
2. **Hold BOOTSEL button**
3. **Connect USB while holding BOOTSEL**
4. **Release BOOTSEL** - Pico appears as USB drive "RP2350" or "RPI-RP2"
5. **Copy `pico2w-wifi-gateway.uf2` to the drive**
6. **Pico automatically reboots** - Drive disappears

**Common Mistake:** Copying the ELF file instead of UF2 file won't work!

---

### Step 3: Enable Debug Logging

The code now has extensive debug logging. To see what's happening:

#### Option A: Using probe-rs (Recommended)

```bash
# Install probe-rs
cargo install probe-rs-tools --locked

# Flash and monitor
probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway
```

**Expected output if working:**
```
=== BOOT: Pico 2W Starting ===
=== Pico 2W LTE Proxy ===
WiFi AP: PicoLTE / 12345678
Loading WiFi firmware...
Firmware loaded: 224190 bytes, CLM: 4752 bytes
Initializing CYW43 pins...
PIO initialized
Creating CYW43 state...
Initializing CYW43 driver...
CYW43 driver initialized
CYW43 task spawned
Initializing WiFi with CLM data...
CLM initialized
Starting AP mode: SSID=PicoLTE
WiFi AP started successfully!
...
```

#### Option B: Using picoprobe/CMSIS-DAP

If you have a second Pico or debug probe:

1. Connect SWD pins (SWDIO, SWCLK, GND)
2. Use `probe-rs` or `openocd`
3. Monitor RTT output

---

### Step 4: Common Hardware Issues

#### Issue: No Debug Output at All

**Possible Causes:**
1. **Wrong chip selected** - Verify `rp235xa` in Cargo.toml features
2. **Corrupt firmware** - Reflash using BOOTSEL method
3. **Power issue** - Try different USB cable/port

**Fix:**
```toml
# In Cargo.toml, verify this feature:
embassy-rp = { ..., features = ["rp235xa", ...] }
```

#### Issue: Panic/Hard Fault on Boot

**Symptoms:** LED flashes rapidly or stays off

**Common Causes:**
1. **Stack overflow** - Check memory.x configuration
2. **Invalid memory access** - Boot2 bootloader issue
3. **Missing firmware files** - Verify cyw43-firmware/*.bin exist

**Fix - Increase Stack Size:**
```rust
// In memory.x or Cargo.toml
_stack_size = 32K;  // Increase if needed
```

#### Issue: CYW43 Initialization Fails

**Symptoms:** Debug log stops at "Initializing CYW43 driver..."

**Causes:**
1. **Incorrect pin configuration**
2. **PIO/SPI issue**
3. **Firmware file corruption**

**Verify pins:**
- GP23: CYW43 Power
- GP24: CYW43 Clock  
- GP25: CYW43 CS
- GP29: CYW43 DIO

**Fix:** Re-download firmware files:
```bash
# Get fresh firmware files from Embassy
wget https://github.com/embassy-rs/embassy/raw/main/cyw43-firmware/43439A0.bin
wget https://github.com/embassy-rs/embassy/raw/main/cyw43-firmware/43439A0_clm.bin
```

#### Issue: WiFi AP Doesn't Start

**Symptoms:** Debug log shows "CYW43 driver initialized" but stops before "WiFi AP started"

**Causes:**
1. **CLM data issue**
2. **Regulatory domain mismatch**
3. **Channel conflict**

**Fix:** Try different channel:
```rust
// In main.rs, change channel from 5 to 1 or 6
control.start_ap_open(WIFI_SSID, 1).await;  // Try channel 1
```

---

### Step 5: EC800K Module Issues

**CRITICAL:** EC800K requires **external 5V/2A power supply**

#### Verify EC800K Power

1. **Check power LED on EC800K** - Should be solid or blinking
2. **Measure voltage** - Should be 5V on power pins
3. **Check current** - Should draw 100-500mA when idle

#### Verify UART Wiring

**Correct wiring:**
```
Pico GP12 (TX) → EC800K RX
Pico GP13 (RX) → EC800K TX
GND           → GND
```

**Common mistakes:**
- ❌ TX to TX, RX to RX (should be crossed!)
- ❌ Using 3.3V from Pico to power EC800K (insufficient!)
- ❌ No common ground

---

### Step 6: Memory Issues

#### Check Flash Usage

```bash
# After building, check size
arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway

# Output should show:
#    text    data     bss     dec     hex filename
#  250000   1000   10000  261000   3fc18 pico2w-wifi-gateway
```

**If text > 2MB:** Firmware too large for flash!

**Fix:**
1. Enable LTO in Cargo.toml (already done)
2. Reduce buffer sizes in code
3. Remove debug symbols from release build

---

## Advanced Debugging

### Using Logic Analyzer

Monitor these signals:
- **SPI (PIO):** CLK, DIO between Pico and CYW43
- **UART:** TX/RX between Pico and EC800K at 921600 baud

### Using Oscilloscope

Check:
- **3.3V rail** - Should be stable
- **CYW43 power pin** - Should go LOW then HIGH during init
- **Clock signals** - Should show activity on PIO CLK

---

## Known Working Configuration

```rust
// Verified working on:
// - Raspberry Pi Pico 2 W (RP2350A)
// - Embassy commit: 286d887529c66d8d1b4c7b56849e7a95386d79db
// - Rust toolchain: stable or nightly
// - probe-rs: 0.24.0

// Features in Cargo.toml:
embassy-rp = { features = ["rp235xa", "binary-info", ...] }

// Pin configuration:
GP12: UART TX to EC800K
GP13: UART RX from EC800K  
GP23: CYW43 Power
GP24: CYW43 Clock
GP25: CYW43 CS
GP29: CYW43 DIO
```

---

## Success Indicators

✅ **If working correctly, you should see:**

1. **Debug log output** (via probe-rs):
   - Boot messages
   - WiFi initialization
   - "WiFi AP started successfully!"
   - LED blink count incrementing

2. **LED behavior:**
   - Blinks every 1 second (500ms on, 500ms off)
   - Consistent, not erratic

3. **WiFi AP visible:**
   - SSID: "PicoLTE"
   - Open network (no password for scanning)
   - Channel 5 (or configured channel)

4. **Network connectivity:**
   - Can connect to WiFi AP
   - Can access http://192.168.4.1

---

## Still Not Working?

### Checklist:

- [ ] Confirmed Pico 2 W hardware (not Pico W or Pico 2)
- [ ] Built with correct features (`rp235xa`)
- [ ] Converted ELF to UF2 format
- [ ] Flashed using BOOTSEL method
- [ ] Firmware files exist and are correct
- [ ] EC800K has external 5V power supply
- [ ] UART wiring is crossed (TX to RX)
- [ ] Using probe-rs or debug probe to see logs
- [ ] Tried different USB cable/port
- [ ] Tried reflashing multiple times

### Get Help:

1. **Capture debug output** with probe-rs
2. **Take photos** of your wiring setup
3. **Document** exact hardware revisions
4. **Share** firmware build command and output
5. **Open issue** on GitHub with all above info

---

## Quick Test Firmware

If nothing works, try this minimal blink test:

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);
    
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
```

**If this works:** Hardware is OK, issue is in WiFi/network code  
**If this doesn't work:** Hardware or toolchain issue

---

**Last Updated:** 2025-01-28  
**Status:** Comprehensive troubleshooting guide