# Moto G35 real Firmware.zip — CASE SUMMARY

> Product: **ums9620 / QogirN6Pro** (Unisoc) · Android 14 stock (`UOA34.216-174-1`) · ≠ TaurOS complete

| Image | Blocks | Ψ | Confidence | Port package |
|-------|--------|---|------------|--------------|
| `analyze_lk` | 45 | 0.164 | 85.9% `conclusive_match` | wrap=35 rewrite=10 fossils=184 |
| `analyze_boot` | 53 | 0.303 | 76.8% `inconclusive` | wrap=53 rewrite=0 fossils=22 |
| `analyze_kernel` | 415 | 0.192 | 83.9% `inconclusive` | wrap=415 rewrite=0 fossils=12 |

## OS-port platform inventory (DTB)

Heurística MMIO **não basta** — classes obrigatórias: cpu, gic, arm_generic_timer, mmu, dram_controller, uart, gpio, pmic, storage_emmc_ufs, gpu_framebuffer, device_tree.

| Source | Readiness | CPU | Missing |
|--------|-----------|-----|---------|
| **`vendor_boot.img`** (board DT) | **100%** checklist | ARMv8.2-A / Cortex-A76 + Cortex-A55 | (none in checklist) |
| `dtbo.img` (overlay) | 36% | unknown (SoC id only) | gic, timer, dram, uart, gpio, pmic, gpu |

Achados em `vendor_boot` (primário para OS port):

- **GIC**: `arm,gic-v3`
- **UART**: `sprd,ums9620-uart` / `sprd,sc9836-uart`
- **Storage**: `sprd,ufshc-ums9620` + SDHCI
- **GPU/FB**: `sprd,mali-natt` + `sprd,qogirn6pro-dpu`
- **PMIC**: `sprd,ump9620-*` / `sprd,sc27xx-pd`
- **DRAM**: `sprd,pub-dmc`
- Stats: ~431 MMIO · 353 IRQs · 10 I2C · 4 SPI

Artefactos: `out_real/platform_vendor_boot/PLATFORM_INVENTORY.md` · `platform_dtbo/`

Nota: só `port_package_lk` recebe `--dtb` no `run_real_fw.sh`; boot/kernel usam os passos `port platform` separados (preferir **vendor_boot**).

## Filogenia validada (lk / boot / kernel)

- Newick: `(analyze_lk:0.253652,(analyze_boot:0.117143,analyze_kernel:0.096798)n3:0.253652);`
- `analyze_boot`↔`analyze_kernel`: d_tree=0.214 d_φ=0.188 J_anc=0.750 Φ=0.853 shared_anc=3
- `analyze_lk`↔`analyze_kernel`: d_tree=0.604 d_φ=0.481 J_anc=0.150 Φ=0.853 shared_anc=1
- `analyze_lk`↔`analyze_boot`: d_tree=0.624 d_φ=0.557 J_anc=0.150 Φ=0.794 shared_anc=1
- Achado: páginas exactas J=0; bandas SoC ligam **boot↔kernel** (menor d_tree); LK = estrato lowmap (especiação/plasticidade).
- NJ usa Ψ híbrido; d_φ anota relógio molecular suave.
- Artefactos: `out_real/phylo/` (gitignored)

## Primary atlas (usar primeiro)

`examples/pilot_moto_g35/out_real/port_package_lk/`

- Capstone MMIO real no **lk-sign.bin** (Little Kernel)
- Ψ **ConclusiveMatch** (~86%)
- 35 wrap / 10 rewrite / 184 fossils
- + `PLATFORM_INVENTORY.md` quando `--dtb` (DTBO overlay incompleto — preferir vendor_boot)

## Reproduzir

```bash
./examples/pilot_moto_g35/run_real_fw.sh
# ou só inventário / filogenia:
base port platform examples/pilot_moto_g35/real_fw/vendor_boot.img \
  --flash-cfg examples/pilot_moto_g35/real_fw/flash.cfg \
  -o examples/pilot_moto_g35/out_real/platform_vendor_boot
base paleo phylo out_real/analyze_{lk,boot,kernel}/evidence_db.yaml \
  --delta-t 1 --delta-t 2 --delta-t 3 -o out_real/phylo/
```

## Honesty

- Checklist 100% no DTB ≠ OS bootável / TaurOS turnkey
- Filogenia ≠ prova de plágio
- `generates_os: false` · `Firmware.zip` / `real_fw/` gitignored
- status: **OK**
