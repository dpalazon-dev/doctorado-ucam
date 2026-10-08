# Spec Compliance: Issues found — Task Quality: Needs fixes

Ámbito: revisión independiente Rust de T01, persistencia, actor, contratos y arranque Windows. BASE `dd31de021d9d1ee60c0dd3d60c6347525d0e5903`; HEAD `13f985f212b29008e699fbde1ee00d5a556dd408`. Checkout `C:/Users/david/Projects/Research-Workbench/.worktrees/task-01-scaffold`. Revisión por secciones Rust del paquete `review-dd31de0..13f985f.diff`; no se modificó producto, índice, HEAD ni ramas y no se despacharon agentes.

## Spec Compliance

**Issues found.** Las garantías fundacionales de cierre controlado, diagnóstico sin mutación de biblioteca y preservación exacta de presencia/null requieren R1–R3. Las funciones futuras deliberadamente no implementadas y `UnsupportedCapability` cumplen el alcance T01; estos hallazgos no solicitan implementarlas.

Autoridad utilizada: brief T01, clarificación de PhaseDefinition, instrucciones de revisión, INTENT/STATUS y pasajes pertinentes de DATA/SPECS/CONTRACTS. La ausencia de revisión optimista o gates de módulos futuros no se confunde con una omisión de T01. Se aplicaron `rust-patterns`, Karpathy y la plantilla de revisión instalada de Superpowers.

## Strengths

- **Actor y límites reales:** `src-tauri/src/adapters/sqlite/actor.rs:141` crea cola 64; `:202` usa `try_send`, y `:149` prepara y posee Connection/lock en el hilo dedicado. `src-tauri/tests/db_actor.rs:81` satura con canales de coordinación, sin sleeps para determinar capacidad. Los tests prueban conexión/hilo único y rechazo Busy.
- **Atomicidad e idempotencia:** `src-tauri/src/application/unit_of_work.rs:6` inicia transacción Immediate y confirma al terminar acción; `src-tauri/src/adapters/sqlite/receipts.rs:28` integra acción, receipt y evento genérico en ella, compara comando/hash y recupera el resultado previo. `:23` ordena objetos recursivamente antes de SHA256; el orden de arrays se conserva. El historial específico de cambios/revisiones corresponde al servicio propietario futuro dentro de esa misma transacción; el evento genérico con `{}` de `:55` no demuestra todavía auditoría de dominio ni se presenta aquí como tal.
- **Configuración SQLite y defensa de migración:** `src-tauri/src/adapters/sqlite/actor.rs:55` activa FKs, `:58` WAL, `:59` FULL y `:54` timeout 5000 ms. `src-tauri/src/adapters/sqlite/migrations.rs:32` verifica checksums publicados; `:190` conserva snapshot previo cuando existe esquema y ejecuta ledger y cambios en UoW. `src-tauri/tests/desktop_bootstrap.rs:60` demuestra rollback de migración fallida sin tabla parcial y conservación del backup; FTS5/integridad/reapertura se ejercitan con SQLite real bundled.
- **Snapshot DB y recursos:** `src-tauri/src/adapters/sqlite/migrations.rs:139` usa Backup API, no copia aislada de DB con WAL; `:122` rechaza transacción abierta. La copia verifica hashes, rechaza symlinks/reparse points de los recursos recorridos y `:157` restringe rutas documentales relativas. `:174` genera inventario ordenado con hashes/tamaños y manifiesto de identidad. `src-tauri/tests/desktop_bootstrap.rs:195` comprueba DB/metadata/PDF y manifiesto; `:217` rechaza un PDF referido cuyo contenido cambió. Esta exclusión se confía al lease documentado del llamador; la única invocación de producto T01 está en arranque antes de admitir jobs.
- **Fallos ordinarios y panic:** `src-tauri/src/adapters/sqlite/actor.rs:192` captura unwind de jobs y revierte una transacción que quede abierta; `src-tauri/tests/db_actor.rs:131` verifica que el cambio inyectado no persiste. `:154` prueba que un timeout de shutdown mantiene el lock y permite un cierre posterior. R1 trata la integración de esta garantía con el ciclo de vida de la aplicación.
- **Wire y Windows:** `src-tauri/src/transport/dto.rs:14` valida UUID canónico al deserializar; los DTOs usan camelCase/deny_unknown_fields y `:2` exige presencia de nullables obligatorios. La generación desde Rust está centralizada en `src-tauri/src/bin/generate_contracts.rs:8`, con verificación de cobertura/drift en `scripts/generate-contracts.ps1:12`. `src-tauri/src/adapters/windows/startup.rs:30` justifica correctamente la FFI MessageBoxW y mantiene buffers vivos/NUL-terminated; `src-tauri/src/main.rs:3` trata Result de arranque y termina con error tras diagnóstico seguro.

