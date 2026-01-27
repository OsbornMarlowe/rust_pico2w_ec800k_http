# Quick Start Guide - Pico 2W EC800K HTTP Proxy

## What This Does
Your Pico 2W creates a WiFi hotspot and proxies HTTP requests through the EC800K LTE module. Connect your phone/laptop to the Pico's WiFi, and browse websites through 4G/LTE!

## Hardware Setup

### Required Components
- Raspberry Pi Pico 2W
- EC800K LTE Module
- SIM Card (activated, with data plan)
- Power supply for EC800K (5V/2A recommended)

### Wiring
```
EC800K          →    Pico 2W
─────────────────────────────
TX              →    GP13 (RX)
RX              →    GP12 (TX)
GND             →    GND
VCC (5V)        →    External 5V/2A power
```

**Important Notes:**
- EC800K needs its own power supply (can draw up to 2A)
- Do NOT power EC800K from Pico's 3.3V or 5V pins
- Double-check TX/RX connections (they are crossed)

## Software Setup

### 1. Install Rust (if not already installed)
```bash
# Visit https://rustup.rs/ or run:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add ARM Cortex-M target
rustup target add thumbv8m.main-none-eabihf
```

### 2. Install Flashing Tool
```bash
# Option A: probe-rs (for debug probe)
cargo install probe-rs --features cli

# Option B: elf2uf2-rs (for BOOTSEL mode)
cargo install elf2uf2-rs
```

### 3. Build the Project
```bash
cd rust_pico2w_ec800k_http
cargo build --release
```

### 4. Flash to Pico 2W

**Method A: Using probe-rs (with debug probe)**
```bash
probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway
```

**Method B: Using BOOTSEL mode**
```bash
# 1. Hold BOOTSEL button while plugging in Pico
# 2. Convert ELF to UF2
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway pico2w-wifi-gateway.uf2

# 3. Copy UF2 file to the mounted Pico drive
# On Windows: copy pico2w-wifi-gateway.uf2 D:\
# On Linux/Mac: cp pico2w-wifi-gateway.uf2 /media/RPI-RP2/
```

## Usage

### 1. Power Up
- Insert activated SIM card into EC800K
- Connect EC800K to external power (5V/2A)
- Connect Pico 2W to USB power or battery
- Wait 10-30 seconds for initialization

### 2. Connect to WiFi
- **Network Name (SSID):** `PicoLTE`
- **Password:** `12345678`
- **Your device will get IP:** Auto-assigned (or manually set 192.168.4.2-254)

### 3. Browse!
Open your browser and visit:

- **Auto-proxy (default):** `http://192.168.4.1`
  - Automatically loads www.gzxxzlk.com through LTE
  
- **Custom URL:** `http://192.168.4.1/proxy?url=http://example.com`
  - Replace `example.com` with any HTTP website

### 4. LED Indicator
- Pico's LED blinks slowly (1 sec on/off) = System running OK

## Configuration

### Change WiFi Settings
Edit `src/main.rs`:
```rust
const WIFI_SSID: &str = "PicoLTE";        // Your SSID
const WIFI_PASSWORD: &str = "12345678";   // Your password
```

### Change Default Website
Edit `src/main.rs`:
```rust
const DEFAULT_HOST: &str = "www.gzxxzlk.com";  // Your default site
const DEFAULT_PATH: &str = "/";
```

### Change APN (for different carrier)
Edit `src/main.rs` in `uart_task()` function:
```rust
send_at_command(&mut uart, "AT+QICSGP=1,1,\"CTNET\"").await;
                                              ^^^^^^
                                              Change this to your APN
```

**Common APNs:**
- China Telecom: `CTNET`
- China Mobile: `CMNET`
- China Unicom: `3GNET`
- AT&T (US): `phone`
- T-Mobile (US): `fast.t-mobile.com`
- Vodafone (EU): `internet`

### Change UART Baud Rate
Edit `src/main.rs`:
```rust
const UART_BAUDRATE: u32 = 921600;  // Try 115200 if having issues
```

## Troubleshooting

