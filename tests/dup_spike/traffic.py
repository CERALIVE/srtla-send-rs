# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run by netns_dup_spike through python3 inside its isolated network namespaces.
"""Deterministic CBR source, byte-preserving UDP sink, and unregistered replay."""

import socket
import sys
import time
from collections.abc import Callable
from pathlib import Path
from typing import Final


def source(args: list[str]) -> None:
    root = Path(args[0])
    with (
        socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock,
        (root / "source.bin").open("wb") as output,
    ):
        start = time.monotonic()
        for index in range(3000):
            time.sleep(max(0, start + index / 200 - time.monotonic()))
            packet = index.to_bytes(4, "big") * 329
            output.write(packet)
            assert sock.sendto(packet, ("127.0.0.1", 4002)) == len(packet)


def sink(args: list[str]) -> None:
    with (
        socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock,
        (Path(args[0]) / "sink.bin").open("wb", buffering=0) as output,
    ):
        sock.bind(("127.0.0.1", 4100))
        while True:
            packet, _ = sock.recvfrom(65536)
            output.write(packet)


def replay(args: list[str]) -> None:
    data = (Path(args[0]) / "replay.bin").read_bytes()
    offset = 0
    sent = 0
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
        sock.bind((args[1], 49000))
        while offset < len(data):
            length = int.from_bytes(data[offset : offset + 2], "big")
            offset += 2
            packet = data[offset : offset + length]
            offset += length
            assert len(packet) == length
            assert sock.sendto(packet, (args[2], 5000)) == length
            sent += 1
            time.sleep(0.005)
    assert sent == 50


COMMANDS: Final[dict[str, Callable[[list[str]], None]]] = {
    "source": source,
    "sink": sink,
    "replay": replay,
}

if __name__ == "__main__":
    COMMANDS[sys.argv[1]](sys.argv[2:])
