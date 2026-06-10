#!/usr/bin/env python3
"""
DS-MCTI v0 laptop transport common code.

This mirrors the ESP32 fixed-size packet layout with Python struct packing.
When run on two laptops on the same L2/L3 network, packets are real UDP
broadcast traffic between machines, not an in-process simulation.
"""

from __future__ import annotations

import argparse
import os
import queue
import socket
import struct
import threading
import time
from dataclasses import dataclass
from typing import Optional, Tuple


DSMC_MAGIC = 0x44534D43
DSMC_VERSION = 1

MSG_BEACON = 1
MSG_REPLY = 2
MSG_CLOSURE_ACK = 3
MSG_NORMALIZED_RETURN = 4

ROLE_EARTH_A = 1
ROLE_COSMIC_B = 2

LAYER_1_7 = 7
LAYER_1_58 = 58
LAYER_1_59 = 59

BROADCAST_HOST = "255.255.255.255"
DEFAULT_PORT = 45858

ROTATIONS_1_7 = [
    "142857",
    "285714",
    "428571",
    "571428",
    "714285",
    "857142",
]
BASE_1_58 = "1724137931034482758620689655"

TOPOLOGY_ROUTE = [
    {"layer_id": 7, "source_denominator": 7, "period": 6, "cycle": "142857"},
    {"layer_id": 17, "source_denominator": 17, "period": 16, "cycle": "0588235294117647"},
    {"layer_id": 19, "source_denominator": 19, "period": 18, "cycle": "052631578947368421"},
    {"layer_id": 23, "source_denominator": 23, "period": 22, "cycle": "0434782608695652173913"},
    {"layer_id": 29, "source_denominator": 29, "period": 28, "cycle": "0344827586206896551724137931"},
    {"layer_id": 47, "source_denominator": 47, "period": 46, "cycle": "0212765957446808510638297872340"},
    # Wireless route layer 58 is the Cosmic Echo normalization layer.
    {"layer_id": 58, "source_denominator": 58, "period": 58, "cycle": BASE_1_58},
]
TOPOLOGY_CODES = {
    item["layer_id"]: item["cycle"] for item in TOPOLOGY_ROUTE
}
TOPOLOGY_LAYERS = [item["layer_id"] for item in TOPOLOGY_ROUTE]
TOPOLOGY_SOURCE_DENOMINATORS = [item["source_denominator"] for item in TOPOLOGY_ROUTE]
TOPOLOGY_PERIODS = [item["period"] for item in TOPOLOGY_ROUTE]

# Matches Arduino struct field order:
# uint32, 5x uint8, 2x uint32, uint8, char[32], char[16], int8, char[8]
PACKET_STRUCT = struct.Struct("<IBBBBBIIB32s16sb8s")


@dataclass
class DsmcPacket:
    magic: int = DSMC_MAGIC
    version: int = DSMC_VERSION
    msg_type: int = MSG_BEACON
    device_role: int = ROLE_EARTH_A
    layer_id: int = LAYER_1_7
    phase: int = 0
    seq: int = 0
    nonce: int = 0
    cycle_len: int = 0
    cycle: str = ""
    closure_tag: str = "999999"
    rssi_placeholder: int = 0
    reserved: bytes = b"\x00" * 8

    def pack(self) -> bytes:
        cycle_bytes = self.cycle.encode("ascii")[:31]
        closure_bytes = self.closure_tag.encode("ascii")[:15]
        return PACKET_STRUCT.pack(
            self.magic,
            self.version,
            self.msg_type,
            self.device_role,
            self.layer_id,
            self.phase,
            self.seq,
            self.nonce,
            len(cycle_bytes),
            cycle_bytes.ljust(32, b"\x00"),
            closure_bytes.ljust(16, b"\x00"),
            self.rssi_placeholder,
            self.reserved[:8].ljust(8, b"\x00"),
        )

    @classmethod
    def unpack(cls, data: bytes) -> "DsmcPacket":
        if len(data) != PACKET_STRUCT.size:
            raise ValueError(f"bad packet size {len(data)} != {PACKET_STRUCT.size}")
        fields = PACKET_STRUCT.unpack(data)
        cycle_raw = fields[9].split(b"\x00", 1)[0]
        closure_raw = fields[10].split(b"\x00", 1)[0]
        return cls(
            magic=fields[0],
            version=fields[1],
            msg_type=fields[2],
            device_role=fields[3],
            layer_id=fields[4],
            phase=fields[5],
            seq=fields[6],
            nonce=fields[7],
            cycle_len=fields[8],
            cycle=cycle_raw.decode("ascii", errors="ignore"),
            closure_tag=closure_raw.decode("ascii", errors="ignore"),
            rssi_placeholder=fields[11],
            reserved=fields[12],
        )


