#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> build apps"
cargo build -p signbridge-cli -p signbridge-host

echo "==> CLI ping"
cargo run -q -p signbridge-cli -- ping

echo "==> native messaging ping"
python3 - <<'PY'
import json, struct, subprocess, sys
msg = json.dumps({"id": "1", "method": "ping"}).encode()
proc = subprocess.Popen(
    ["cargo", "run", "-q", "-p", "signbridge-host"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
)
proc.stdin.write(struct.pack("<I", len(msg)) + msg)
proc.stdin.close()
n = struct.unpack("<I", proc.stdout.read(4))[0]
print(proc.stdout.read(n).decode())
PY

echo "==> apps smoke passed"
