# T02 fix2 — rerevisión Rust independiente

Estado: **DONE_WITH_CONCERNS**. Conformidad: **NOT APPROVED**. Calidad Rust: **BLOCK**, por dos Important abiertos. No Critical ni hallazgos adicionales concretos de seguridad de memoria/FFI en el delta.

BASE: `922a0669d771d6778cd3d264bcc5be91f0e96ec6`. HEAD: `21fd14ed20ae391518a7dabaaece3dc551a92fa2`. Checkout: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-02-library`. Revisor: `/root/task02_rust_review`. Sin cambios de producto, configuración, pruebas, commits ni subagentes.

## Alcance y método

Revisión única y completa del paquete `review-922a066..21fd14e.diff`, en porciones contiguas; líneas finales obtenidas de los hunks. Contexto, brief e informe fix2 centrales; normativa principal `TASK02_PORTS` resoluciones 7–9, `MANAGED_FILES` con C1 final y `WINDOWS_DIRECTORY_GUARDS`. Se conserva la evidencia y cierres previos; no se vuelve a auditar T01 ni se exige API Reader/T03. Aplicado el template Superpowers de task-reviewer y criterios de rust-patterns. Dos veredictos separados: conformidad y calidad.

Se ejecutó también `git diff HEAD~1 -- '*.rs'`: vacío, pues HEAD es documental; el rango completo de revisión es el paquete BASE..HEAD. `git status --short` permaneció vacío y `git rev-parse HEAD` confirmó el SHA indicado después de las comprobaciones.

## Seguimiento

| Punto | Resultado | Evidencia y límites |
|---|---|---|
| F3 — helpers, validación y auditoría | **NOT ADDRESSED** | `application/library.rs:149` contiene preparación sobre primitivas; update/confirm normalizan y los helpers escriben auditoría de entidad en la Transaction prestada. Pruebas de rollback, preparación y replay pasan. Falta auditoría de entidad en cancelación por recovery: Important I1. |
| F7 — fallo de staging durable | **ADDRESSED** | `StageFailure` distingue error original de prueba DONE/PENDING; `library_repository.rs:469` registra FAILED y auditoría sin receipt de éxito; `:520` reconcilia ausencia comprobada. Pruebas de PDF inválido/cifrado, source ausente, >500 MiB, parcial conservado y ausencia tras reapertura pasan. No se dispensa un STAGING sin recursos: el flujo normal ahora registra la variante stageFailure; recursos no probados continúan visibles. |
| F9 / Rust R8 — todas las mutaciones por requestId | **ADDRESSED** | `modules/library/service.rs:371`, `:414`, `:454` añaden update/archive/restore al registro Weak compartido y transfieren request/operation permits al job admitido. Mismo orden request antes de token, sin TX retenida durante filesystem. Prueba determinista de las tres mutaciones frente a cancelación pasa; caller-drop y cierre pasan. |
| F10 — guardas Windows | **ADDRESSED**, dentro de Windows local NTFS autorizado | Aperturas relativas `managed_files.rs:502` y `:623`: componente validado, UTF-16 y tamaños comprobados, padre prestado vivo, buffers/OBJECT_ATTRIBUTES vivos durante NtCreateFile síncrono y conversión a File solo tras NTSTATUS satisfactorio. Read pins sin SHARE_DELETE y handles de ancestros conservados. Los unwrap de last corresponden a vec inicializada no vacía. Pruebas FSCTL con control positivo, pin/delete, rechazo de reparse, colisión DB atómica y WAL real pasan. No es garantía contra kernel/admin/driver ni otros filesystems. |
| F11 — cancelar preparación con destino reservado ausente | **ADDRESSED** | La condición de consistencia sin destino se aplica cuando destination_path es None; Some mantiene namespace, IDs, referencias y plan actual. Cancelación preparada y recovery PENDING pasan. |
| F13 / Rust N2 — hash acotado | **ADDRESSED** | `store.rs:169` hace hash streaming con buffer de 64 KiB y límite MAX+1; comprobación de tamaño antes/después en el mismo ManagedFile. No asigna un Vec por hash. Pruebas instrumentadas y límite real 500 MiB pasan; el buffer para parser sigue bajo su gate. |
| M5 — precisión del informe futura DB | **ADDRESSED** | Informe aclara construcción de Store sin I/O y omisión de recovery en composición readonly. Pruebas futura DB/inventario íntegro pasan; no se confunde construcción de objeto con escritura. |
| C1 — candidates y cancelación de promoción propia | **NOT ADDRESSED** | Candidates DOI/SHA combinados provienen de consulta ordenada de la TX tanto en preparación como commit tardío. Store exige prueba de promoción, hash/tamaño en mismo handle, ausencia de staging, y ausencia de referencias. Carrera tardía, loser cancel/replay y reapertura pasan. Un wrapper FAILED de versión desconocida puede ser convertido en autoridad v1 antes de validarlo: Important I2. |

R1–R7 originales y minors ya cerrados se mantienen cerrados; R8 se cierra con F9. El defecto de memoria N2 se cierra con F13. F3 y C1 conservan cierres parciales, sin declarar cumplimiento completo.

## Important abiertos

### I1 — F3: recovery confirma cancelación sin auditoría de entidad

**Ubicación:** `src-tauri/src/adapters/sqlite/library_repository.rs:1011` (`mark_recovered_cancelled`); comparación con `:934` (`commit_cancel`).

La rama recovery cambia cleanup a DONE y publica receipt con `with_receipt`, pero su acción termina en `Ok(Json::Null)` sin `audit_change`. La auditoría de ejecución que genera with_receipt tiene entity_id NULL y no sustituye `import.cancelled` sobre operation_id. Una cancelación PENDING terminada tras reinicio deja historia distinta de la misma cancelación terminada normalmente, incumpliendo TASK02_PORTS y la atomicidad auditoría/receipt exigida.

**Corrección:** usar el mismo helper/acción transaccional de cancelación para ambas rutas, incluyendo un evento `import.cancelled` ligado a operation_id y requestId dentro de la TX. Replay no debe duplicarlo. Extender la prueba de recuperación existente para exigir un evento de entidad y rollback conjunto si su inserción falla. Corroborado por lectura del delta; no se repitió la reproducción de coordinación.

### I2 — C1: se normaliza un wrapper desconocido antes de validar su autoridad

**Ubicación:** `src-tauri/src/application/library.rs:90`–`:94`; `src-tauri/src/adapters/sqlite/library_repository.rs:737`–`:739` (`prepare_cancel`).

Para FAILED, `has_durable_promoted_confirmation` acepta cualquier error_json cuyo kind sea cancellation y promotionConfirmed sea true; no exige version=1 ni la estructura completa de cancelación en esa rama. `prepare_cancel` usa ese booleano y sobrescribe error_json por un wrapper version=1 nuevo. La validación posterior ve el wrapper normalizado y puede autorizar borrar el destino propio. Un wrapper de versión desconocida que debería permanecer ambiguo adquiere autoridad por reinterpretación, contrario a MANAGED_FILES C1 y rechazo de JSON interno desconocido.

**Corrección:** validar el wrapper previo completo, incluida versión, cleanup, requestId, command y payload vinculados, antes de preservar promotionConfirmed o modificar el record. Una versión desconocida debe dar ImportRecoveryRequired y conservar archivo y JSON. Mantener la autoridad duradera de los reintentos v1 válidos. Añadir el negativo FAILED/cancellation/version desconocida con promotionConfirmed=true; no confiar solo en la validación del wrapper recién escrito. Corroborado estáticamente; reproducción de coordinación no duplicada.

## Comprobaciones ejecutadas

Cada shell Cargo cargó primero `. ./scripts/development-env.ps1` desde el checkout, con cache compartida `C:/Users/david/Projects/Research-Workbench/work/cargo-target` y MSVC configurado. Ejecutadas secuencialmente, una vez:

```powershell
& C:/Users/david/.cargo/bin/cargo.exe check --manifest-path src-tauri/Cargo.toml
& C:/Users/david/.cargo/bin/cargo.exe clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
& C:/Users/david/.cargo/bin/cargo.exe fmt --manifest-path src-tauri/Cargo.toml --check
& C:/Users/david/.cargo/bin/cargo.exe test --manifest-path src-tauri/Cargo.toml
```

Todos exit 0. Suite: 97 entradas del runner, 96 de comportamiento y helper `pending_wal_fixture_child`: unit 11, actor 8, bootstrap 13, Library 51, lifecycle 3, contracts 6, schema 4 y settings 1. Sin fallos/ignorados. Cargo no escribe modificaciones rastreadas en el checkout; árbol limpio comprobado. La salida de herramientas constituye la evidencia propia de esta revisión; no se fabrican logs ni se atribuye al revisor el gate UI/native del autor.

Evidencia del autor leída en el checkout: `work/evidence/fix2-final-check2.log` y `.exitcode` (0), `fix2-native-debug-build.log` y `.exitcode` (0). El build finalizó en perfil dev. No se repitieron UI, build nativo, benchmarks, repros de otros revisores ni instalación/NSIS; esta revisión no demuestra GUI, instalación limpia o resistencia a power loss. La aprobación eventual presupone CI verde y conflictos resueltos, además de cerrar I1/I2.