## Issues

### Critical

Ninguno confirmado en esta pasada.

### Important

**R1 — El timeout del actor no impide que termine el proceso.** `src-tauri/src/lib.rs:18` atiende ExitRequested y `:21` descarta el Result de `state.shutdown()`. `src-tauri/src/desktop/lifecycle.rs:100` espera cinco segundos; `src-tauri/src/adapters/sqlite/actor.rs:238` puede devolver Busy dejando vivos el hilo, la Connection y trabajos ya admitidos. Al no prevenir la salida ni reintentar, `run` termina y `main` sale: Windows termina el hilo restante y libera su lock por fin del proceso. Un job admitido que aún no alcanzó su commit y los jobs pendientes no terminan ni entregan resultado. FULL y UoW siguen protegiendo los commits ya realizados; no se afirma corrupción de DB ni pérdida de un guardado ya confirmado. En el UI T01 actual esto puede interrumpir lecturas de status; el defecto bloquea la garantía de cierre de la infraestructura entregada para trabajos persistentes. **Remedio:** conservar el proceso y el estado ante Busy, prevenir ExitRequested y completar/reintentar el drenaje con una política explícita de cierre y diagnóstico antes de permitir salida. Mantener la espera acotada fuera del hilo de ventana. **Verificación necesaria:** caso de ciclo de vida con un job bloqueado más allá del timeout y otro job admitido, comprobando que el proceso/ciclo no salen hasta la política de cierre y que el lock no se libera anticipadamente. `src-tauri/tests/db_actor.rs:154` cubre actor aislado, no ExitRequested.

**R2 — La rama de esquema futuro crea un archivo dentro de la biblioteca antes de conocer el esquema.** `src-tauri/src/adapters/sqlite/actor.rs:42` adquiere LibraryLock antes de leer user_version (`:43`) y elegir readonly (`:48`). `src-tauri/src/adapters/windows/lock.rs:15` pide escritura y `:16` permite crear `.writer.lock`. Para una biblioteca futura válida sin ese archivo —por ejemplo una copia que conserva DB/metadata/recursos, ya que `src-tauri/src/adapters/sqlite/migrations.rs:144` no copia el lock— abrir el diagnóstico modifica su raíz. Si esa raíz permite lectura pero no creación/escritura, el arranque falla antes de poder mostrar el diagnóstico. El fichero DB permanece intacto; el problema es incumplir «no se modifica la biblioteca» y exigir permiso de escritura para diagnosticarla (DATA:122 / SPEC REQ-001-04). Para un DB existente futuro, `:39` no crea la raíz, `:109` no crea documents/staging/recovery, y tampoco escribe identidad/manifiesto; no se atribuyen esas mutaciones a esta rama. **Remedio:** separar la detección/entrada de diagnóstico de la adquisición que crea el lock del escritor y garantizar que la rama readonly no crea/actualiza entradas en la biblioteca. **Verificación necesaria:** comparar inventario y contenidos de toda una raíz futura sin `.writer.lock`, además de un caso con permisos de solo lectura. `src-tauri/tests/desktop_bootstrap.rs:40` reutiliza una biblioteca creada por el actor, por lo que el lock ya existe, y solo compara bytes de research.sqlite (`:57`).

