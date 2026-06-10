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
    MSG_NORMALIZED_RETURN,
    ROLE_COSMIC_B,
    ROLE_EARTH_A,
    ROTATIONS_1_7,
    Stats,
    TOPOLOGY_ROUTE,
    base_arg_parser,
    is_valid_1_7_rotation,
    is_valid_1_58_fragment_or_rotation,
    make_socket,
    next_nonce,
    route_reserved,
    role_name,
    start_receiver,
    verify_1_7_closure,
    verify_1_58_slices,
)

ROUTE_SEND_FORWARD = 1
WAIT_NORMALIZED_RETURN = 2


def build_packet(seq: int, msg_type: int, route_index: int) -> DsmcPacket:
    route_item = TOPOLOGY_ROUTE[route_index % len(TOPOLOGY_ROUTE)]
    layer_id = route_item["layer_id"]
    if layer_id == LAYER_1_7:
        phase = seq % 6
        cycle = ROTATIONS_1_7[phase]
    else:
        phase = route_index % 255
        cycle = route_item["cycle"]
    return DsmcPacket(
        msg_type=msg_type,
        device_role=ROLE_EARTH_A,
        layer_id=layer_id,
        phase=phase,
        seq=seq,
        nonce=next_nonce(),
        cycle=cycle,
        closure_tag="999999",
        reserved=route_reserved(route_index % len(TOPOLOGY_ROUTE)),
    )


def main() -> None:
    parser = base_arg_parser("DS-MCTI v0 local Device A: Earth Beacon")
    args = parser.parse_args()

    sock = make_socket(args.bind, args.port)
    rx_queue: "queue.Queue[tuple[DsmcPacket, tuple[str, int]]]" = queue.Queue()
    start_receiver(sock, rx_queue)

    stats = Stats(device_role="EARTH_A", boot_time=time.monotonic())
    seq_state = [0]
    route_index_state = [0]
    route_state = [ROUTE_SEND_FORWARD]
    last_tx = 0.0
    last_stats = time.monotonic()
    route_start_at = time.monotonic() + 0.5
    peer_port = args.peer_port if args.peer_port is not None else args.port
    target = (args.broadcast, peer_port)

    print(f"[A] DS-MCTI v0 local Earth Beacon ready port={args.port} target={target[0]}")
    route_text = "->".join(
        f"1/{item['layer_id']}(src=1/{item['source_denominator']},offset=2*29)" if item["layer_id"] == LAYER_1_58
        else f"1/{item['layer_id']}(src=1/{item['source_denominator']},p={item['period']})"
        for item in TOPOLOGY_ROUTE
    )
    print(f"[A] topology_route={route_text}=>1/7 peer_layer=58 peer_cycle={BASE_1_58}")

    while True:
        now = time.monotonic()
        if route_state[0] == ROUTE_SEND_FORWARD and now >= route_start_at and now - last_tx >= 0.100:
            route_index = route_index_state[0]
            seq_state[0] += 1
            packet = build_packet(seq_state[0], MSG_BEACON, route_index)
            sock.sendto(packet.pack(), target)
            stats.tx_count += 1
            next_state = "WAIT_NORMALIZED_RETURN" if packet.layer_id == LAYER_1_58 else "ROUTE_SEND_FORWARD"
            print(f"[A] TX route_step layer={packet.layer_id} next={next_state}")
            if packet.layer_id == LAYER_1_58:
                route_state[0] = WAIT_NORMALIZED_RETURN
            else:
                route_index_state[0] = (route_index_state[0] + 1) % len(TOPOLOGY_ROUTE)
            last_tx = now

        while True:
            try:
                packet, addr = rx_queue.get_nowait()
            except queue.Empty:
                break
            process_packet(packet, addr, sock, target, stats, seq_state, route_state, route_index_state)

        if now - last_stats >= args.stats_interval:
            print(stats.format_line("A"))
            last_stats = now

        time.sleep(0.005)


def process_packet(
    packet: DsmcPacket,
    addr,
    sock,
    target,
    stats: Stats,
    seq_state: list[int],
    route_state: list[int],
    route_index_state: list[int],
) -> None:
    stats.rx_count += 1

    if packet.magic != DSMC_MAGIC or packet.version != DSMC_VERSION:
        stats.invalid_packet_count += 1
        print(f"[A] RX role={role_name(packet.device_role)} layer={packet.layer_id} seq={packet.seq} verify_layer=FAIL reason=bad_magic_version")
        return

    if packet.device_role == ROLE_EARTH_A:
        return

    if route_state[0] == WAIT_NORMALIZED_RETURN:
        layer_ok = (
            packet.msg_type == MSG_NORMALIZED_RETURN
            and packet.device_role == ROLE_COSMIC_B
            and packet.layer_id == LAYER_1_7
        )
        rotation_ok, _ = is_valid_1_7_rotation(packet.cycle)
        closure_ok = rotation_ok and verify_1_7_closure(packet.cycle) and packet.closure_tag == "999999"
        if layer_ok and closure_ok:
            stats.valid_peer_count += 1
            stats.closure_pass_count += 1
            stats.phase_lock_count += 1
            stats.note_peer()
            print("[A] RX normalized_return role=COSMIC_B layer=7 verify_layer=PASS closure=PASS route_return=PASS")
            print("[A] CHAIN_CLOSURE_PASS route=1/7->1/17->1/19->1/23->1/29->1/47->1/58=>1/7")
            route_state[0] = ROUTE_SEND_FORWARD
            route_index_state[0] = 0
        else:
            reason = "bad_normalized_return"
            if packet.device_role == ROLE_COSMIC_B and packet.layer_id == LAYER_1_58:
                reason = "waiting_for_normalized_return"
                print(f"[A] RX role=COSMIC_B layer=58 seq={packet.seq} verify_layer=PASS reason={reason}")
                return
            stats.invalid_packet_count += 1
            print(f"[A] RX role={role_name(packet.device_role)} layer={packet.layer_id} seq={packet.seq} verify_layer={'PASS' if layer_ok else 'FAIL'} reason={reason}")
        return

    layer_ok = packet.device_role == ROLE_COSMIC_B and packet.layer_id == LAYER_1_58
    closure_ok = False
    reason = "none"
    if not layer_ok:
        reason = "bad_layer"
    elif not is_valid_1_58_fragment_or_rotation(packet.cycle):
        reason = "bad_cycle"
    else:
        closure_ok = verify_1_58_slices(packet.cycle)
        reason = "none" if closure_ok else "bad_closure"

    if layer_ok and closure_ok:
        stats.valid_peer_count += 1
        stats.closure_pass_count += 1
        stats.phase_lock_count += 1
        stats.note_peer()
        print(f"[A] RX role=COSMIC_B layer=58 seq={packet.seq} phase={packet.phase} verify_layer=PASS closure=PASS action=ECHO_SEEN")
    else:
        stats.invalid_packet_count += 1
        if layer_ok:
            stats.closure_fail_count += 1
        else:
            stats.phase_error_count += 1
        print(f"[A] RX role={role_name(packet.device_role)} layer={packet.layer_id} seq={packet.seq} verify_layer={'PASS' if layer_ok else 'FAIL'} reason={reason}")


if __name__ == "__main__":
    main()
