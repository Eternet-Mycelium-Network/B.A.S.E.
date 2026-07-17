# Handoff — Wedge P0 G35 → tree externo

> Assist B.A.S.E. **termina aqui**. O port do OS vive noutro repositório.
> `generates_os: false` · ≠ TaurOS turnkey · ≠ earlycon verificado no silício.

## Bases absolutas (ums9620 / manila)

| Periférico | Base | Origem |
|------------|------|--------|
| UART0 | `0x20200000` | USB `20200000.serial` |
| GICD (GICv3) | `0x12000000` | DT `reg[0]` (`#address-cells=2`) |
| GICR (GICv3) | `0x12040000` | DT `reg[1]` (size `0x100000` no vendor) |
| UFS | `0x22000000` | USB `22000000.ufs` |

Arch timer = CNT* (sem MMIO no atlas).

## Artefactos a copiar

Gerar (telefone ADB opcional para USB; atlas/stub usam `out_real` se já existir):

```bash
./examples/pilot_moto_g35/run_wedge_pipeline.sh
```

Levar para o tree externo:

| Ficheiro | Uso |
|----------|-----|
| `out_real/wedge_p0/board-ums9620-wedge-p0.dtsi` | fragmento DT (GICD+GICR) |
| `out_real/clocks_pinctrl/board-ums9620-wedge-clocks-pinctrl.dtsi` | clocks/pinctrl hints |
| `out_real/wedge_p0/cmdline_earlycon.txt` | candidatos `earlycon=` |
| `out_real/wedge_p0/hal_wedge_p0.[ch]` | stub host / referência |
| `out_real/usb_cross/wedge_mmio_map.yaml` | mapa P0 machine-readable |
| `out_real/wedge_specter/` | Specter twin + QMP live (≠ ums9620) |
| `out_real/wedge_hw/PHASE_C_CHECKLIST.md` | lab |

## Trabalho no tree externo (ordem)

1. Integrar DTSI (ou só cmdline) no board teu
2. Resolver phandles `clocks=` / pinctrl UART a partir de `CLOCKS_PINCTRL.md` + vendor DT
3. Completar `#redistributor-regions` no GICv3
4. Build Image / boot.img
5. Flash **manual** (fastboot/EDL) — nunca CI default
6. Preencher `hw_boot_receipt.json` (`result`, `image_sha256`)

## O que o B.A.S.E. não faz

- Compilar/bootar o OS alvo
- Garantir earlycon no telefone
- Modem / GPU / TrustZone

Vault: [[24.41]]–[[24.44]] · SOP.md
