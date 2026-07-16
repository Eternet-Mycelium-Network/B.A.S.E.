# HIL Lab SOP (Gate A3) — template

> Copiar/adaptar ao lab do Cliente. **Não** é flash de produção turnkey.

## 1. Operador

- Nome / contato: _______________
- Só flasheia com SOW §HIL assinado (Gate A5).

## 2. Pré-checks

```bash
base hil lab-status --sop examples/hil_lab/SOP.md -o /tmp/hil_gate/
# A1 Detected, A2 ALLOW_FLASH+CMD, A3 este ficheiro, A4 receipt≠production, A5 --sow-signed
```

## 3. Dry-run (obrigatório antes de silício)

```bash
base hil flash firmware.bin --mock-flash -o /tmp/hil/
# mode=mock_dry_run — zero silício
```

## 4. Lab-assist (só se Gate A verde)

```bash
export BASE_HIL_ALLOW_FLASH=1
export BASE_HIL_PROGRAMMER_CMD='picotool load {image}'   # exemplo
# build com --features hil_programmer,hil_usb conforme lab
base hil flash firmware.bin -o /tmp/hil/
# mode=experimental_external_cmd — ainda ≠ "production"
```

## 5. Rollback / log

- Guardar `hil_flash_receipt.json` + hash do binário.
- Se falhar: reverter imagem anterior documentada no SOW.

## 6. Proibido

- Flash no CI default  
- Claim `production` / SaaS plug-and-flash  
- Flash sem Detected / sem ALLOW_FLASH  

Ref: `base-vault/22 - Path to v1.2/22.30 - SOW Industrial Gate.md`