**R3 — El DTO pierde la diferencia entre campo omitido y null explícito.** `src-tauri/src/transport/dto.rs:1953` usa `serde(default, skip_serializing_if = "Option::is_none")` para `ConceptUpdateArgs.domain: Option<Option<String>>` (`:1955`). La deserialización normal del Option exterior da None tanto para ausencia como para JSON null. Por tanto `{domain:null}` pierde la intención explícita de limpiar el dominio y al serializar de nuevo desaparece el campo. CONTRACTS:294–295 permite `domain?: string|null`, con tres estados distintos: no cambiar, limpiar, asignar valor. Es un defecto de los DTOs exactos entregados ahora, aunque updateConcept siga honestamente no disponible. **Remedio:** deserializador de campo presente que produzca `Some(Option<String>::deserialize(...))`, manteniendo default para ausencia, o un tipo de presencia equivalente; conservar wire/TS canónicos. **Verificación necesaria:** round-trip de ausencia, null y string, con asserts respectivos None / Some(None) / Some(Some(...)). La comprobación adicional enfocada del impl Option de la dependencia instalada (`serde_core-1.0.229/src/de/impls.rs:884` y `:893`) confirma que visit_unit/visit_none retornan None; no se añadió ni ejecutó una suite para este caso.

### Minor

Sin hallazgos separados de estilo. Las nulabilidades extra generadas de `PageDto.total` y `WorkflowGetPhaseDefinitionArgs.version` corresponden a la revisión general/TypeScript comunicada por el coordinador; no se duplican aquí como nuevos R.

## Comprobaciones y alcance de evidencia

El mandato literal del rol Rust de nivel developer fue: **«Run cargo check, cargo clippy -- -D warnings, cargo fmt --check, and cargo test — if any fail, stop and report»**, seguido de **«Run git diff HEAD~1 -- '*.rs'»**. El coordinador confirmó que ese mínimo superior prevalece sobre la prohibición ordinaria de repetir suites del encargo. Por esa razón se ejecutaron una sola vez, secuencialmente, desde el checkout revisado y con `. ./scripts/development-env.ps1` antes de cada comando Cargo. No fueron duplicaciones voluntarias ni se abrió otro barrido de tests.

| Comando exacto tras cargar entorno | Resultado |
| --- | --- |
| `cargo check --manifest-path src-tauri/Cargo.toml` | exit 0 |
| `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` | exit 0, sin warnings |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | exit 0, sin cambios |
| `cargo test --manifest-path src-tauri/Cargo.toml` | exit 0; 7 actor + 12 bootstrap + 4 contratos = 23 integración; unit/doctests 0 |
| `git diff HEAD~1 -- '*.rs'` | exit 0; inspección obligatoria de cambios recientes; el dictamen usa el paquete BASE..HEAD asignado |

Lectura adicional enfocada: impl de Option de serde_core para R3. No se releyeron fuentes cambiadas fuera del paquete; se recuperaron secciones de paquete cortadas por límites de salida y se derivaron sus números de línea de hunk. No se ejecutaron fault-injection adicional, pruebas nativas/instalador, npm, ni auditorías de dependencias ajenas al mínimo del rol.

## Cannot Verify

- El diálogo MessageBoxW vigente ante segunda instancia no fue observado en esta revisión; el test de texto/clasificación y la compilación no equivalen a observar ese diálogo (`src-tauri/tests/desktop_bootstrap.rs:226`). El informe del autor declara expresamente esta limitación.
- No se demuestra resistencia del snapshot ante corte de alimentación ni fallo VFS/rollback: los tests verifican copia/SQL/hash en operación normal y errores inyectados de SQL/PDF. Tampoco se demuestra defensa frente a sustitución concurrente de rutas por un proceso externo; la exclusión ofrecida es la del escritor/maintenance de la aplicación.
- NSIS instalado, WebView2 offline, release PE sin consola y equipo limpio corresponden a T10; no se aprueban por estos resultados Cargo. La observación nativa anterior del autor precede al pequeño cambio de startup/settings; su build vigente documentado no demuestra una nueva observación visual.
- La aprobación final de integración requiere revisiones general/TS sin Important abiertos, árbol/merge coherentes y checks pertinentes verdes conforme a AGENTS; esta revisión no efectuó merge ni confirma estado de CI remoto.

## Assessment

**Task quality: Needs fixes.** La estructura de actor/UoW/migraciones y las pruebas reales ofrecen una base sólida, pero R1–R3 contradicen garantías asignadas a T01 y deben volver al autor antes de integrar. Los cuatro mínimos Cargo verdes no invalidan esos defectos de semántica/ciclo de vida.
