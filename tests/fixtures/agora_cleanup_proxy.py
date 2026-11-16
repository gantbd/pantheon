#!/usr/bin/env python3
"""Test-only proxy for refused-receipt and abrupt Agora shutdown cuts."""

import json
import os
import subprocess
import sys
import time


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: agora_cleanup_proxy.py CONFIG")
    executable = os.environ["WORKBENCH_AGORA_REAL_EXECUTABLE"]
    child = subprocess.Popen(
        [executable, sys.argv[1]],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
    )
    assert child.stdin is not None
    assert child.stdout is not None
    request = sys.stdin.buffer.read(1)
    mode = os.environ.get("WORKBENCH_AGORA_PROXY_MODE", "residue-receipt")
    if mode == "kill-on-shutdown":
        child.kill()
        child.wait()
        return 70
    if mode == "hang-after-child-kill":
        child.kill()
        child.wait()
        time.sleep(60)
        return 71
    if mode != "residue-receipt":
        raise SystemExit(f"unknown WORKBENCH_AGORA_PROXY_MODE: {mode}")
    if request:
        child.stdin.write(request)
        child.stdin.flush()
    child.stdin.close()
    receipt = child.stdout.read()
    status = child.wait()
    if status != 0:
        return status
    value = json.loads(receipt)
    value["cleanup"] = "residue-preserved"
    sys.stdout.write(json.dumps(value, separators=(",", ":")) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
