# base-hil — **EXPERIMENTAL**

Template de probe HIL (host agent + gerador de firmware stub RP2350).

| Claim | Status |
|-------|--------|
| Compila no host sem hardware | ✅ `cargo test -p base-hil` |
| Enumerate USB real | ✅ feature opt-in `hil_usb` (rusb) — **não** no CI default |
| Flash automático sem probe | ❌ **`FlashDenied::NotDetected`** |
| Path Detected offline | ✅ `with_presence(Detected)` / `BASE_HIL_MOCK_DETECTED` |
| Dry-run flash (sem silício) | ✅ `with_mock_flash(Detected)` → `mock_dry_run` |
| Programador USB real | ❌ U3 — `ProgrammerUnimplemented` |
| Ligado ao `base pipeline` default | ❌ não |

## Enumerate (U2)

Ordem em `HilAgent::enumerate_presence(vid, pid)`:

1. `BASE_HIL_MOCK_DETECTED` set → `Detected` (sem USB)
2. Feature `hil_usb` + dispositivo USB presente → `Detected`
3. Caso contrário → `Simulated`

VID:PID canônico do stub: `0xCAFE:0x4007` (`DEFAULT_PROBE_VID` / `DEFAULT_PROBE_PID`).

```bash
# CI / default — zero libusb
cargo test -p base-hil
cargo build -p base-hil

# Máquina com libusb + probe (opt-in)
cargo test -p base-hil --features hil_usb
# Hardware real (ignorado no CI):
cargo test -p base-hil --features hil_usb -- --ignored
```

Deps de sistema para `hil_usb`: `libusb-1.0` (ex.: `libusb-1.0-0-dev` no Debian).

## Uso

```rust
use base_hil::{HilAgent, ProbePresence, DEFAULT_PROBE_PID, DEFAULT_PROBE_VID};

// CI / default
let a = HilAgent::connect(DEFAULT_PROBE_VID, DEFAULT_PROBE_PID)?; // Simulated
assert!(a.try_flash(&[0]).is_err());

// Offline Detected (testes) — programador real ainda ausente
let d = HilAgent::with_presence(ProbePresence::Detected);
assert!(d.try_flash(&[0]).is_err()); // ProgrammerUnimplemented

// Dry-run explícito (ainda ≠ silício)
let m = HilAgent::with_mock_flash(ProbePresence::Detected);
let receipt = m.try_flash(&[1, 2, 3])?;
assert_eq!(receipt.mode, "mock_dry_run");
```

Env opcional: `BASE_HIL_MOCK_DETECTED=1` faz `enumerate_presence` / `connect` retornar Detected **sem USB**.

## Requisitos futuros (U3+)

- Programador sob Detected sem `mock_flash`
- CLI `base hil …` (não existe)

← vault: [Sprint U2](../base-vault/15%20-%20Path%20to%20v0.5/15.12%20-%20Sprint%20U2%20HIL%20USB.md)
