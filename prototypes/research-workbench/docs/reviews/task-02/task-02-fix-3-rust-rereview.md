# T02 fix3 — rerevisión Rust focal

Estado: **DONE_WITH_CONCERNS**. Conformidad: **NOT APPROVED**. Calidad Rust: **BLOCK** por una regresión Important. F3 y F7 ADDRESSED; C1 parcialmente corregido, **NOT ADDRESSED** en conjunto. Sin Critical ni otros problemas concretos encontrados en el delta.

BASE `21fd14ed20ae391518a7dabaaece3dc551a92fa2` → HEAD `0c3799bf6e92fd5d1a588d15f26eb0493549d41d`. Revisor `/root/task02_rust_review`. Checkout `.worktrees/task-02-library`, limpio antes y después de comprobaciones. Lectura completa, una vez en porciones, del paquete `review-21fd14e..0c3799b.diff` (833 líneas, 5 archivos, dos commits), incluido el código y pruebas; el informe del autor no se usó como evidencia única. Brief fix3 y normativa principal MANAGED_FILES C1 contrastados. Resto de hallazgos previos conservados cerrados. Sin código, configuración, pruebas nuevas, commits ni subagentes; única escritura este informe.

## Seguimiento

| Hallazgo | Veredicto | Evidencia |
|---|---|---|
| F3 — finalización de cancelación en recovery sin evento de entidad | **ADDRESSED** | `library_repository.rs:1034` valida el snapshot de cancelación y requestId; el UPDATE exige el mismo error_json vigente. `:1077` inserta `import.cancelled` con operationId/requestId en la misma acción de with_receipt que cambia PENDING a DONE. La prueba existente de reapertura ahora inyecta un trigger que falla la auditoría: archivo retirado pero estado PENDING, sin receipt/evento; al retirar el trigger, recovery y replay producen exactamente un evento y un receipt. Esa prueba pasó en la suite propia. |
| C1 — wrapper desconocido normalizado antes de validar autoridad | **NOT ADDRESSED** en conjunto | El defecto original se corrige: FAILED se valida dentro de TX antes de escribir, y `has_durable_promoted_confirmation` usa el validador común. Los negativos version99 y token discordante conservan bytes/wrapper y no publican receipt; ambos subcasos pasan. El validador introduce la regresión Important I1 descrita abajo. |
| F7 — DONE temprano sin demostrar staging ausente | **ADDRESSED** | `store.rs:192` comprueba ausencia mediante apertura administrada después de validar UUID canónico. Archivo existente o error de inspección implican PENDING; no borra recurso desconocido. UUID inválido retorna PENDING sin explorar namespace. Prueba de servicio `library_integration.rs:1644` coloca bytes tras begin_import mediante barrera, comprueba error original SourceUnreadable y FAILED/PENDING, luego reapertura mantiene bytes e incidencia. Prueba de ID inválido `store.rs:536` y control vacío `source_unreadable` pasan. |

## Important I1 — C1 rechaza una cancelación v1 válida sin flag de promoción

**Ubicación:** `src-tauri/src/application/library.rs:150`, `:165`, `:187`; consumidores `src-tauri/src/adapters/sqlite/library_repository.rs:731`, `:835`, `:1044`.

`valid_pending_cancellation` exige un conjunto exacto de claves que incluye promotionConfirmed y luego exige que su valor sea booleano. Si ese campo falta, retorna None aunque el sobre tenga version1, kind cancellation, código, cleanup PENDING, requestId canónico y receipt/token correctos.

La norma vigente `docs/development/MANAGED_FILES.md:51` permite explícitamente promotionConfirmed **false/ausente** fuera de promoción. Un PENDING válido sin el campo, con solo staging propio o ambas rutas ausentes, debe poder finalizar sin conceder autoridad sobre destino. Ahora prepare_cancel devuelve ImportRecoveryRequired y la validación de cleanup/recovery también lo rechaza, dejando una cancelación legítima permanentemente pendiente. Los positivos actuales generan el campo false/true, por eso la suite verde no cubre este estado normativo.

**Corrección:** aceptar el campo ausente como false, manteniendo rechazo de valor presente de tipo incorrecto y del resto del wrapper desconocido/inconsistente. Exigir true y ConfirmIntent durable íntegro exclusivamente para borrar destino; ausencia jamás concede esa autoridad. Añadir control focal de v1 PENDING sin flag: staging propio/ausencia puede finalizar con evento y receipt únicos; un destino existente sigue protegido. No normalizar una versión o payload inválidos para implementar la compatibilidad.

**Evidencia:** corroboración estática del delta y norma, compartida con coordinación; no se añadió ni ejecutó otra suite/reproducción. Falla de conformidad demostrable por las dos condiciones del validador, sin atribuir una prueba roja inexistente.

## Comprobaciones propias

Ejecutadas una vez, secuencialmente desde el checkout, cargando `. ./scripts/development-env.ps1` en cada shell Cargo y utilizando MSVC/cache compartida `C:/Users/david/Projects/Research-Workbench/work/cargo-target`:

```powershell
& C:/Users/david/.cargo/bin/cargo.exe check --manifest-path src-tauri/Cargo.toml
& C:/Users/david/.cargo/bin/cargo.exe clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
& C:/Users/david/.cargo/bin/cargo.exe fmt --manifest-path src-tauri/Cargo.toml --check
& C:/Users/david/.cargo/bin/cargo.exe test --manifest-path src-tauri/Cargo.toml
```

Todos **exit 0**. Rust: unit12, actor8, bootstrap13, Library53, lifecycle3, contracts6, schema4, settings1: **100 entradas**, 99 de comportamiento y helper hijo pending_wal_fixture_child. Cero fallos/ignorados. `git diff HEAD~1 -- '*.rs'` vacío porque el último commit documenta evidencia; revisión realizada sobre el rango completo BASE..HEAD. `git status --short` vacío y SHA comprobado tras la suite.

Leídos finales de `work/evidence/fix3-final-check2.log` y `fix3-native-debug-build.log`, y ambos `.exitcode`=0 en el checkout. Los logs rojos/verdes y gate del autor habían sido comprobados por el coordinador; no se repitieron UI, build nativo ni reproducciones históricas. La salida de herramientas de esta revisión acredita los cuatro comandos propios, sin inventar logs nuevos. Ningún fallo de compilación histórico se cuenta como prueba de comportamiento.

Limitaciones: lectura focal de este delta; no reabre F9/F10/F11/F13/M5 ni revisiones anteriores. No acredita selector GUI, NSIS/equipo limpio, Reader/T03 ni power loss. No se consultó OpenViking (peer no verificado y contexto local suficiente). Una aprobación posterior requiere cerrar I1, CI verde y conflictos resueltos; las comprobaciones verdes actuales no dispensan la regresión normativa.