### WiFi not appearing
- ✅ Verify you have Pico **2W** (not regular Pico 2)
- ✅ Check firmware files in `cyw43-firmware/` directory
- ✅ Wait 10-20 seconds after power-up
- ✅ Try restarting Pico

### Can't connect to WiFi
- ✅ Password is correct: `12345678`
- ✅ 2.4GHz WiFi (5GHz not supported)
- ✅ Try forgetting network and reconnecting

### Browser shows "No response from EC800K"
- ✅ Check wiring (TX/RX must be crossed)
- ✅ EC800K has power (separate 5V/2A supply)
- ✅ SIM card inserted and activated
- ✅ LED on EC800K is blinking (indicates network activity)

### "TCP connection failed"
- ✅ SIM card has active data plan
- ✅ Network signal is good (check EC800K LED pattern)
- ✅ APN configured correctly for your carrier
- ✅ Wait 30-60 seconds for network registration

### Slow or incomplete pages
- ✅ Normal! LTE latency + processing = 2-5 seconds per page
- ✅ Large pages may timeout (8KB buffer limit)
- ✅ Try simpler websites
- ✅ Check signal strength

### "No HTML content found"
- ✅ Some websites block non-browser user agents
- ✅ HTTPS sites won't work (only HTTP supported)
- ✅ Try a different website

## Debug Output

To see detailed logs:

### Using probe-rs
```bash
probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway
```

You'll see:
```
INFO  UART task started at 921600 baud
INFO  Initializing EC800K...
INFO  TX: AT
INFO  RX: AT
       OK
INFO  WiFi AP started!
INFO  Network stack initialized at 192.168.4.1
...
```

### What to look for:
- `TX:` = Commands sent to EC800K
- `RX:` = Responses from EC800K
- `+QIOPEN: 0,0` = TCP connection successful
- `SEND OK` = HTTP request sent
- `Complete response detected` = Got full webpage

## Performance

| Metric | Value |
|--------|-------|
| WiFi Speed | ~10-20 Mbps |
| LTE Speed | Carrier dependent |
| Page Load Time | 2-5 seconds |
| Max Page Size | 8KB (configurable) |
| Concurrent Requests | 1 (sequential) |

## Limitations

- ❌ **HTTPS not supported** (only HTTP)
- ❌ **One request at a time** (no concurrent connections)
- ❌ **8KB response limit** (large pages may be truncated)
- ❌ **No caching** (every request goes through LTE)
- ❌ **Basic error handling** (some edge cases may fail)

## Example Use Cases

✅ **IoT data collection** - Fetch data from HTTP APIs  
✅ **Remote monitoring** - Display sensor data via LTE  
✅ **Emergency connectivity** - Basic web access via LTE when WiFi unavailable  
✅ **Testing EC800K** - Verify module functionality  
✅ **Learning project** - Understand embedded networking  

## Next Steps

1. ✅ Get it working with default settings
2. ✅ Try different websites
3. ✅ Adjust APN for your carrier
4. ✅ Monitor debug output to understand flow
5. ✅ Modify code for your specific needs

## Getting Help

- 📖 Read `IMPLEMENTATION_SUMMARY.md` for technical details
- 🐛 Check GitHub issues
- 📝 Review debug output for error messages
- 🔧 Try different baud rates if UART issues

## Safety Notes

⚠️ **Power Supply:**
- EC800K can draw up to 2A during transmission
- Use adequate power supply
- Separate from Pico power

⚠️ **Heat:**
- EC800K may get warm during operation (normal)
- Ensure adequate ventilation

⚠️ **Data Usage:**
- Each page request uses cellular data
- Monitor your data plan
- Some pages can be several MB

## Success Checklist

- [ ] Pico 2W (not regular Pico 2)
- [ ] EC800K with activated SIM card
- [ ] Correct wiring (TX↔RX crossed)
- [ ] EC800K has separate power (5V/2A)
- [ ] Code built and flashed successfully
- [ ] WiFi "PicoLTE" appears
- [ ] Can connect with password "12345678"
- [ ] Browser opens to http://192.168.4.1
- [ ] Website loads through LTE

If all checked ✅ - You're done! Enjoy your LTE-powered WiFi hotspot!

---

**Questions?** Check the implementation summary or debug output for more details.