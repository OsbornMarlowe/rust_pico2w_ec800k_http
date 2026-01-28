# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-01-28

### Fixed
- **Embassy API Compatibility**: Updated all code to work with Embassy commit `286d887529c66d8d1b4c7b56849e7a95386d79db`
- **ImageDef Import**: Changed from `embassy_rp::binary_info::ImageDef` to `embassy_rp::block::ImageDef`
- **Unsafe Attributes**: Added `unsafe(...)` wrapper for `link_section` attribute
- **BufferedUart API**: Removed lifetime parameters from `BufferedUart<'static>` type signature
- **BufferedUart Constructor**: Updated parameter order - pins now come before `Irqs` binding
- **Stack Type**: Changed from `Stack<'static>` to `Stack<cyw43::NetDriver<'static>>`
- **Write Macro Ambiguity**: Replaced `write!` macro with explicit `FmtWrite::write_fmt` + `format_args!` to avoid defmt conflicts
- **PioSpi Constructor**: Added clock frequency parameter (32 MHz) and reordered pin parameters (DIO before CLK)
- **Task Spawning**: Changed from `unwrap!(spawner.spawn(task()))` to `spawner.spawn(task()).unwrap()`
- **Unused Imports**: Removed `embassy_net::tcp::TcpSocket` and redundant `core::fmt::Write` imports

### Added
- **EMBASSY_API_FIXES.md**: Comprehensive documentation of all 33 API compatibility fixes
- **QUICK_FIX_REFERENCE.md**: Quick reference guide with one-liner solutions for common issues
- **CHANGELOG.md**: This file to track changes over time

### Changed
- All string formatting operations now use `core::fmt::Write as FmtWrite` explicitly
- Task spawning now uses standard Rust `.unwrap()` instead of defmt's `unwrap!()` macro
- UART pin order in constructor: RX (GP13) before TX (GP12)
- PioSpi pins: PIN_29 (DIO) before PIN_24 (CLK)

## [0.1.0] - 2025-01-27

### Added
- Initial Rust port from CircuitPython
- WiFi Access Point mode (SSID: "PicoLTE", password: "12345678")
- HTTP proxy server on port 80
- EC800K LTE module communication via UART at 921600 baud
- Auto-proxy default website (www.gzxxzlk.com) on root path
- Proxy any HTTP URL via `/proxy?url=http://example.com`
- Embassy async framework integration
- Three-task architecture:
  - CYW43 WiFi task
  - UART communication task
  - HTTP server task
- Inter-task communication via embassy channels
- AT command sequence for EC800K initialization
- TCP connection management via AT+QIOPEN
- HTTP request forwarding via AT+QISEND
- Response parsing and forwarding to client
- LED status indicator (blinks when system is running)
- Static IP configuration (192.168.4.1/24)
- Buffered UART for reliable communication
- HTML response formatting
- Error handling and user-friendly error pages

### Documentation
- README.md with quick start guide
- QUICK_START.md with detailed setup instructions
- IMPLEMENTATION_SUMMARY.md with technical architecture details
- Hardware wiring diagrams
- Configuration examples for different carriers
- Troubleshooting guide

## Notes

### Breaking Changes in 0.2.0
This version requires code changes if upgrading from 0.1.0. All changes are documented in EMBASSY_API_FIXES.md.

### Embassy Version Pinning
This project is tested against Embassy commit `286d887529c66d8d1b4c7b56849e7a95386d79db`. 
Using other Embassy versions may require additional API adaptations.

### Hardware Requirements
- Raspberry Pi Pico 2W (not Pico 2 or Pico W)
- EC800K LTE module with active SIM card
- External 5V/2A power supply for EC800K
- Proper TX/RX wiring (crossed connection)

### Known Limitations
- HTTP only (no HTTPS support)
- Sequential request handling (no concurrency)
- 8KB response buffer limit
- No DHCP server (clients may need manual IP)
- No caching mechanism

---

[Unreleased]: https://github.com/OsbornMarlowe/rust_pico2w_ec800k_http/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/OsbornMarlowe/rust_pico2w_ec800k_http/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/OsbornMarlowe/rust_pico2w_ec800k_http/releases/tag/v0.1.0