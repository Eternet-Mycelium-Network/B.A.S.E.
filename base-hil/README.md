# base-hil — **EXPERIMENTAL**

Template de probe HIL (host agent + gerador de firmware stub RP2350).

| Claim | Status |
|-------|--------|
| Compila no host sem hardware | ✅ `cargo test -p base-hil` |
| Captura USB real / CMSIS-DAP | ❌ stub simulado |
| Flash automático sem probe | ❌ **rejeitado** por API |
| Ligado ao `base pipeline` default | ❌ não |

## Uso

```bash
cargo test -p base-hil
cargo build -p base-hil
```

O host agent (`HilAgent::connect`) abre em modo **simulado** até existir detecção real de probe.
`HilAgent::flash_probe_firmware` só sucede se `ProbePresence::Detected` — hoje sempre `Simulated`.

O ficheiro Rust embutido gerado por `ProbeFirmware::generate()` é um **esqueleto** (não um firmware certificados/`cargo build` thumbv8m no CI).

## Requisitos futuros (fora de v0.3)

- Probe aberto / CMSIS-DAP / VID:PID mapeado
- Target `thumbv8m.main-none-eabi` + `rp235x-hal` fora do workspace default
- CLI `base hil …` (não existe ainda)

← vault: [Sprint S4](../base-vault/13%20-%20Path%20to%20v0.3/13.14%20-%20Sprint%20S4%20HIL.md) · [HIL Cluster](../base-vault/09%20-%20B.A.S.E.%20v2%20Expansion/09.06%20-%20HIL%20Cluster.md)
