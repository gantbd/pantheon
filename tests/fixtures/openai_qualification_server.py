#!/usr/bin/env python3
"""Bounded loopback OpenAI-compatible fixture for provider qualification."""

import json
import socket
import sys
from pathlib import Path


port_path = Path(sys.argv[1])
count_path = Path(sys.argv[2])
if len(sys.argv) > 4:
    assert Path(sys.argv[4]).read_bytes() == b"hestia fixture model"
listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
listener.bind(("127.0.0.1", int(sys.argv[3]) if len(sys.argv) > 3 else 0))
listener.listen(4)
port_path.write_text(str(listener.getsockname()[1]), encoding="ascii")
count = 0

while True:
    connection, _ = listener.accept()
    with connection:
        request = b""
        while b"\r\n\r\n" not in request:
            chunk = connection.recv(4096)
            if not chunk:
                request = b""
                break
            request += chunk
        # Hestia's bounded TCP-accept readiness basis deliberately proves only
        # that the listener accepted a connection. It is not an HTTP request.
        if not request:
            continue
        headers, body = request.split(b"\r\n\r\n", 1)
        content_length = 0
        for line in headers.split(b"\r\n"):
            if line.lower().startswith(b"content-length:"):
                content_length = int(line.split(b":", 1)[1].strip())
        while len(body) < content_length:
            chunk = connection.recv(4096)
            if not chunk:
                raise RuntimeError("qualification request ended before body")
            body += chunk
        payload = json.loads(body[:content_length])
        assert payload["model"] == "qwen/qwen3-coder-next"
        assert payload["stream"] is False
        count += 1
        count_path.write_text(str(count), encoding="ascii")
        response = json.dumps(
            {
                "choices": [
                    {
                        "message": {"content": "bounded qualification complete"},
                        "finish_reason": "stop",
                    }
                ],
                "usage": {"prompt_tokens": 17, "completion_tokens": 4},
            },
            separators=(",", ":"),
        ).encode("utf-8")
        connection.sendall(
            b"HTTP/1.1 200 OK\r\n"
            b"Content-Type: application/json\r\n"
            + f"Content-Length: {len(response)}\r\n".encode("ascii")
            + b"Connection: close\r\n\r\n"
            + response
        )
