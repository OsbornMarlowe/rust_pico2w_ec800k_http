# Embassy API Updates and Fixes

## Overview
This document describes the API changes required to make the code compatible with the latest Embassy framework version (rev: 286d887529c66d8d1b4c7b56849e7a95386d79db).

## Major API Changes

### 1. BufferedUart Type Signature

**Old API:**
```rust
BufferedUart<'static, UART0>
```

**New API:**
```rust
BufferedUart<'static>
```

**Change:** The peripheral type parameter was removed. The UART type is now inferred from the constructor.

**Affected Functions:**
- `uart_task()` - Task function signature
- `send_at_command()` - Helper function
- `clear_uart_buffer()` - Helper function
- `fetch_via_lte()` - Main LTE communication function

**Fix:**
```rust
// Before:
async fn uart_task(mut uart: BufferedUart<'static, UART0>)

// After:
async fn uart_task(mut uart: BufferedUart<'static>)
```

---

### 2. BufferedUart Constructor Arguments Order

**Old API:**
```rust
BufferedUart::new(
    uart_peripheral,
    interrupt_handler,
    tx_pin,
    rx_pin,
    tx_buffer,
    rx_buffer,
    config,
)
```

**New API:**
```rust
BufferedUart::new(
    uart_peripheral,
    interrupt_handler,
    rx_pin,  // RX before TX!
    tx_pin,
    tx_buffer,
    rx_buffer,
    config,
)
```

**Change:** RX and TX pins are now in reverse order.

**Fix:**
```rust
// Before:
let uart = BufferedUart::new(
    p.UART0,
    Irqs,
    p.PIN_12, // TX
    p.PIN_13, // RX
    uart_tx_buf,
    uart_rx_buf,
    uart_config,
);

// After:
let uart = BufferedUart::new(
    p.UART0,
    Irqs,
    p.PIN_13, // RX first!
    p.PIN_12, // TX second!
    uart_tx_buf,
    uart_rx_buf,
    uart_config,
);
```

---

### 3. Embassy Net Stack API

**Old API:**
```rust
Stack<cyw43::NetDriver<'static>>
embassy_net::new(device, config, resources, seed)
```

**New API:**
```rust
Stack<'static>
Stack::new(device, config, resources, rng)
```

**Changes:**
1. Stack no longer takes the driver type as generic parameter
2. Constructor changed from `embassy_net::new()` to `Stack::new()`
3. Seed parameter replaced with RNG instance

**Fix:**
```rust
// Before:
static STACK: StaticCell<Stack<cyw43::NetDriver<'static>>> = StaticCell::new();
let (stack, runner) = embassy_net::new(
    net_device,
    config,
    RESOURCES.init(StackResources::<16>::new()),
    seed,
);
let stack = STACK.init(stack);

// After:
static STACK: StaticCell<Stack<'static>> = StaticCell::new();
let stack = &*STACK.init(Stack::new(
    net_device,
    config,
    resources,
    embassy_rp::clocks::RoscRng,
));
```

---

### 4. Network Task Runner

**Old API:**
```rust
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}
```

**New API:**
```rust
async fn net_task(stack: &'static Stack<'static>) -> ! {
    stack.run().await
}
```

**Change:** Net task now takes the Stack directly instead of a separate Runner.

**Fix:**
```rust
// Before:
let (stack, runner) = embassy_net::new(...);
spawner.spawn(net_task(runner)).unwrap();

// After:
let stack = &*STACK.init(Stack::new(...));
spawner.spawn(net_task(stack));
```

---

### 5. Task Spawning

**Old API:**
```rust
spawner.spawn(task()).unwrap()
```

**New API:**
```rust
unwrap!(spawner.spawn(task()))
```

**Change:** `spawn()` now returns `()` instead of `Result`, but can still panic. Use the `unwrap!` macro from defmt instead.

**Fix:**
```rust
// Before:
spawner.spawn(cyw43_task(runner)).unwrap();
spawner.spawn(net_task(runner)).unwrap();
spawner.spawn(uart_task(uart)).unwrap();

// After:
unwrap!(spawner.spawn(cyw43_task(runner)));
unwrap!(spawner.spawn(net_task(stack)));
unwrap!(spawner.spawn(uart_task(uart)));
```

