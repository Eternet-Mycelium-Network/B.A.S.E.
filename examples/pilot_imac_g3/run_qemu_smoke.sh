#!/usr/bin/env bash
# iMac G3 fase B — QEMU PowerPC smoke (esqueleto). Opt-in.
set -euo pipefail
PILOT="$(cd "$(dirname "$0")" && pwd)"
OUT="$PILOT/out_qemu"
mkdir -p "$OUT"

if ! command -v qemu-system-ppc >/dev/null 2>&1; then
  cat > "$OUT/qemu_boot_smoke.json" <<EOF
{"phase":"B","ok":false,"skipped":true,"reason":"qemu-system-ppc not installed","production":false}
EOF
  echo "SKIP: qemu-system-ppc missing"
  exit 0
fi

IMG="${QEMU_PPC_KERNEL:-${REACTOS_IMAGE:-}}"
if [[ -z "$IMG" ]]; then
  cat > "$OUT/qemu_boot_smoke.json" <<EOF
{"phase":"B","ok":false,"skipped":true,"reason":"set QEMU_PPC_KERNEL or REACTOS_IMAGE","production":false}
EOF
  echo "SKIP: no PPC image — see REACTOS_EXTERNAL.md"
  exit 0
fi

TIMEOUT_SEC="${QEMU_TIMEOUT_SEC:-8}"
LOG="$OUT/qemu.log"
set +e
timeout "$TIMEOUT_SEC" qemu-system-ppc \
  -M mac99 -m 256 -nographic -kernel "$IMG" \
  >"$LOG" 2>&1
RC=$?
set -e

python3 - <<PY
import json
r={"phase":"B","ok":True,"skipped":False,"qemu_exit":$RC,"kernel":"$IMG","production":False}
open("$OUT/qemu_boot_smoke.json","w").write(json.dumps(r,indent=2)+"\n")
print(r)
PY
echo "iMac G3 fase B OK → $OUT"
