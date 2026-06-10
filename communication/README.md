# DS-MCTI v0 Communication Demo

This folder contains a minimal two-device communication demo for DS-MCTI v0.

The demo validates a seven-layer path-dependent closure route:

`1/7 -> 1/17 -> 1/19 -> 1/23 -> 1/29 -> 1/47 -> 1/58 => 1/7`

- **Device A** sends the ordered route.  
- **Device B** verifies the route and sends a normalized return after the 1/58 return gate.  
- **Device A** accepts the result only when the returned 1/7 closure is verified.  

```

A successful run prints:



```text
CHAIN_CLOSURE_PASS

```

---

## Files

* `ds_mcti_common.py`
* `device_a_earth_beacon.py`
* `device_b_cosmic_echo.py`

---

## Requirements

* Python 3.10+ is recommended.
* No external Python package is required.

---

## Local Loopback Test

Open two terminals in this folder.

### Terminal A:

```bash
python device_a_earth_beacon.py --bind 127.0.0.1 --port 45859 --broadcast 127.0.0.1 --peer-port 45860 --stats-interval 1

```

### Terminal B:

```bash
python device_b_cosmic_echo.py --bind 127.0.0.1 --port 45860 --broadcast 127.0.0.1 --peer-port 45859 --stats-interval 1

```

---

## Two-Device LAN Test

### Environment Example:

* **Device A:** `192.168.1.10`
* **Device B:** `192.168.1.12`

### Run on Device A:

```bash
python device_a_earth_beacon.py --bind 192.168.1.10 --port 45859 --broadcast 192.168.1.12 --peer-port 45860 --stats-interval 1

```

### Run on Device B:

```bash
python device_b_cosmic_echo.py --bind 192.168.1.12 --port 45860 --broadcast 192.168.1.10 --peer-port 45859 --stats-interval 1

```

---

## Windows Firewall Rules

If the devices cannot communicate, you must allow UDP ports `45859-45860`. Run the following command on both devices with **Administrator** permissions:

```powershell
powershell -Command "New-NetFirewallRule -DisplayName 'DS-MCTI UDP 45859-45860' -Direction Inbound -Protocol UDP -LocalPort 45859-45860 -Action Allow"

```

### Optional ICMP Ping Rule:

```powershell
powershell -Command "New-NetFirewallRule -DisplayName 'Allow ICMPv4 Echo Request DS-MCTI' -Protocol ICMPv4 -IcmpType 8 -Direction Inbound -Action Allow"

```

---

## Expected Success Log

Device A should print logs similar to the following:

```text
[A] TX route_step layer=58 next=WAIT_NORMALIZED_RETURN
[A] RX normalized_return role=COSMIC_B layer=7 verify_layer=PASS closure=PASS route_return=PASS
[A] CHAIN_CLOSURE_PASS route=1/7->1/17->1/19->1/23->1/29->1/47->1/58=>1/7

```

---

## Notes

* This demo exposes the **closure behavior**, not the full closure-generation theory.
* It is intended to demonstrate that a context coordinate can be acquired and verified through a **path-dependent topological route** instead of a flat identifier or simple echo.
