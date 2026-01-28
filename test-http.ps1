# Test HTTP connectivity to Pico 2W
# Run this in PowerShell to diagnose HTTP 502 error

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "Pico 2W HTTP Connectivity Test" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host ""

# Test 1: Ping
Write-Host "Test 1: Ping 192.168.4.1" -ForegroundColor Yellow
$pingResult = Test-Connection -ComputerName 192.168.4.1 -Count 4 -Quiet
if ($pingResult) {
    Write-Host "✅ PASS: Ping successful" -ForegroundColor Green
} else {
    Write-Host "❌ FAIL: Ping failed - Network layer not working" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Test 2: TCP Port 80
Write-Host "Test 2: TCP Port 80 connectivity" -ForegroundColor Yellow
try {
    $tcpResult = Test-NetConnection -ComputerName 192.168.4.1 -Port 80 -WarningAction SilentlyContinue
    if ($tcpResult.TcpTestSucceeded) {
        Write-Host "✅ PASS: Port 80 is open" -ForegroundColor Green
    } else {
        Write-Host "❌ FAIL: Port 80 not responding - HTTP server not running" -ForegroundColor Red
        Write-Host "   Action: Rebuild and reflash firmware with latest code" -ForegroundColor Yellow
        exit 1
    }
} catch {
    Write-Host "❌ FAIL: Cannot test port 80 - $_" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Test 3: HTTP GET with curl
Write-Host "Test 3: HTTP GET request" -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://192.168.4.1" -TimeoutSec 5 -UseBasicParsing
    Write-Host "✅ PASS: HTTP request successful!" -ForegroundColor Green
    Write-Host "Status Code: $($response.StatusCode)" -ForegroundColor Cyan
    Write-Host "Content Length: $($response.Content.Length) bytes" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Response Content:" -ForegroundColor Cyan
    Write-Host $response.Content -ForegroundColor White
} catch {
    $statusCode = $_.Exception.Response.StatusCode.value__
    Write-Host "❌ FAIL: HTTP request failed" -ForegroundColor Red
    Write-Host "Error: $_" -ForegroundColor Red

    if ($statusCode -eq 502) {
        Write-Host ""
        Write-Host "HTTP 502 Bad Gateway detected!" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "This means:" -ForegroundColor White
        Write-Host "  • Network layer works (ping successful)" -ForegroundColor White
        Write-Host "  • TCP connection works (port 80 open)" -ForegroundColor White
        Write-Host "  • HTTP server is NOT responding correctly" -ForegroundColor White
        Write-Host ""
        Write-Host "Solutions:" -ForegroundColor Yellow
        Write-Host "  1. Rebuild firmware with latest code:" -ForegroundColor White
        Write-Host "     cargo clean" -ForegroundColor Gray
        Write-Host "     cargo build --release" -ForegroundColor Gray
        Write-Host "     elf2uf2-rs target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway pico2w.uf2" -ForegroundColor Gray
        Write-Host ""
        Write-Host "  2. Reflash to Pico 2W (BOOTSEL + copy .uf2)" -ForegroundColor White
        Write-Host ""
        Write-Host "  3. Wait 10-15 seconds after LED starts blinking" -ForegroundColor White
        Write-Host ""
        Write-Host "  4. Monitor with probe-rs to see logs:" -ForegroundColor White
        Write-Host "     probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/pico2w-wifi-gateway" -ForegroundColor Gray
        Write-Host ""
        Write-Host "Look for these messages in probe-rs output:" -ForegroundColor White
        Write-Host "  ✅ 'HTTP server task spawned successfully'" -ForegroundColor Green
        Write-Host "  ✅ 'Network link is UP!'" -ForegroundColor Green
        Write-Host "  ✅ 'Network config is UP!'" -ForegroundColor Green
        Write-Host "  ✅ 'HTTP SERVER READY on 192.168.4.1:80'" -ForegroundColor Green
        Write-Host "  ✅ '🔵 Listening on TCP port 80...'" -ForegroundColor Green
    }
    exit 1
}
Write-Host ""

# Test 4: Alternative with curl.exe if available
Write-Host "Test 4: Testing with curl (if available)" -ForegroundColor Yellow
$curlPath = Get-Command curl.exe -ErrorAction SilentlyContinue
if ($curlPath) {
    Write-Host "Running: curl -v http://192.168.4.1" -ForegroundColor Cyan
    curl.exe -v http://192.168.4.1 2>&1
} else {
    Write-Host "⚠️  curl.exe not found, skipping this test" -ForegroundColor Yellow
}
Write-Host ""

# Summary
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "Test Summary" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "If all tests passed, your HTTP server is working!" -ForegroundColor Green
Write-Host "Access in browser: http://192.168.4.1" -ForegroundColor White
Write-Host ""
Write-Host "If you see HTTP 502 error:" -ForegroundColor Yellow
Write-Host "  • Rebuild firmware with latest fixes" -ForegroundColor White
Write-Host "  • Check debug output with probe-rs" -ForegroundColor White
Write-Host "  • See CONNECTIVITY_TEST.md for detailed help" -ForegroundColor White
Write-Host "==================================================" -ForegroundColor Cyan
