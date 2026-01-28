# Client Connection Guide - Pico 2W WiFi AP

## ⚠️ IMPORTANT: Manual IP Configuration Required

This WiFi Access Point does **NOT** have a DHCP server. You must manually configure your device's IP address.

---

## Quick Setup (3 Steps)

### Step 1: Connect to WiFi

**SSID:** `PicoLTE`  
**Password:** `12345678`  
**Security:** WPA2-PSK

---

### Step 2: Configure IP Address Manually

The Pico is at **192.168.4.1** - you need to pick a different address for your device.

#### On Windows

1. Open **Settings** → **Network & Internet** → **WiFi**
2. Click **PicoLTE** network → **Properties**
3. Under **IP assignment**, click **Edit**
4. Change to **Manual**
5. Enable **IPv4**
6. Enter:
   - **IP address:** `192.168.4.2` (or .3, .4, .5, etc.)
   - **Subnet mask:** `255.255.255.0`
   - **Gateway:** `192.168.4.1`
   - **DNS:** `192.168.4.1` (optional)
7. Click **Save**

#### On macOS

1. Open **System Preferences** → **Network**
2. Select **Wi-Fi** → **Advanced**
3. Go to **TCP/IP** tab
4. Set **Configure IPv4:** to **Manually**
5. Enter:
   - **IP Address:** `192.168.4.2`
   - **Subnet Mask:** `255.255.255.0`
   - **Router:** `192.168.4.1`
6. Click **OK** → **Apply**

#### On Linux

```bash
# Replace wlan0 with your WiFi interface name
sudo ip addr add 192.168.4.2/24 dev wlan0
sudo ip route add default via 192.168.4.1 dev wlan0
```

Or use NetworkManager:
```bash
nmcli con modify PicoLTE ipv4.addresses 192.168.4.2/24
nmcli con modify PicoLTE ipv4.gateway 192.168.4.1
nmcli con modify PicoLTE ipv4.method manual
nmcli con up PicoLTE
```

#### On iOS (iPhone/iPad)

1. Connect to **PicoLTE** WiFi
2. Tap the **(i)** icon next to the network name
3. Tap **Configure IP**
4. Select **Manual**
5. Enter:
   - **IP Address:** `192.168.4.2`
   - **Subnet Mask:** `255.255.255.0`
   - **Router:** `192.168.4.1`
6. Tap **Save**

#### On Android

1. Long-press **PicoLTE** → **Modify network**
2. Expand **Advanced options**
3. Set **IP settings** to **Static**
4. Enter:
   - **IP address:** `192.168.4.2`
   - **Gateway:** `192.168.4.1`
   - **Network prefix length:** `24`
5. Tap **Save**

---

### Step 3: Access the Proxy

Open your web browser to:

**http://192.168.4.1**

This will automatically load: **www.gzxxzlk.com**

To proxy other sites:

**http://192.168.4.1/proxy?url=http://example.com**

---

## Troubleshooting

### Problem: Can't connect to WiFi

**Check:**
- SSID is exactly: `PicoLTE` (case-sensitive)
- Password is: `12345678`
- WiFi is enabled on your device
- Pico LED is blinking (confirms running)

### Problem: Connected but can't ping 192.168.4.1

**Check:**
1. **Verify your IP is correct:**
   ```bash
   # Windows
   ipconfig
   
   # macOS/Linux
   ifconfig
   # or
   ip addr show
   ```
   
   Should show something like:
   ```
   inet 192.168.4.2  netmask 255.255.255.0
   ```

2. **Disable IPv6 temporarily:**
   
   IPv6 can interfere. Try disabling it on your network adapter.

3. **Check firewall:**
   
   Some firewalls block access to local networks.

4. **Try ping:**
   ```bash
   ping 192.168.4.1
   ```
   
   Should get responses with <10ms latency.

### Problem: Ping works but browser times out

**Check:**
1. Use **http://** not https://
   ```
   ✅ http://192.168.4.1
   ❌ https://192.168.4.1
   ```

2. Clear browser cache/cookies

3. Try different browser

4. Check browser proxy settings (disable any proxies)

5. Use **curl** to test:
   ```bash
   curl -v http://192.168.4.1
   ```

### Problem: IPv6 address showing instead of IPv4

**Symptoms:**
- Browser shows `http://[fe80::...]`
- Connection fails

**Solution:**
1. Disable IPv6 on your WiFi adapter
2. Or force IPv4 in browser:
   ```bash
   curl -4 http://192.168.4.1
   ```

---

## Valid IP Address Range

You can use any IP in the range **192.168.4.2** to **192.168.4.254**

Examples:
- ✅ `192.168.4.2`
- ✅ `192.168.4.10`
- ✅ `192.168.4.100`
- ❌ `192.168.4.1` (Pico's IP - don't use!)
- ❌ `192.168.4.0` (Network address)
- ❌ `192.168.4.255` (Broadcast address)

---

## Network Configuration Summary

| Setting | Value |
|---------|-------|
| Pico IP | `192.168.4.1` |
| Your IP | `192.168.4.2` (or higher) |
| Subnet Mask | `255.255.255.0` |
| Gateway | `192.168.4.1` |
| DNS | `192.168.4.1` (optional) |
| DHCP | ❌ Not available |

---

## Multiple Devices

You can connect multiple devices simultaneously. Each needs a **different IP address**:

- Device 1: `192.168.4.2`
- Device 2: `192.168.4.3`
- Device 3: `192.168.4.4`
- etc.

**Maximum:** Depends on Pico resources (typically 3-5 concurrent connections)

---

## Advanced: Using curl

### Test basic connectivity
```bash
curl http://192.168.4.1
```

### Test proxy function
```bash
curl "http://192.168.4.1/proxy?url=http://example.com"
```

### View response headers
```bash
curl -v http://192.168.4.1
```

### Set timeout
```bash
curl --max-time 10 http://192.168.4.1
```

---

## Debug Information

To see what's happening on the Pico:

```bash
# If you have probe-rs connected
probe-rs attach RP2350

# Look for these messages:
# "Network link is UP!"
# "Network stack ready"
# "Listening on TCP port 80..."
# "✅ Client connected from remote!"
```

---

## Security Note

⚠️ **This is an open WiFi network!**

- No WPA2/WPA3 password protection on the AP itself (the password is just for identification)
- Anyone in range can connect
- No encryption on HTTP traffic
- Suitable for testing/development only
- Do not transmit sensitive data

---

## Restoring Your Network Settings

After disconnecting, remember to:

1. **Switch back to DHCP:**
   - Windows: Change IP assignment to **Automatic**
   - macOS: Set Configure IPv4 to **Using DHCP**
   - Linux: `sudo dhclient <interface>`

2. **Or just forget the network:**
   - Remove/forget "PicoLTE" from saved networks
   - Your device will forget the manual IP settings

---

## Why No DHCP?

The Pico 2W is a microcontroller with limited resources:
- **RAM:** 512KB total
- **Running:** WiFi stack, HTTP server, UART handler, LTE modem
- **DHCP server** would add ~20-30KB RAM overhead
- Manual IP is simpler and more reliable for embedded systems

For production use, consider:
- Adding a DHCP server (if you have spare RAM)
- Using a more powerful router/gateway
- Using USB-to-Ethernet adapter with DHCP support

---

**Last Updated:** 2025-01-28  
**Status:** ✅ Tested and verified

**Need Help?** See HARDWARE_TROUBLESHOOTING.md