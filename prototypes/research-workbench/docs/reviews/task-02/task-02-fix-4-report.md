# T02 — corrección 4: informe

Estado: corrección focal implementada y comprobada; lista para rerevisión del delta acumulado fix3+fix4.

- Rama/worktree: `agent/task-02-library` / `.worktrees/task-02-library`
- BASE fix4: `0c3799bf6e92fd5d1a588d15f26eb0493549d41d`
- Commit de implementación: `1a691c2aa67bb287bb75ca7262e773cd577b8b85`
- HEAD al cierre de implementación: `1a691c2aa67bb287bb75ca7262e773cd577b8b85`
- Archivos: `src-tauri/src/application/library.rs`, `src-tauri/tests/library_integration.rs`.

## Corrección

`valid_pending_cancellation` admite el sobre v1 conocido con `promotionConfirmed` ausente y devuelve `false` en ese caso. Si la propiedad está presente exige un booleano; conserva el rechazo de versiones, tipos, propiedades extra y enlaces request/token/receipt desconocidos o inconsistentes. La ausencia no concede autoridad para retirar un destino: el control con destino promovido pero sin flag conserva los bytes, deja cleanup PENDING y no emite receipt. No se modificó DTO/IPC, esquema, puertos ni normativa.

Las regresiones cubren retry y replay de una cancelación staging-only sin flag, recovery tras cierre y reapertura de SQLite, y unicidad de receipt/evento. Se verifica que no hay destino físico y que staging se retira. El negativo reabre una intención con destino presente y flag ausente: recovery la marca como incidencia, conserva el destino y no publica receipt. El control del wrapper desconocido también comprueba que `promotionConfirmed:"false"` se rechaza, junto con los negativos previos de versión 99 y payload/token discordante.

## Evidencia

- Rojo de comportamiento de recovery antes del cambio: `work/evidence/fix4-missing-flag-recovery-red.log`, exit 101; una cancelación PENDING sin flag produjo `recovered=0` en vez de `1`.
- Verde focal tras cierre/reapertura real: `fix4-missing-flag-reopen-green.log`, exit 0, 1 passed.
- Retry y replay staging-only: `fix4-missing-flag-retry-green.log`, exit 0, 1 passed; receipt y evento de entidad quedan exactamente en uno.
- Destino presente sin autoridad de flag: `fix4-missing-flag-target-control-green.log`, exit 0, 1 passed; bytes intactos, PENDING y sin receipt.
- Flag presente de tipo string rechazado, manteniendo los negativos v99 y token discordante: `fix4-invalid-promotion-boolean-green.log`, exit 0, 1 passed.

`fix4-missing-flag-retry-red.log` exit 101 no es rojo conductual: se ejecutó después del cambio y reveló que retry normaliza un wrapper válido a `promotionConfirmed:false`; la aserción inicial esperaba que se preservara la ausencia. Se corrigió la aserción para el contrato y el retry pasó. No se cuenta como fallo pre-cambio.

| Comando final | Resultado | Log |
|---|---:|---|
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts\check.ps1` | exit 0 | `work/evidence/fix4-final-check.log` |
| `. .\scripts\development-env.ps1; cargo build --manifest-path src-tauri\Cargo.toml` | exit 0 | `work/evidence/fix4-native-debug-build.log` |
| `git diff --check` | exit 0 | `work/evidence/fix4-diff-check.log` |

El gate final incluye TypeScript, Vitest 14/14, build Vite, format, Clippy y Rust. Rust: 102 entradas de runner (101 comprobaciones de comportamiento y el helper hijo `pending_wal_fixture_child`); library 55/55. `scripts/check.ps1` y build nativo se ejecutaron sobre el código final antes del commit. No se acredita selector GUI, instalador/NSIS ni instalación en equipo limpio. La revisión independiente del delta fix3+fix4 queda pendiente; estas pruebas no la sustituyen.