@dataclass
class Stats:
    device_role: str
    boot_time: float
    tx_count: int = 0
    rx_count: int = 0
    valid_peer_count: int = 0
    invalid_packet_count: int = 0
    closure_pass_count: int = 0
    closure_fail_count: int = 0
    phase_lock_count: int = 0
    phase_error_count: int = 0
    last_peer_seen_ms: int = 0
    discovery_latency_ms: int = 0

    def uptime_ms(self) -> int:
        return int((time.monotonic() - self.boot_time) * 1000)

    def note_peer(self) -> None:
        now_ms = self.uptime_ms()
        self.last_peer_seen_ms = now_ms
        if self.discovery_latency_ms == 0:
            self.discovery_latency_ms = now_ms

    def format_line(self, prefix: str) -> str:
        return (
            f"[{prefix}] STATS device_role={self.device_role} "
            f"uptime_ms={self.uptime_ms()} tx_count={self.tx_count} rx_count={self.rx_count} "
            f"valid_peer_count={self.valid_peer_count} invalid_packet_count={self.invalid_packet_count} "
            f"closure_pass_count={self.closure_pass_count} closure_fail_count={self.closure_fail_count} "
            f"phase_lock_count={self.phase_lock_count} phase_error_count={self.phase_error_count} "
            f"last_peer_seen_ms={self.last_peer_seen_ms} discovery_latency_ms={self.discovery_latency_ms}"
        )


def role_name(role: int) -> str:
    if role == ROLE_EARTH_A:
        return "EARTH_A"
    if role == ROLE_COSMIC_B:
        return "COSMIC_B"
    return "UNKNOWN"


def is_valid_1_7_rotation(cycle: str) -> Tuple[bool, Optional[int]]:
    for phase, rotation in enumerate(ROTATIONS_1_7):
        if cycle == rotation:
            return True, phase
    return False, None


def verify_1_7_closure(cycle: str) -> bool:
    if len(cycle) != 6 or not cycle.isdigit():
        return False
    value = int(cycle)
    if value * 7 == 999999:
        return True
    ok, _ = is_valid_1_7_rotation(cycle)
    return ok and 142857 * 7 == 999999


def verify_9_complement_6(a: str, b: str) -> bool:
    if len(a) != 6 or len(b) != 6 or not a.isdigit() or not b.isdigit():
        return False
    return all(int(x) + int(y) == 9 for x, y in zip(a, b))


def contains_slice(haystack: str, needle: str) -> bool:
    return needle in haystack


def contains_cyclic_slice(haystack: str, needle: str) -> bool:
    if not haystack or not needle or len(needle) > len(haystack):
        return False
    doubled = haystack + haystack[: len(needle) - 1]
    return needle in doubled


def verify_1_58_slices(cycle: str) -> bool:
    pairs = [
        ("172413", "827586"),
        ("413793", "586206"),
        ("344827", "655172"),
    ]
    passes = 0
    for a, b in pairs:
        if (
            contains_cyclic_slice(cycle, a)
            and contains_cyclic_slice(cycle, b)
            and verify_9_complement_6(a, b)
        ):
            passes += 1
    return passes >= 2


def route_index_for_layer(layer_id: int) -> int:
    try:
        return TOPOLOGY_LAYERS.index(layer_id)
    except ValueError:
        return -1


def validate_topology_code(layer_id: int, cycle: str) -> bool:
    if layer_id == 7:
        ok, _ = is_valid_1_7_rotation(cycle)
        return ok and verify_1_7_closure(cycle)
    if layer_id == LAYER_1_58:
        return is_valid_1_58_fragment_or_rotation(cycle) and verify_1_58_slices(cycle)
    expected = TOPOLOGY_CODES.get(layer_id)
    return expected is not None and cycle == expected


def route_reserved(path_index: int) -> bytes:
    item = TOPOLOGY_ROUTE[path_index % len(TOPOLOGY_ROUTE)]
    return bytes([
        path_index & 0xFF,
        len(TOPOLOGY_ROUTE) & 0xFF,
        item["source_denominator"] & 0xFF,
        item["period"] & 0xFF,
    ]) + b"\x00" * 4


def parse_route_reserved(reserved: bytes) -> tuple[int, int, int, int]:
    if len(reserved) < 4:
        return -1, 0, 0, 0
    return reserved[0], reserved[1], reserved[2], reserved[3]


def is_valid_1_58_fragment_or_rotation(cycle: str) -> bool:
    if not cycle or len(cycle) > len(BASE_1_58):
        return False
    return contains_cyclic_slice(BASE_1_58, cycle)


def make_socket(bind_host: str, port: int) -> socket.socket:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
    try:
        sock.bind((bind_host, port))
    except OSError:
        sock.bind(("", port))
    sock.settimeout(0.2)
    return sock


def start_receiver(sock: socket.socket, rx_queue: "queue.Queue[tuple[DsmcPacket, tuple[str, int]]]") -> threading.Thread:
    def recv_loop() -> None:
        while True:
            try:
                data, addr = sock.recvfrom(512)
            except socket.timeout:
                continue
            except OSError:
                break
            try:
                rx_queue.put((DsmcPacket.unpack(data), addr))
            except ValueError:
                continue

    thread = threading.Thread(target=recv_loop, daemon=True)
    thread.start()
    return thread


def next_nonce() -> int:
    return int.from_bytes(os.urandom(4), "little")


def base_arg_parser(description: str) -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("--bind", default="0.0.0.0", help="Local bind address. Default: 0.0.0.0")
    parser.add_argument("--broadcast", default=BROADCAST_HOST, help="Broadcast/peer host. Default: 255.255.255.255")
    parser.add_argument("--port", type=int, default=DEFAULT_PORT, help=f"UDP port. Default: {DEFAULT_PORT}")
    parser.add_argument("--peer-port", type=int, default=None, help="Peer UDP port. Default: same as --port")
    parser.add_argument("--stats-interval", type=float, default=5.0, help="Stats interval seconds. Default: 5")
    return parser
