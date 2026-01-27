#![no_std]
#![no_main]

use core::fmt::Write as _;
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config, Stack, StackResources};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0, UART0};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig};
use embassy_time::{Duration, Timer};
use embedded_io_async::Read;
use embedded_io_async::Write;
use heapless::String;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Program metadata
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Pico LTE Proxy"),
    embassy_rp::binary_info::rp_program_description!(
        c"Pico 2W WiFi AP + EC800K LTE HTTP Proxy"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

const WIFI_SSID: &str = "PicoLTE";
const WIFI_PASSWORD: &str = "12345678";
const UART_BAUDRATE: u32 = 921600;

// Default proxy target
const DEFAULT_HOST: &str = "www.gzxxzlk.com";
const DEFAULT_PATH: &str = "/";

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

// Global channel for UART communication
static UART_CHANNEL: embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    UartRequest,
    1,
> = embassy_sync::channel::Channel::new();

static UART_RESPONSE: embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    UartResponse,
    1,
> = embassy_sync::channel::Channel::new();

#[derive(Clone)]
struct UartRequest {
    host: String<64>,
    path: String<128>,
}

struct UartResponse {
    data: String<8192>,
    success: bool,
}

#[embassy_executor::task]
async fn uart_task(mut uart: BufferedUart<'static, UART0>) {
    info!("UART task started at {} baud", UART_BAUDRATE);

    // Initialize EC800K
    Timer::after(Duration::from_secs(2)).await;

    info!("Initializing EC800K...");
    send_at_command(&mut uart, "AT").await;
    Timer::after(Duration::from_millis(500)).await;

    send_at_command(&mut uart, "AT+CPIN?").await;
    Timer::after(Duration::from_millis(500)).await;

    send_at_command(&mut uart, "AT+CREG?").await;
    Timer::after(Duration::from_millis(500)).await;

    send_at_command(&mut uart, "AT+CGATT=1").await;
    Timer::after(Duration::from_secs(1)).await;

    send_at_command(&mut uart, "AT+QICSGP=1,1,\"CTNET\"").await;
    Timer::after(Duration::from_millis(500)).await;

    send_at_command(&mut uart, "AT+QIACT=1").await;
    Timer::after(Duration::from_secs(2)).await;

    send_at_command(&mut uart, "AT+QIACT?").await;
    Timer::after(Duration::from_millis(500)).await;

    send_at_command(&mut uart, "AT+QIDNSCFG=1,\"114.114.114.114\",\"8.8.8.8\"").await;
    Timer::after(Duration::from_millis(500)).await;

    info!("EC800K initialized successfully!");

    // Main loop - wait for HTTP requests
    loop {
        let request = UART_CHANNEL.receive().await;
        info!(
            "Received request for {}:{}",
            request.host.as_str(),
            request.path.as_str()
        );

        let result = fetch_via_lte(&mut uart, &request.host, &request.path).await;

        UART_RESPONSE.send(result).await;
    }
}

async fn send_at_command(uart: &mut BufferedUart<'static, UART0>, cmd: &str) {
    let mut cmd_buf = String::<256>::new();
    let _ = cmd_buf.push_str(cmd);
    let _ = cmd_buf.push_str("\r\n");

    info!("TX: {}", cmd);
    let _ = uart.write_all(cmd_buf.as_bytes()).await;

    // Read response
    let mut response = [0u8; 512];
    Timer::after(Duration::from_millis(100)).await;

    if let Ok(n) =
        embassy_time::with_timeout(Duration::from_secs(2), uart.read(&mut response)).await
    {
        if let Ok(n) = n {
            if let Ok(resp_str) = core::str::from_utf8(&response[..n]) {
                info!("RX: {}", resp_str.trim());
            }
        }
    }
}

async fn clear_uart_buffer(uart: &mut BufferedUart<'static, UART0>) {
    Timer::after(Duration::from_millis(500)).await;
    let mut discard = [0u8; 256];
    while let Ok(_) =
        embassy_time::with_timeout(Duration::from_millis(100), uart.read(&mut discard)).await
    {}
}

async fn fetch_via_lte(
    uart: &mut BufferedUart<'static, UART0>,
    host: &str,
    path: &str,
) -> UartResponse {
    info!("Fetching http://{}{} via LTE...", host, path);

    // Clear buffer
    clear_uart_buffer(uart).await;

    // Step 1: Open TCP connection
    info!("1. Opening TCP connection...");
    let mut open_cmd = String::<256>::new();
    let _ = write!(open_cmd, "AT+QIOPEN=1,0,\"TCP\",\"{}\",80,0,1\r\n", host);
    let _ = uart.write_all(open_cmd.as_bytes()).await;

    // Wait for +QIOPEN: 0,0
    let mut response = [0u8; 256];
    let mut connected = false;
    for _ in 0..20 {
        Timer::after(Duration::from_millis(500)).await;
        if let Ok(n) = embassy_time::with_timeout(
            Duration::from_millis(500),
            uart.read(&mut response),
        )
        .await
        {
            if let Ok(n) = n {
                if let Ok(resp_str) = core::str::from_utf8(&response[..n]) {
                    info!("Open response: {}", resp_str);
                    if resp_str.contains("+QIOPEN: 0,0") {
                        connected = true;
                        break;
                    }
                }
            }
        }
    }

    if !connected {
        warn!("TCP connection failed");
        return UartResponse {
            data: String::from("TCP connection failed"),
            success: false,
        };
    }

    info!("✅ TCP connected");
    Timer::after(Duration::from_secs(1)).await;

    // Step 2: Prepare HTTP request
    let mut http_request = String::<512>::new();
    let _ = write!(
        http_request,
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: PicoLTE-Proxy/1.0\r\n\r\n",
        path, host
    );

    // Step 3: Send HTTP data
    info!("2. Sending HTTP request...");
    let mut send_cmd = String::<64>::new();
    let _ = write!(send_cmd, "AT+QISEND=0,{}\r\n", http_request.len());
    let _ = uart.write_all(send_cmd.as_bytes()).await;

    // Wait for '>'
    Timer::after(Duration::from_millis(500)).await;
    let mut got_prompt = false;
    if let Ok(n) =
        embassy_time::with_timeout(Duration::from_secs(5), uart.read(&mut response)).await
    {
        if let Ok(n) = n {
            if let Ok(resp_str) = core::str::from_utf8(&response[..n]) {
                if resp_str.contains(">") {
                    got_prompt = true;
                }
            }
        }
    }

    if !got_prompt {
        warn!("No send prompt received");
        let _ = uart.write_all(b"AT+QICLOSE=0\r\n").await;
        return UartResponse {
            data: String::from("No send prompt"),
            success: false,
        };
    }

    // Send actual HTTP data
    let _ = uart.write_all(http_request.as_bytes()).await;
    Timer::after(Duration::from_millis(500)).await;

    // Wait for SEND OK
    info!("3. Waiting for SEND OK...");
    let mut got_send_ok = false;
    for _ in 0..10 {
        if let Ok(n) = embassy_time::with_timeout(
            Duration::from_millis(500),
            uart.read(&mut response),
        )
        .await
        {
            if let Ok(n) = n {
                if let Ok(resp_str) = core::str::from_utf8(&response[..n]) {
                    if resp_str.contains("SEND OK") {
                        got_send_ok = true;
                        info!("✅ SEND OK received");
                        break;
                    }
                }
            }
        }
        Timer::after(Duration::from_millis(100)).await;
    }

    if !got_send_ok {
        warn!("SEND OK not received");
    }

    // Step 4: Collect HTTP response
    info!("4. Collecting HTTP response...");
    let mut http_data = String::<8192>::new();
    let mut buffer = [0u8; 512];
    let mut no_data_count = 0;

    for _ in 0..60 {
        // 30 seconds max
        match embassy_time::with_timeout(Duration::from_millis(500), uart.read(&mut buffer)).await
        {
            Ok(Ok(n)) => {
                if let Ok(chunk) = core::str::from_utf8(&buffer[..n]) {
                    let _ = http_data.push_str(chunk);
                    no_data_count = 0;

                    // Check if we have complete response
                    if http_data.contains("</html>") || http_data.contains("</HTML>") {
                        info!("✅ Complete response detected");
                        break;
                    }
                }
            }
            _ => {
                no_data_count += 1;
                if no_data_count > 6 && http_data.len() > 0 {
                    info!("✅ No more data");
                    break;
                }
            }
        }
    }

    info!("Total response: {} bytes", http_data.len());

    // Step 5: Close connection
    info!("5. Closing connection...");
    let _ = uart.write_all(b"AT+QICLOSE=0\r\n").await;
    Timer::after(Duration::from_millis(500)).await;

    UartResponse {
        data: http_data,
        success: true,
    }
}

#[embassy_executor::task]
async fn http_server_task(stack: &'static Stack<cyw43::NetDriver<'static>>) {
    info!("HTTP server starting...");
    Timer::after(Duration::from_secs(1)).await;

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(30)));

        info!("Listening on port 80...");
        if let Err(e) = socket.accept(80).await {
            warn!("Accept error: {:?}", e);
            continue;
        }

        info!("Client connected!");

        // Read HTTP request
        let mut request_buf = [0u8; 1024];
        let mut total_read = 0;

        loop {
            match socket.read(&mut request_buf[total_read..]).await {
                Ok(0) => break,
                Ok(n) => {
                    total_read += n;
                    if total_read >= request_buf.len()
                        || request_buf[..total_read]
                            .windows(4)
                            .any(|w| w == b"\r\n\r\n")
                    {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        if total_read == 0 {
            warn!("No data received");
            continue;
        }

        let request_str = match core::str::from_utf8(&request_buf[..total_read]) {
            Ok(s) => s,
            Err(_) => {
                warn!("Invalid UTF-8");
                continue;
            }
        };

        info!(
            "Request: {}",
            request_str.split("\r\n").next().unwrap_or("")
        );

        // Parse request
        let (host, path) = if request_str.starts_with("GET /proxy?url=") {
            // Parse URL parameter
            if let Some(url_start) = request_str.find("url=http://") {
                let url_part = &request_str[url_start + 11..];
                if let Some(url_end) = url_part.find(|c: char| c.is_whitespace() || c == '&') {
                    let full_url = &url_part[..url_end];
                    if let Some(slash_pos) = full_url.find('/') {
                        let h = &full_url[..slash_pos];
                        let p = &full_url[slash_pos..];
                        let mut host_str = String::<64>::new();
                        let _ = host_str.push_str(h);
                        let mut path_str = String::<128>::new();
                        let _ = path_str.push_str(p);
                        (Some(host_str), Some(path_str))
                    } else {
                        let mut host_str = String::<64>::new();
                        let _ = host_str.push_str(full_url);
                        let mut path_str = String::<128>::new();
                        let _ = path_str.push_str("/");
                        (Some(host_str), Some(path_str))
                    }
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        } else {
            // Default to www.gzxxzlk.com
            let mut host = String::<64>::new();
            let _ = host.push_str(DEFAULT_HOST);
            let mut path = String::<128>::new();
            let _ = path.push_str(DEFAULT_PATH);
            (Some(host), Some(path))
        };

        let response = if let (Some(h), Some(p)) = (host, path) {
            info!("Proxying: {}:{}", h.as_str(), p.as_str());

            // Send request to UART task
            UART_CHANNEL
                .send(UartRequest {
                    host: h.clone(),
                    path: p.clone(),
                })
                .await;

            // Wait for response
            let uart_resp = UART_RESPONSE.receive().await;

            if uart_resp.success {
                // Extract HTML content
                let html_content = extract_html(&uart_resp.data);

                if html_content.len() > 0 {
                    info!("✅ Sending {} bytes to browser", html_content.len());
                    format_http_response(&html_content)
                } else {
                    info!("⚠️ No HTML content found");
                    format_error_response("No HTML content found in response")
                }
            } else {
                format_error_response(uart_resp.data.as_str())
            }
        } else {
            format_error_response("Invalid URL format. Use /proxy?url=http://example.com")
        };

        // Send response
        if let Err(e) = socket.write_all(response.as_bytes()).await {
            warn!("Write error: {:?}", e);
        }

        socket.close();
        Timer::after(Duration::from_millis(100)).await;
    }
}

fn extract_html(data: &str) -> String<8192> {
    let mut result = String::<8192>::new();

    // Find header end
    if let Some(header_end) = data.find("\r\n\r\n") {
        let _ = result.push_str(&data[header_end + 4..]);
    } else if let Some(html_start) = data.find("<!DOCTYPE") {
        let _ = result.push_str(&data[html_start..]);
    } else if let Some(html_start) = data.find("<html") {
        let _ = result.push_str(&data[html_start..]);
    } else if let Some(body_start) = data.find("<body") {
        let _ = result.push_str(&data[body_start..]);
    } else {
        let _ = result.push_str(data);
    }

    // Clean AT command artifacts
    let artifacts = ["AT+", "+QI", "SEND OK", "OK\r\n"];
    for artifact in &artifacts {
        if let Some(pos) = result.find(artifact) {
            result.truncate(pos);
            break;
        }
    }

    result
}

fn format_http_response(content: &str) -> String<8192> {
    let mut response = String::<8192>::new();
    let _ = write!(
        response,
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n{}",
        content
    );
    response
}

fn format_error_response(error: &str) -> String<8192> {
    let mut response = String::<8192>::new();
    let _ = write!(
        response,
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n\
        <!DOCTYPE html>\
        <html>\
        <head><title>Pico LTE Proxy - Error</title>\
        <style>body{{font-family:Arial,sans-serif;margin:40px;}}\
        .error{{color:red;background:#ffe6e6;padding:15px;border-radius:5px;}}</style>\
        </head>\
        <body>\
        <h1>Pico LTE Proxy</h1>\
        <div class=\"error\"><h2>Error</h2><p>{}</p></div>\
        <p>Baud rate: <strong>921600</strong></p>\
        <p>Try: <a href=\"/\">Auto-proxy www.gzxxzlk.com</a></p>\
        <p>Or: /proxy?url=http://example.com</p>\
        </body></html>",
        error
    );
    response
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("=== Pico 2W LTE Proxy ===");
    info!("WiFi AP: {} / {}", WIFI_SSID, WIFI_PASSWORD);
    info!("UART: {} baud on GP12/GP13", UART_BAUDRATE);

    // Initialize WiFi
    let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
        RM2_CLOCK_DIVIDER,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    spawner.spawn(cyw43_task(runner)).unwrap();

    // Start WiFi AP
    control.init(clm).await;
    control.start_ap_open(WIFI_SSID, 5).await;

    info!("WiFi AP started!");

    // Configure network stack with static IP
    let config = Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(embassy_net::Ipv4Address::new(192, 168, 4, 1), 24),
        dns_servers: heapless::Vec::new(),
        gateway: None,
    });

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let resources = RESOURCES.init(StackResources::new());

    static STACK: StaticCell<Stack<cyw43::NetDriver<'static>>> = StaticCell::new();
    let stack = STACK.init(Stack::new(
        net_device,
        config,
        resources,
        embassy_rp::clocks::RoscRng,
    ));

    spawner.spawn(net_task(stack.run())).unwrap();

    info!("Network stack initialized at 192.168.4.1");

    // Initialize UART for EC800K
    let uart_tx_buf = {
        static BUF: StaticCell<[u8; 256]> = StaticCell::new();
        BUF.init([0; 256])
    };
    let uart_rx_buf = {
        static BUF: StaticCell<[u8; 256]> = StaticCell::new();
        BUF.init([0; 256])
    };

    let mut uart_config = UartConfig::default();
    uart_config.baudrate = UART_BAUDRATE;

    let uart = BufferedUart::new(
        p.UART0,
        Irqs,
        p.PIN_12, // TX
        p.PIN_13, // RX
        uart_tx_buf,
        uart_rx_buf,
        uart_config,
    );

    spawner.spawn(uart_task(uart)).unwrap();

    info!("UART initialized");

    // Start HTTP server
    spawner.spawn(http_server_task(stack)).unwrap();

    info!("==================================================");
    info!("🚀 Auto-Proxy Ready!");
    info!("Connect to WiFi: {}", WIFI_SSID);
    info!("Open: http://192.168.4.1");
    info!("This will automatically show: {}", DEFAULT_HOST);
    info!("For other sites: http://192.168.4.1/proxy?url=http://example.com");
    info!("==================================================");

    // Keep LED blinking to show alive
    loop {
        control.gpio_set(0, true).await;
        Timer::after(Duration::from_secs(1)).await;
        control.gpio_set(0, false).await;
        Timer::after(Duration::from_secs(1)).await;
    }
}
