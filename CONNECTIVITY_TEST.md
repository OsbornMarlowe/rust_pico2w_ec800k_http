# Connectivity Testing Guide - Pico 2W HTTP Server

## Current Status: WiFi Connected, Page Not Loading

You've successfully connected to the WiFi but can't access the webpage at 192.168.4.1. Let's diagnose this step by step.

---

## Quick Diagnostic Checklist

### ✅ Step 1: Verify WiFi Connection
- [x] Can see "PicoLTE" SSID
- [x] Successfully connected with password "12345678"
- [x] LED on Pico is blinking

### ✅ Step 2: Verify IP Configuration
Your settings:
- IP Address: `192.168.4.2` ✅
- Subnet: `255.255.255.0` (prefix /24) ✅
- Gateway: `192.168.4.1` ✅
- DNS: `114.114.114.114` and `8.8.8.8` ✅

### ❓ Step 3: Test Network Connectivity

Open Command Prompt (Windows) or Terminal (Mac/Linux) and run:

```bash
# Test 1: Can you reach the Pico?
ping 192.168.4.1

# Expected output:
# Reply from 192.168.4.1: bytes=32 time=5ms TTL=64
```

**If ping fails:**
- ❌ Network stack on Pico isn't responding
- Check firmware logs with probe-rs
- Verify Pico is still running (LED blinking)

**If ping succeeds:**
- ✅ Layer 3 (IP) is working
- Continue to Step 4

---

### ❓ Step 4: Test TCP Port 80

```bash
# Windows (PowerShell)
Test-NetConnection -ComputerName 192.168.4.1 -Port 80

# Mac/Linux
nc -zv 192.168.4.1 80
# or
telnet 192.168.4.1 80

# Expected output:
# Connection to 192.168.4.1 port 80 [tcp/http] succeeded!
```

**If port test fails:**
- ❌ HTTP server not listening
- Rebuild firmware with latest fixes
- Check if http_server_task is spawned

**If port test succeeds:**
- ✅ HTTP server is listening
- Continue to Step 5

---

### ❓ Step 5: Test HTTP with curl

```bash
# Send HTTP request
curl -v http://192.168.4.1

# Expected output:
# * Connected to 192.168.4.1 (192.168.4.1) port 80
# > GET / HTTP/1.1
# > Host: 192.168.4.1
# < HTTP/1.1 200 OK
# < Content-Type: text/html
# ...
# <html>SUCCESS! Pico 2W HTTP Server is working!</html>
```

**If curl succeeds:**
- ✅ HTTP server is working!
- Problem is browser-specific
- Try different browser or clear cache

**If curl fails/hangs:**
- ❌ Server not responding to HTTP
- Check firmware logs
- See "Common Issues" below

---

## Common Issues and Fixes

### Issue 1: Ping works, but HTTP times out

**Symptoms:**
- `ping 192.168.4.1` succeeds
- Browser just spins/times out
- `curl http://192.168.4.1` hangs

**Cause:** HTTP server not fully initialized

**Debug with probe-rs:**
```bash
probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway
```

Look for these messages:
```
✅ REQUIRED:
Network link is UP!
Network config is UP!
HTTP SERVER READY on 192.168.4.1:80
🔵 Listening on TCP port 80...

❌ If missing:
- Network stack didn't initialize
- Rebuild with latest code
```

**Fix:**
1. Rebuild firmware with latest changes (includes network ready checks)
2. Reflash to Pico
3. Wait 5-10 seconds after LED starts blinking
4. Try again

---

### Issue 2: Browser redirects to HTTPS

**Symptoms:**
- Browser changes `http://192.168.4.1` to `https://192.168.4.1`
- Connection fails (no HTTPS support)

**Fix:**
```
1. Clear browser cache/history
2. Try incognito/private mode
3. Explicitly type: http://192.168.4.1
4. Press Enter (don't let browser auto-complete)

Or use curl which doesn't redirect:
curl http://192.168.4.1
```

---

### Issue 3: Browser uses IPv6 instead of IPv4

**Symptoms:**
- Browser shows `http://[fe80::xxxx]`
- Connection fails

**Fix:**
```
1. Disable IPv6 on WiFi adapter:
   - Windows: Network Adapter Properties → Uncheck IPv6
   - Mac: Network → Advanced → TCP/IP → Configure IPv6: Off
   - Linux: sudo sysctl -w net.ipv6.conf.wlan0.disable_ipv6=1

2. Disconnect and reconnect to WiFi

3. Try again with http://192.168.4.1
```

---

### Issue 4: Firewall blocking connection

**Symptoms:**
- `ping` works from command line
- `curl` works from command line
- Browser fails

**Fix:**
```
1. Temporarily disable firewall:
   - Windows: Windows Security → Firewall → Turn off
   - Mac: System Preferences → Security → Firewall → Turn Off
   - Linux: sudo ufw disable

2. Try browser again

3. If it works, add firewall exception for 192.168.4.0/24

4. Re-enable firewall
```

---

### Issue 5: DNS resolution problem

