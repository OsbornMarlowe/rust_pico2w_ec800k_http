# Quick Flashing Instructions - Pico 2W EC800K HTTP Proxy

## Prerequisites

1. **Rust toolchain installed**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **ARM target installed**
   ```bash
   rustup target add thumbv8m.main-none-eabihf
   ```

3. **elf2uf2-rs tool installed**
   ```bash
   cargo install elf2uf2-rs --locked
   ```

4. **probe-rs (optional, for debugging)**
   ```bash
   cargo install probe-rs-tools --locked
   ```

---

## Method 1: Direct UF2 Flashing (Easiest)

### Step 1: Build the firmware
```bash
cd rust_pico2w_ec800k_http
cargo build --release
```

### Step 2: Convert to UF2 format
```bash
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway pico2w-wifi-gateway.uf2
```

### Step 3: Flash to Pico 2W

1. **Disconnect** Pico from USB
2. **Hold BOOTSEL button** on Pico
3. **Connect USB** while holding BOOTSEL
4. **Release BOOTSEL** - Pico appears as "RPI-RP2" or "RP2350" drive
5. **Copy** `pico2w-wifi-gateway.uf2` to the drive
6. **Wait** - Pico reboots automatically when copy completes

### Step 4: Verify

- LED should start blinking (500ms on, 500ms off)
- WiFi AP "PicoLTE" should appear in WiFi scan
- Connect with password: `12345678`
- Open browser to: `http://192.168.4.1`

---

## Method 2: Using probe-rs (For Debugging)

### Requirements
- probe-rs installed
- Debug probe OR second Pico as debugger
- SWD connections (SWDIO, SWCLK, GND)

### Flash and Monitor
```bash
cargo build --release
probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway
```

**Expected Output:**
```
=== BOOT: Pico 2W Starting ===
=== Pico 2W LTE Proxy ===
WiFi AP: PicoLTE / 12345678
Loading WiFi firmware...
Firmware loaded: 224190 bytes, CLM: 4752 bytes
Initializing CYW43 pins...
...
WiFi AP started successfully!
🚀 Auto-Proxy Ready!
```

---

## Method 3: Using picotool (Alternative)

### Install picotool
```bash
# On Linux/macOS with Homebrew
brew install picotool

# Or build from source
git clone https://github.com/raspberrypi/picotool.git
cd picotool
mkdir build && cd build
cmake ..
make
sudo make install
```

### Flash
```bash
# Put Pico in BOOTSEL mode first
picotool load pico2w-wifi-gateway.uf2
picotool reboot
```

---

## Troubleshooting Flashing Issues

### Problem: "No device found" or drive doesn't appear

**Solution:**
- Try a different USB cable (some are charge-only)
- Try a different USB port
- Hold BOOTSEL longer (3-5 seconds)
- Verify you have Pico 2W (not Pico W or Pico 2)

### Problem: Copy fails or Pico doesn't reboot

**Solution:**
- Ensure UF2 file is complete (check file size ~250KB-300KB)
- Try copying to drive again
- Manually eject/unmount drive before copying
- Check USB cable supports data transfer

### Problem: LED doesn't blink after flashing

**Possible causes:**
1. Wrong hardware - need Pico 2W specifically
2. Firmware corruption - reflash
3. See HARDWARE_TROUBLESHOOTING.md for detailed diagnostics

**Quick test:**
```bash
# Flash again in debug mode and check output
probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway
```

### Problem: WiFi AP doesn't appear

**Check:**
- LED is blinking (confirms firmware running)
- Using 5GHz-capable WiFi scanner (AP is on 2.4GHz)
- SSID: "PicoLTE" (case-sensitive)
- Wait 30-60 seconds after boot
- Check debug output with probe-rs

---

## Build Variants

### Release (Optimized, recommended)
```bash
cargo build --release
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway pico2w-wifi-gateway.uf2
```

### Debug (Larger, more verbose)
```bash
cargo build
elf2uf2-rs target/thumbv8m.main-none-eabihf/debug/pico2w-wifi-gateway pico2w-wifi-gateway-debug.uf2
```

---

## File Locations

After building, your files are located at:

```
rust_pico2w_ec800k_http/
├── target/thumbv8m.main-none-eabihf/
│   └── release/
│       └── pico2w-wifi-gateway          # ELF binary (can't flash directly)
└── pico2w-wifi-gateway.uf2              # UF2 file (flash this!)
```

**Important:** Only the `.uf2` file can be flashed via BOOTSEL method!

---

## Automated Build Script

Save this as `build-and-flash.sh`:

```bash
#!/bin/bash
set -e

echo "Building firmware..."
cargo build --release

echo "Converting to UF2..."
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway pico2w-wifi-gateway.uf2

echo "✅ Build complete: pico2w-wifi-gateway.uf2"
echo ""
echo "To flash:"
echo "1. Hold BOOTSEL button on Pico"
echo "2. Connect USB"
echo "3. Copy pico2w-wifi-gateway.uf2 to the drive"
echo ""
echo "Or use: probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway"
```

Make executable:
```bash
chmod +x build-and-flash.sh
./build-and-flash.sh
```

---

## Quick Reference

| Step | Command |
|------|---------|
| Build | `cargo build --release` |
| Convert | `elf2uf2-rs target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway pico2w-wifi-gateway.uf2` |
| Flash | Hold BOOTSEL + Copy UF2 file |
| Debug | `probe-rs run --chip RP2350 target/...` |

**WiFi AP:**
- SSID: `PicoLTE`
- Password: `12345678`
- IP: `192.168.4.1`

**Default proxy:** http://192.168.4.1 → www.gzxxzlk.com

---

**Last Updated:** 2025-01-28