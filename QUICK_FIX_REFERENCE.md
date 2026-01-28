# Quick Fix Reference - Embassy API Updates

## One-Liner Fixes

### 1. ImageDef Location Change
```rust
// Change this:
embassy_rp::binary_info::ImageDef
// To this:
embassy_rp::block::ImageDef
```

### 2. Unsafe Attribute
```rust
// Change this:
#[link_section = ".start_block"]
// To this:
#[unsafe(link_section = ".start_block")]
```

### 3. BufferedUart Type
```rust
// No change needed - use as-is:
BufferedUart<'static>
```

### 4. BufferedUart Constructor
```rust
// Change parameter order:
BufferedUart::new(
    p.UART0,
    p.PIN_13,      // RX first
    p.PIN_12,      // TX second
    Irqs,          // After pins
    uart_tx_buf,
    uart_rx_buf,
    uart_config,
)
```

### 5. Stack Type
```rust
// Change this:
Stack<'static>
// To this:
Stack<cyw43::NetDriver<'static>>
```

### 6. Write Macro
```rust
// Add import at top:
use core::fmt::Write as FmtWrite;

// Change this:
write!(buffer, "format {}", arg)
// To this:
FmtWrite::write_fmt(&mut buffer, format_args!("format {}", arg))
```

### 7. PioSpi Constructor
```rust
// Add clock frequency and reorder:
PioSpi::new(
    &mut pio.common,
    pio.sm0,
    32_000_000,    // Add: Clock frequency
    pio.irq0,
    cs,
    p.PIN_29,      // DIO first
    p.PIN_24,      // CLK second
    p.DMA_CH0,
)
```

### 8. Task Spawning
```rust
// Change this:
unwrap!(spawner.spawn(task_name(args)));
// To this:
spawner.spawn(task_name(args)).unwrap();
```

---

## Complete Example Pattern

### Before (Broken):
```rust
use embassy_net::tcp::TcpSocket;

#[link_section = ".start_block"]
pub static IMAGE_DEF: embassy_rp::binary_info::ImageDef = ...;

async fn uart_task(mut uart: BufferedUart<'static>) { ... }

async fn net_task(stack: &'static Stack<'static>) -> ! {
    stack.run().await
}

let _ = write!(buffer, "GET {} HTTP/1.1\r\n", path);

let spi = PioSpi::new(
    &mut pio.common,
    pio.sm0,
    pio.irq0,
    cs,
    p.PIN_24,
    p.PIN_29,
    p.DMA_CH0,
);

let uart = BufferedUart::new(
    p.UART0,
    Irqs,
    p.PIN_12,
    p.PIN_13,
    uart_tx_buf,
    uart_rx_buf,
    uart_config,
);

unwrap!(spawner.spawn(net_task(stack)));
```

### After (Fixed):
```rust
use core::fmt::Write as FmtWrite;

#[unsafe(link_section = ".start_block")]
pub static IMAGE_DEF: embassy_rp::block::ImageDef = ...;

async fn uart_task(mut uart: BufferedUart<'static>) { ... }

async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

let _ = FmtWrite::write_fmt(&mut buffer, 
    format_args!("GET {} HTTP/1.1\r\n", path));

let spi = PioSpi::new(
    &mut pio.common,
    pio.sm0,
    32_000_000,
    pio.irq0,
    cs,
    p.PIN_29,
    p.PIN_24,
    p.DMA_CH0,
);

let uart = BufferedUart::new(
    p.UART0,
    p.PIN_13,
    p.PIN_12,
    Irqs,
    uart_tx_buf,
    uart_rx_buf,
    uart_config,
);

spawner.spawn(net_task(stack)).unwrap();
```

---

## Common Errors → Quick Solutions

| Error Message | Solution |
|---------------|----------|
| `cannot find type 'ImageDef' in crate 'embassy_rp::binary_info'` | Change to `embassy_rp::block::ImageDef` |
| `unsafe attribute used without unsafe` | Wrap with `unsafe(...)` |
| `struct takes 0 lifetime arguments but 1 lifetime argument was supplied` | Remove the lifetime: `BufferedUart<'static>` stays as is |
| `'write' is ambiguous` | Use `FmtWrite::write_fmt` with `format_args!` |
| `the trait bound 'DMA_CH0: PioPin' is not satisfied` | Add clock frequency, swap PIN_29 and PIN_24 |
| `this function takes 8 arguments but 7 arguments were supplied` | Add `32_000_000` clock parameter to `PioSpi::new` |
| `expected 'SpawnToken<_>', found 'Result<...'` | Change `unwrap!(spawner.spawn(task()))` to `spawner.spawn(task()).unwrap()` |
| `no method named 'run' found for '&embassy_net::Stack'` | Change Stack type to include `cyw43::NetDriver<'static>` |

---

## Embassy Git Revision

Current working revision:
```
286d887529c66d8d1b4c7b56849e7a95386d79db
```

Pin in Cargo.toml:
```toml
embassy-rp = { git = "https://github.com/embassy-rs/embassy.git", rev = "286d887", features = [...] }
```

---

## Build Commands

```bash
# Clean build
cargo clean
cargo build --release

# Flash to Pico
# Method 1: Hold BOOTSEL, copy .uf2 file
cargo build --release
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway pico.uf2

# Method 2: Use probe-rs
probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway
```

---

## Hardware Notes

- **UART Baudrate:** 921600 (EC800K)
- **WiFi AP:** SSID="PicoLTE", Password="12345678"
- **IP Address:** 192.168.4.1
- **Pins:**
  - GP12: UART TX to EC800K
  - GP13: UART RX from EC800K
  - GP23: CYW43 Power
  - GP24: CYW43 Clock
  - GP25: CYW43 CS
  - GP29: CYW43 DIO

⚠️ **EC800K requires external 5V/2A power supply!**

---

**Last Updated:** 2025-01-28
**Status:** ✅ All fixes applied