**Symptoms:**
- Can't access by IP
- Browser search instead of connecting

**Fix:**
```
1. Check URL carefully:
   ✅ http://192.168.4.1
   ❌ 192.168.4.1 (browser might search this)
   ❌ https://192.168.4.1 (no HTTPS)

2. Add to hosts file (optional):
   
   Windows: C:\Windows\System32\drivers\etc\hosts
   Mac/Linux: /etc/hosts
   
   Add line:
   192.168.4.1  pico.local
   
   Then access: http://pico.local
```

---

## Advanced Diagnostics

### Check Network Interfaces

**Windows:**
```powershell
ipconfig /all

# Look for:
# Wireless LAN adapter Wi-Fi:
#    Connection-specific DNS Suffix  . :
#    IPv4 Address. . . . . . . . . . . : 192.168.4.2
#    Subnet Mask . . . . . . . . . . . : 255.255.255.0
#    Default Gateway . . . . . . . . . : 192.168.4.1
```

**Mac/Linux:**
```bash
ifconfig
# or
ip addr show

# Look for:
# wlan0: ...
#     inet 192.168.4.2/24 brd 192.168.4.255 scope global wlan0
```

### Check Routing Table

**Windows:**
```powershell
route print

# Should show route to 192.168.4.0/24 via your WiFi interface
```

**Mac/Linux:**
```bash
ip route show
# or
netstat -rn

# Should show:
# 192.168.4.0/24 dev wlan0 scope link
```

### Packet Capture (Advanced)

**Windows (PowerShell as Admin):**
```powershell
netsh trace start capture=yes tracefile=C:\capture.etl
# Try to connect
netsh trace stop
# Analyze with Message Analyzer or Wireshark
```

**Mac/Linux:**
```bash
sudo tcpdump -i wlan0 -n host 192.168.4.1

# Try to connect in another terminal
# You should see:
# SYN packets going to 192.168.4.1:80
# SYN-ACK coming back (if server is up)
# PSH/ACK for HTTP data (if request sent)
```

---

## Rebuild Firmware with Diagnostics

The latest code includes test mode that responds immediately:

```bash
# Rebuild
cargo clean
cargo build --release

# Convert
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway pico2w.uf2

# Flash: Hold BOOTSEL + Connect + Copy pico2w.uf2
```

**Expected behavior after flash:**
1. LED starts blinking within 2-3 seconds
2. WiFi "PicoLTE" appears within 5-10 seconds
3. After connecting with manual IP, wait 10 seconds
4. `curl http://192.168.4.1` should return:
   ```html
   <!DOCTYPE html>
   <html>
   <head><title>Pico 2W Works!</title></head>
   <body>
   <h1>SUCCESS!</h1>
   <p>Pico 2W HTTP Server is working!</p>
   <p>Connection #1</p>
   </body>
   </html>
   ```

---

## Minimal Test Firmware

If still not working, try this minimal test (replace src/main.rs):

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
        Timer::after(Duration::from_millis(100)).await;
        led.set_low();
        Timer::after(Duration::from_millis(100)).await;
    }
}
```

**If this works (rapid blink):** Hardware OK, issue in WiFi/network code
**If this doesn't work:** Hardware or toolchain problem

---

## Expected Debug Output (with probe-rs)

When firmware is working correctly, you should see:

```
=== BOOT: Pico 2W Starting ===
Loading WiFi firmware...
Firmware loaded: 224190 bytes, CLM: 4752 bytes
Initializing CYW43 pins...
PIO initialized
Creating CYW43 state...
Initializing CYW43 driver...
CYW43 driver initialized
Initializing WiFi with CLM data...
CLM initialized
Starting AP mode: SSID=PicoLTE
WiFi AP started successfully!
Configuring network stack...
Network config: 192.168.4.1/24
Network stack initialized at 192.168.4.1
UART initialized
HTTP server starting...
Waiting for network link...
Network link is UP!
Waiting for network config...
Network config is UP!
==================================================
HTTP SERVER READY on 192.168.4.1:80
Client IP must be: 192.168.4.2-254/24
Gateway must be: 192.168.4.1
==================================================
🔵 Listening on TCP port 80... (connections: 0)
```

Then when you connect:
```
✅ Client connected! (connection #1)
✅ Response sent successfully
```

---

## Still Not Working?

1. **Verify hardware:** Confirm you have Pico 2W (not Pico W or Pico 2)

2. **Check firmware size:**
   ```bash
   arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway
   # text should be < 2MB
   ```

3. **Try different client device:** Test from phone, laptop, different OS

4. **Use probe-rs for live debugging:**
   ```bash
   probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway
   # Watch for errors in real-time
   ```

5. **Check Embassy version:** Should be commit `286d887`
   ```bash
   grep "rev.*286d887" Cargo.toml
   ```

6. **Report with details:**
   - Output of `ping 192.168.4.1`
   - Output of `curl -v http://192.168.4.1`
   - Debug logs from probe-rs
   - OS and browser version

---

**Last Updated:** 2025-01-28
**Status:** Troubleshooting network connectivity after WiFi connection