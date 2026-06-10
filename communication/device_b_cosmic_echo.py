#!/usr/bin/env python3
from __future__ import annotations

import queue
import time

from ds_mcti_common import (
    BASE_1_58,
    DSMC_MAGIC,
    DSMC_VERSION,
    DsmcPacket,
    LAYER_1_7,
    LAYER_1_58,
    MSG_BEACON,
    MSG_CLOSURE_ACK,
    MSG_NORMALIZED_RETURN,
    MSG_REPLY,
    ROLE_COSMIC_B,
    ROLE_EARTH_A,
    ROTATIONS_1_7,
    Stats,
    TOPOLOGY_LAYERS,
    TOPOLOGY_PERIODS,
    TOPOLOGY_ROUTE,
    TOPOLOGY_SOURCE_DENOMINATORS,
    base_arg_parser,
    is_valid_1_7_rotation,
    make_socket,
    next_nonce,
    parse_route_reserved,
    route_index_for_layer,
    role_name,
    start_receiver,
    validate_topology_code,
    verify_1_7_closure,
)


def build_packet(seq: int, msg_type: int) -> DsmcPacket:
    return DsmcPacket(
        msg_type=msg_type,
        device_role=ROLE_COSMIC_B,
        layer_id=LAYER_1_58,
        phase=seq % len(BASE_1_58),
        seq=seq,
        nonce=next_nonce(),
        cycle=BASE_1_58,
        closure_tag="999999",
    )


def build_normalized_return(seq: int) -> DsmcPacket:
    return DsmcPacket(
        msg_type=MSG_NORMALIZED_RETURN,
        device_role=ROLE_COSMIC_B,
        layer_id=LAYER_1_7,
        phase=0,
        seq=seq,
        nonce=next_nonce(),
        cycle=ROTATIONS_1_7[0],
        closure_tag="999999",
    )


def main() -> None:
    parser = base_arg_parser("DS-MCTI v0 local Device B: Cosmic Echo")
    args = parser.parse_args()

    sock = make_socket(args.bind, args.port)
    rx_queue: "queue.Queue[tuple[DsmcPacket, tuple[str, int]]]" = queue.Queue()
    start_receiver(sock, rx_queue)

    stats = Stats(device_role="COSMIC_B", boot_time=time.monotonic())
    seq_state = [0]
    expected_route_index = [0]
    route_closure_count = [0]
    route_steps_seen = [0]
    last_tx = 0.0
    last_stats = time.monotonic()
    peer_port = args.peer_port if args.peer_port is not None else args.port
    target = (args.broadcast, peer_port)

    print(f"[B] DS-MCTI v0 local Cosmic Echo ready port={args.port} target={target[0]}")
    print(f"[B] layer=58 base_cycle={BASE_1_58} peer_layer=7")

    while True:
        now = time.monotonic()
        if now - last_tx >= 0.150:
            seq_state[0] += 1
            packet = build_packet(seq_state[0], MSG_BEACON)
            sock.sendto(packet.pack(), target)
            stats.tx_count += 1
            last_tx = now

        while True:
            try:
                packet, addr = rx_queue.get_nowait()
            except queue.Empty:
                break
            process_packet(
                packet,
                addr,
                sock,
                target,
                stats,
                seq_state,
                expected_route_index,
                route_closure_count,
                route_steps_seen,
            )

        if now - last_stats >= args.stats_interval:
            print(stats.format_line("B"))
            last_stats = now

        time.sleep(0.005)


def process_packet(
    packet: DsmcPacket,
    addr,
    sock,
    target,
    stats: Stats,
    seq_state: list[int],
    expected_route_index: list[int],
    route_closure_count: list[int],
    route_steps_seen: list[int],
) -> None:
    stats.rx_count += 1

    if packet.magic != DSMC_MAGIC or packet.version != DSMC_VERSION:
        stats.invalid_packet_count += 1
        print(f"[B] RX role={role_name(packet.device_role)} layer={packet.layer_id} seq={packet.seq} verify_layer=FAIL reason=bad_magic_version")
        return

    if packet.device_role == ROLE_COSMIC_B:
        return

    layer_ok = packet.device_role == ROLE_EARTH_A and packet.layer_id in TOPOLOGY_LAYERS
    route_index, route_len, declared_source_denominator, declared_period = parse_route_reserved(packet.reserved)
    expected_for_layer = route_index_for_layer(packet.layer_id)
    expected_period = TOPOLOGY_PERIODS[expected_for_layer] if expected_for_layer >= 0 else -1
    expected_source_denominator = TOPOLOGY_SOURCE_DENOMINATORS[expected_for_layer] if expected_for_layer >= 0 else -1
    resync_anchor = packet.layer_id == LAYER_1_7 and route_index == 0
    route_ok = (
        route_len == len(TOPOLOGY_ROUTE)
        and route_index == expected_for_layer
        and (route_index == expected_route_index[0] or resync_anchor)
        and declared_source_denominator == expected_source_denominator
        and declared_period == expected_period
    )
    code_ok = validate_topology_code(packet.layer_id, packet.cycle)
    rotation_ok, detected_phase = is_valid_1_7_rotation(packet.cycle)
    phase_ok = True
    if packet.layer_id == LAYER_1_7:
        phase_ok = rotation_ok and detected_phase == packet.phase
    closure_ok = code_ok

    reason = "none"
    if not layer_ok:
        reason = "bad_layer"
    elif not route_ok:
        reason = "bad_route"
    elif not code_ok:
        reason = "bad_cycle"
    elif not phase_ok:
        reason = "bad_phase"
    elif not closure_ok:
        reason = "bad_closure"

    if layer_ok and route_ok and code_ok and phase_ok and closure_ok:
        stats.valid_peer_count += 1
        stats.closure_pass_count += 1
        stats.phase_lock_count += 1
        stats.note_peer()
        if packet.msg_type == MSG_CLOSURE_ACK:
            print(f"[B] RX role=EARTH_A layer={packet.layer_id} seq={packet.seq} phase={packet.phase} route_index={route_index} verify_layer=PASS route=PASS closure=PASS action=ACK_SEEN")
        elif packet.layer_id == LAYER_1_58:
            print(f"[B] RX route_step expect=58 rx=58 verify=PASS normalize_to=7 action=SEND_NORMALIZED_1_7")
            seq_state[0] += 1
            normalized = build_normalized_return(seq_state[0])
            sock.sendto(normalized.pack(), addr)
            stats.tx_count += 1
            route_closure_count[0] += 1
            print(f"[B] TX normalized_return role=COSMIC_B layer=7 target={addr[0]}:{addr[1]}")
        else:
            print(f"[B] RX role=EARTH_A layer={packet.layer_id} seq={packet.seq} phase={packet.phase} route_index={route_index} verify_layer=PASS route=PASS closure=PASS action=ROUTE_STEP")

        if packet.msg_type != MSG_CLOSURE_ACK:
            route_steps_seen[0] += 1
            expected_route_index[0] = 0 if packet.layer_id == LAYER_1_58 else (route_index + 1) % len(TOPOLOGY_ROUTE)
    else:
        stats.invalid_packet_count += 1
        if layer_ok and code_ok:
            if not closure_ok:
                stats.closure_fail_count += 1
            if not phase_ok:
                stats.phase_error_count += 1
        print(f"[B] RX role={role_name(packet.device_role)} layer={packet.layer_id} seq={packet.seq} verify_layer={'PASS' if layer_ok else 'FAIL'} route={'PASS' if route_ok else 'FAIL'} reason={reason}")


if __name__ == "__main__":
    main()
