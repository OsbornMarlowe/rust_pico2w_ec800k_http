# Rust Pico 2W EC800K HTTP Proxy

🚀 **Turn your Raspberry Pi Pico 2W into a WiFi Access Point that proxies HTTP requests through an EC800K LTE module!**

## What It Does

This project creates a portable WiFi hotspot using the Pico 2W that routes all HTTP traffic through a 4G/LTE cellular connection via the EC800K module. Connect your phone or laptop to the Pico's WiFi and browse the web through cellular data!

## Features

✨ **WiFi Access Point Mode**
- SSID: `PicoLTE`
- Password: `12345678`
- Static IP: `192.168.4.1`

🌐 **HTTP Proxy via LTE**
- Auto-proxy default website (www.gzxxzlk.com)
- Proxy any HTTP URL via query parameter
- Full AT command communication with EC800K

⚡ **High-Speed UART**
- 921600 baud communication
- Buffered async I/O
- Reliable AT command handling

🔧 **Embassy Async Framework**
- Efficient task management
- Non-blocking operations
- Low power consumption

## Quick Start

### Hardware Requirements
- Raspberry Pi Pico 2W
- EC800K LTE Module
- Active SIM card with data plan
- 5V/2A power supply for EC800K

### Wiring
```
EC800K TX  →  Pico GP13 (RX)
EC800K RX  →  Pico GP12 (TX)
GND        →  GND
```

### Installation

1. **Install Rust and tools:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add thumbv8m.main-none-eabihf
cargo install elf2uf2-rs
```

2. **Clone and build:**
```bash
git clone https://github.com/OsbornMarlowe/rust_pico2w_ec800k_http.git
cd rust_pico2w_ec800k_http
cargo build --release
```

3. **Flash to Pico 2W:**
```bash
# Hold BOOTSEL button while connecting USB
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway pico2w-wifi-gateway.uf2
# Copy the .uf2 file to the Pico drive
```

### Usage

1. Power up the Pico 2W with EC800K connected
2. Connect to WiFi network `PicoLTE` (password: `12345678`)
3. Open browser to:
   - `http://192.168.4.1` - Auto-loads www.gzxxzlk.com
   - `http://192.168.4.1/proxy?url=http://example.com` - Proxy any HTTP site

## Documentation

📖 **[Quick Start Guide](QUICK_START.md)** - Step-by-step setup instructions

📖 **[Implementation Summary](IMPLEMENTATION_SUMMARY.md)** - Technical details and architecture

## Configuration

All settings can be modified in `src/main.rs`:

```rust
// WiFi Settings
const WIFI_SSID: &str = "PicoLTE";
const WIFI_PASSWORD: &str = "12345678";

// UART Settings
const UART_BAUDRATE: u32 = 921600;

// Default Proxy Target
const DEFAULT_HOST: &str = "www.gzxxzlk.com";
const DEFAULT_PATH: &str = "/";
```

### APN Configuration

For different carriers, update the APN in the `uart_task()` function:

```rust
send_at_command(&mut uart, "AT+QICSGP=1,1,\"YOUR_APN_HERE\"").await;
```

Common APNs:
- China Telecom: `CTNET`
- China Mobile: `CMNET`
- China Unicom: `3GNET`
- AT&T: `phone`
- T-Mobile: `fast.t-mobile.com`

## Architecture

The project uses Embassy async framework with three main tasks:

1. **WiFi Task** - Manages CYW43 WiFi chip
2. **UART Task** - Communicates with EC800K via AT commands
3. **HTTP Server Task** - Handles incoming HTTP requests

Communication flow:
```
Browser → WiFi → HTTP Server → UART Task → EC800K → Internet
                                    ↓
                            [Response Channel]
                                    ↓
Browser ← WiFi ← HTTP Server ← UART Task ← EC800K ← Internet
```

## Troubleshooting

### WiFi not appearing
- Verify you have Pico **2W** (not regular Pico 2)
- Wait 10-20 seconds after power-up
- Check debug output via probe-rs

### TCP connection failed
- Verify SIM card is activated with data plan
- Check APN settings for your carrier
- Ensure good cellular signal
- Wait 30-60 seconds for network registration

### No response from EC800K
- Double-check TX/RX wiring (must be crossed)
- Verify EC800K has external power (5V/2A)
- Try lower baud rate (115200)

See [QUICK_START.md](QUICK_START.md) for detailed troubleshooting.

## Limitations

- ❌ HTTPS not supported (HTTP only)
- ❌ Sequential requests (one at a time)
- ❌ 8KB response buffer limit
- ❌ No DHCP server (clients may need manual IP configuration)
- ❌ No caching

## Dependencies

- [embassy-rs](https://github.com/embassy-rs/embassy) - Async embedded framework
- [cyw43](https://github.com/embassy-rs/embassy/tree/main/cyw43) - WiFi driver
- [defmt](https://github.com/knurling-rs/defmt) - Efficient logging
- [heapless](https://github.com/rust-embedded/heapless) - Stack-allocated collections

## License

MIT OR Apache-2.0

## Contributing

Contributions welcome! Please feel free to submit issues or pull requests.

## Acknowledgments

- Original CircuitPython implementation concept
- Embassy-rs team for excellent async framework
- Raspberry Pi Foundation for Pico 2W
- Quectel for EC800K documentation

## Related Projects

- [embassy-rs examples](https://github.com/embassy-rs/embassy/tree/main/examples/rp)
- [pico-examples](https://github.com/raspberrypi/pico-examples)

---

**Built with ❤️ using Rust and Embassy**