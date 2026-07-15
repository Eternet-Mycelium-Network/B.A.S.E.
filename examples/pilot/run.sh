#!/usr/bin/env bash
# B.A.S.E. Pilot — UART MMIO wedge (synthetic) — Path to Real R0–R4
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BASE="${ROOT}/target/debug/base"
PILOT="$(cd "$(dirname "$0")" && pwd)"
OUT="${PILOT}/out"

if [[ ! -x "$BASE" ]]; then
  echo "Building base-cli…"
  (cd "$ROOT" && cargo build -p base-cli)
fi

rm -rf "$OUT"
mkdir -p "$OUT"

echo "== bir compile + validate =="
"$BASE" bir "$PILOT/pilot.bsl" --compile --validate -o "$OUT/bir"

echo "== analyze =="
"$BASE" analyze "$PILOT/fw.bin" \
  --mmio-traces "$PILOT/mmio.json" \
  --classify uart \
  -o "$OUT/analyze"

echo "== design =="
"$BASE" design "$OUT/analyze/hardware_spec.yaml" -o "$OUT/design"

echo "== synth =="
"$BASE" synth "$OUT/analyze/hardware_spec.yaml" --max-bom-cost 80 -o "$OUT/synth"

echo "== check (skip without new_trace) =="
"$BASE" check "$OUT/synth/synthesized_spec.yaml" "$PILOT/trace.csv" \
  --format json -o "$OUT/check_skip"
grep -q 'NO_NEW_TRACE' "$OUT/check_skip/validation_report.json"
grep -q '"comparison_mode": "skipped"' "$OUT/check_skip/validation_report.json"

echo "== check (dual: original vs slow) =="
"$BASE" check "$OUT/synth/synthesized_spec.yaml" "$PILOT/trace.csv" \
  "$PILOT/trace_slow.csv" --format json --max-latency 2.0 -o "$OUT/check_dual"
grep -q '"comparison_mode": "dual"' "$OUT/check_dual/validation_report.json"
grep -q 'TIMING_VIOLATION' "$OUT/check_dual/validation_report.json"

test -f "$OUT/analyze/tension_report.json"
grep -q 'overall_tension' "$OUT/analyze/tension_report.json"

echo "== prove (sat) =="
"$BASE" prove "$PILOT/contracts.yaml" -o "$OUT/prove"

echo "== prove (contracts from BIR) =="
"$BASE" prove "$OUT/bir/contracts.yaml" -o "$OUT/prove_bir"

echo "== prove (unsat fixture) =="
"$BASE" prove "$PILOT/contracts.unsat.yaml" -o "$OUT/prove_unsat"

echo "== replay (hand contracts) =="
"$BASE" replay "$PILOT/trace.csv" \
  --contracts "$PILOT/contracts.yaml" \
  --output "$OUT/violations.json"

echo "== replay (--bir) =="
"$BASE" replay "$PILOT/trace.csv" \
  --bir "$OUT/bir/compiled.bir.yaml" \
  --output "$OUT/violations_bir.json"

echo "== replay fail trace =="
"$BASE" replay "$PILOT/trace_fail.csv" \
  --contracts "$PILOT/contracts.yaml" \
  --output "$OUT/violations_fail.json" || true

echo "== event-graph =="
"$BASE" event-graph "$PILOT/contracts.yaml" "$PILOT/trace.csv" \
  --format dot -o "$OUT/event_graph"
"$BASE" event-graph "$PILOT/contracts.yaml" "$PILOT/trace.csv" \
  --format mermaid -o "$OUT/event_graph"
# Publish goldens when regenerating (checked in under expected/)
cp -f "$OUT/event_graph/event_graph.dot" "$PILOT/expected/event_graph.dot"
cp -f "$OUT/event_graph/event_graph.mmd" "$PILOT/expected/event_graph.mmd"

echo "== fw host =="
"$BASE" fw "$OUT/synth/synthesized_spec.yaml" -o "$OUT/fw"
make -C "$OUT/fw" host
"$OUT/fw/firmware_host"

echo
echo "Pilot smoke OK → $OUT"
echo "BIR: $OUT/bir/compiled.bir.yaml + contracts.yaml"
echo "Goldens: expected/event_graph.{dot,mmd}"
