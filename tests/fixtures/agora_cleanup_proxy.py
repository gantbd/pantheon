#!/usr/bin/env python3
"""Test-only proxy that turns a real clean Agora receipt into preserved residue."""

import json
import os
import subprocess
import sys


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
