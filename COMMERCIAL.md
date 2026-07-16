# B.A.S.E. — Estratégia Comercial

> [README.md](README.md) · **Estratégia Comercial**
>
> **Nota v1.3:** forense (Specter) + [Industrial Gate](base-vault/22%20-%20Path%20to%20v1.2/22.30%20-%20SOW%20Industrial%20Gate.md)
> + **HIL Lab Gate A** (`base hil lab-status`, [SOP](examples/hil_lab/SOP.md)) — lab-assist sob SOW;
> ainda ≠ production turnkey / PCB fab / auto-fix.
> Tags: [`v1.3.0-rc`](https://github.com/bmcc-DEV/B.A.S.E./releases/tag/v1.3.0-rc) · [`v1.2.0`](https://github.com/bmcc-DEV/B.A.S.E./releases/tag/v1.2.0).

---

## Mercados

| Mercado | Entrega |
|---------|---------|
| Forense | `run.sh` / `run_study.sh` |
| Industrial | Gate → HIL lab (v1.3) → PCB eng. / fix parcial (futuro) |
| SaaS | Adiado |

```bash
base hil lab-status --sop examples/hil_lab/SOP.md -o /tmp/hil_gate/
./examples/hil_lab/run_hil_lab.sh
```

## Próximo

1. ✅ `v1.2.0` Industrial Gate (docs)  
2. ✅ `v1.3.0-rc` HIL Lab Gate A  
3. Lab Cliente + SOW → `lab_assist_ready`  
4. Path PCB eng. (Gate B) sob demanda  
