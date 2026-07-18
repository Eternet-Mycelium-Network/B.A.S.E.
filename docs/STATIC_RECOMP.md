# Static Recompilation (Path to v1.7)

Vault: [`base-vault/27 - Path to v1.7/`](../base-vault/27%20-%20Path%20to%20v1.7/27.00%20-%20Index.md)  
Crate: `base-recomp`

## Pipeline

```text
x86-32 bytes → lift → SIR → emit → ASM (x86_64|arm|aarch64|mips|ppc|sparc|sh2|sh4)
```

- **amd64** ≡ **x86_64**
- Honesty: `static_recomp_complete: false` · `win32_abi_complete: false` · `runs_any_pe: false`

## Smoke

```bash
cargo test -p base-recomp
base recomp lift --hex 90c3 --target x86_64 -o output/recomp_smoke
# R2 host roundtrip (Linux x86_64 + binutils):
base recomp roundtrip --hex B8010000000502000000C3 --name add3 --expect 3 -o output/r2
```

## R5

Goldens SuperH em `tests/goldens/sh_*.s`. Flavors: `--target sh2` (Saturn class) · `--target sh4` (Dreamcast class).

Path to v1.7 **R0–R5 complete** no código. Tag `v1.7.0-rc` no release.

## R4

Goldens MIPS / PPC / SPARC em `tests/goldens/{mips,ppc,sparc}_*.s`. Assemble live pendente de toolchain cross.

## R3

Goldens ARM / AArch64 em `tests/goldens/arm_*.s` e `aarch64_*.s`.  
Assemble-only: `base recomp assemble-arm` (usa `arm-none-eabi-as` ou `BASE_RECOMP_ARM_AS`).

## R2

Goldens em `base-recomp/tests/goldens/`. Roundtrip: lift → emit → `as --64` → `cc -no-pie` → harness verifica eax.

## Fora do norte v1.7

Win32 completo, Wine, runtime Saturn, “corre qualquer PE”.
