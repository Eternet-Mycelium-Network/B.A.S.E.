#!/usr/bin/env bash
# X3 — USART1 + I2C1 no mesmo STM32 (opt-in). NÃO substitui run.sh / run_w1_spi.sh.
#
# I2C1 @ 0x40005400 (APB1) → page 0x40005000 (≠ USART1 / SPI2).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BASE="${ROOT}/target/debug/base"
PILOT="$(cd "$(dirname "$0")" && pwd)"
OUT="${PILOT}/out_x3_i2c"
PREF="STMicroelectronics"
CLASSIFY="0x40013000=uart,0x40005000=i2c"

if [[ ! -x "$BASE" ]]; then
  echo "Building base-cli…"
  (cd "$ROOT" && cargo build -p base-cli)
fi

echo "== X3 STM32 USART+I2C fixture integrity =="
(cd "$PILOT" && sha256sum -c SHA256SUMS.x3)

rm -rf "$OUT"
mkdir -p "$OUT"

echo "== bir I2C1 =="
"$BASE" bir "$PILOT/pilot_i2c.bsl" --compile --validate -o "$OUT/bir_i2c"

echo "== analyze dual (classify per page) =="
"$BASE" analyze "$PILOT/fw.bin" \
  --mmio-traces "$PILOT/mmio_usart_i2c.json" \
  --classify "$CLASSIFY" \
  -o "$OUT/analyze"

grep -E 'base_address: (1073819648|0x40013000)' "$OUT/analyze/hardware_spec.yaml" >/dev/null
grep -E 'base_address: (1073762304|0x40005000)' "$OUT/analyze/hardware_spec.yaml" >/dev/null
grep -Eqi 'kind:[[:space:]]*(Uart|uart)' "$OUT/analyze/hardware_spec.yaml"
grep -Eqi 'kind:[[:space:]]*(I2c|i2c)' "$OUT/analyze/hardware_spec.yaml"

echo "== design (prefer ST) =="
"$BASE" design "$OUT/analyze/hardware_spec.yaml" \
  --preferred-manufacturer "$PREF" \
  --max-bom-cost 80 \
  -o "$OUT/design"
grep -q 'STM32F103C8' "$OUT/design/reference_design.yaml"

echo "== synth (prefer ST) =="
"$BASE" synth "$OUT/analyze/hardware_spec.yaml" \
  --preferred-manufacturer "$PREF" \
  --max-bom-cost 80 \
  -o "$OUT/synth"
grep -q 'STM32F103C8' "$OUT/synth/synthesized_spec.yaml"
grep -Eqi 'interface:[[:space:]]*uart|"uart"|uart' "$OUT/synth/synthesized_spec.yaml"
grep -Eqi 'interface:[[:space:]]*i2c|"i2c"|i2c' "$OUT/synth/synthesized_spec.yaml"

echo "== prove I2C1 contracts =="
"$BASE" prove "$PILOT/contracts_i2c.yaml" -o "$OUT/prove_i2c"

echo "== replay I2C1 =="
"$BASE" replay "$PILOT/trace_i2c.csv" \
  --contracts "$PILOT/contracts_i2c.yaml" \
  --output "$OUT/violations_i2c.json"

echo "== CASE_SUMMARY_X3 =="
python3 - "$OUT" <<'PY'
import pathlib, sys, re
out = pathlib.Path(sys.argv[1])
design = (out / "design" / "reference_design.yaml").read_text()
synth = (out / "synth" / "synthesized_spec.yaml").read_text()
assert "STM32F103C8" in design
assert re.search(r"(?i)uart", synth), "synth missing uart"
assert re.search(r"(?i)i2c", synth), "synth missing i2c"
summary = out / "CASE_SUMMARY_X3.md"
summary.write_text(
    "# X3 STM32 CASE SUMMARY\n\n"
    "- Dual wedge: USART1 @ 0x40013800 + I2C1 @ 0x40005400\n"
    "- Classify: `0x40013000=uart,0x40005000=i2c`\n"
    "- Prefer manufacturer: STMicroelectronics → STM32F103C8\n"
    "- Gates USART (`run.sh`) e SPI (`run_w1_spi.sh`) intocados\n"
    f"- design bytes: {len(design)}\n"
    "- status: OK\n"
)
print(summary.read_text())
PY

echo "X3 STM32 I2C smoke OK → $OUT"