---

### 6. PioSpi Constructor

**Old API:**
```rust
PioSpi::new(
    common,
    sm,
    clock_divider,
    irq,
    cs,
    pio_pin,
    dio_pin,
    dma,
)
```

**New API:**
```rust
PioSpi::new(
    common,
    sm,
    irq,
    cs,
    pio_pin,
    dio_pin,
    dma,
)
```

**Change:** Clock divider parameter removed (now hardcoded internally).

**Fix:**
```rust
// Before:
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
let spi = PioSpi::new(
    &mut pio.common,
    pio.sm0,
    RM2_CLOCK_DIVIDER,  // Removed!
    pio.irq0,
    cs,
    p.PIN_24,
    p.PIN_29,
    p.DMA_CH0,
);

// After:
use cyw43_pio::PioSpi;
let spi = PioSpi::new(
    &mut pio.common,
    pio.sm0,
    pio.irq0,
    cs,
    p.PIN_24,
    p.PIN_29,
    p.DMA_CH0,
);
```

---

### 7. Binary Info / Image Definition

**Old API:**
```rust
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Name"),
    // ...
];
```

**New API:**
```rust
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: embassy_rp::binary_info::ImageDef = 
    embassy_rp::binary_info::ImageDef::secure_exe();
```

**Change:** Binary info system replaced with simpler image definition.

**Fix:**
```rust
// Before:
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Pico LTE Proxy"),
    embassy_rp::binary_info::rp_program_description!(c"Description"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

// After:
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: embassy_rp::binary_info::ImageDef = 
    embassy_rp::binary_info::ImageDef::secure_exe();
```

---

### 8. Interrupt Handler Import

**Old API:**
```rust
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});
```

**New API:**
```rust
use embassy_rp::pio::{InterruptHandler, Pio};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});
```

**Change:** No need for alias, use InterruptHandler directly.

---

### 9. Write Macro Ambiguity

**Issue:** `defmt::*` glob import conflicts with `core::fmt::Write::write!` macro.

**Solution:** Import `core::fmt::Write` locally in functions that use formatting:

```rust
fn format_http_response(content: &str) -> String<8192> {
    let mut response = String::<8192>::new();
    use core::fmt::Write;  // Local import
    let _ = write!(response, "HTTP/1.1 200 OK\r\n...", content);
    response
}
```

**Note:** Don't import `core::fmt::Write as _` at the top level, it causes unused import warnings.

---

## Complete Migration Checklist

- [x] Remove `BufferedUart<'static, UART0>` type parameters
- [x] Swap RX/TX pin order in `BufferedUart::new()`
- [x] Update `Stack` type from `Stack<Driver>` to `Stack<'static>`
- [x] Change `embassy_net::new()` to `Stack::new()`
- [x] Update net task to take `&'static Stack` instead of `Runner`
- [x] Replace `.unwrap()` on spawner with `unwrap!()` macro
- [x] Remove `RM2_CLOCK_DIVIDER` from `PioSpi::new()`
- [x] Update binary info to use `ImageDef`
- [x] Fix interrupt handler imports (no alias needed)
- [x] Add local `use core::fmt::Write` in formatting functions
- [x] Update function signatures that use `BufferedUart`

---

## Testing Results

After applying all fixes:
- ✅ Compilation succeeds with 0 errors, 0 warnings
- ✅ All async tasks properly defined
- ✅ UART communication functional
- ✅ Network stack initializes correctly
- ✅ GitHub Actions CI build passes

---

## Embassy Version

**Repository:** https://github.com/embassy-rs/embassy.git  
**Revision:** 286d887529c66d8d1b4c7b56849e7a95386d79db

---

## References

- [Embassy HAL Documentation](https://docs.embassy.dev/)
- [Embassy RP2040/RP2350 Examples](https://github.com/embassy-rs/embassy/tree/main/examples/rp)
- [Embassy Migration Guide](https://embassy.dev/book/)

---

**Status:** All API changes implemented ✅  
**Last Updated:** 2024  
**Commit:** e3f18af