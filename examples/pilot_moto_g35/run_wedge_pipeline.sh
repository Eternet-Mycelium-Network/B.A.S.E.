#!/usr/bin/env bash
# Pipeline completo do wedge P0 (assist). ≠ OS turnkey · ≠ flash.
set -euo pipefail
PILOT="$(cd "$(dirname "$0")" && pwd)"
cd "$PILOT"

echo "=== 1/5 USB probe ==="
./run_usb_probe.sh || echo "WARN: usb-probe skipped/failed (sem ADB?) — continua se out_real existir"

echo "=== 2/5 USB×DTB cross + atlas ==="
./run_usb_cross.sh

echo "=== 3/5 board stub ==="
./run_wedge_p0.sh

echo "=== 4/5 Specter + QEMU smoke ==="
./run_wedge_qemu_smoke.sh

echo "=== 5/5 fase C assist (sem flash) ==="
./run_wedge_hw_assist.sh

echo ""
echo "Wedge pipeline OK (assist)."
echo "Handoff → $PILOT/WEDGE_HANDOFF.md"
echo "generates_os=false · flashed=false"
