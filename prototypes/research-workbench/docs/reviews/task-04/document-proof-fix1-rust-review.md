# T04a fix1 — revisión Rust focal

Fecha: 2026-10-03. Revisor: `/root/task04a_rust_review`.

**PASS.** La corrección cubre los fallos a través de ambos puertos del store y cierra la observación Rust sobre documentación. No se identificaron nuevos hallazgos críticos, importantes o medios en el delta focal. El gate SPEC final y la integración corresponden a Sol.

## Corte y alcance

- BASE original T04a: `9188ac5dc4f1cc5ad22347ef1422a2c51d795b1b`.
- Corte previamente revisado: `20c7da9f9c9cb1757bd9755aceb00906d6f56239`.
- HEAD fix1: `9ad0bc269909e8013f3add0dc9ac389290fb0b81`.
- Commit de corrección: `ec527ef9757b700d063ffe6a93ee666ef9282275`.
- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04a-document-proof`.

HEAD y limpieza verificados antes y después. Revisados los tres archivos del delta focal: `store.rs`, `reader_ports.rs` y el informe versionado. Leídos el brief central `task-04a-fix-1-brief.md`, el informe actualizado y los informes previos SPEC/Rust/seguridad. Se mantiene la skill rust-patterns ya leída. No se reabrió la revisión de diseño OS: managed_files y las pruebas de integración no cambian en fix1.

## Resultado de la inspección

1. **Aislamiento de tests.** `store.rs:39-50,63-64,73-80,195-196,211-222` limita a `cfg(test)` el campo, enum, inicialización, setters privados, clon y consumo de la inyección. Cada construcción de store crea su propio Arc/Mutex; sus clones comparten esa instancia deliberadamente. No hay estado global ni API accesible desde IPC. Con `cfg(not(test))` se llama directamente a la misma `managed_file_open` del corte aprobado.
2. **Mutex sin retención durante el panic.** En `store.rs:212`, `take()` extrae la inyección en una sentencia terminada antes del match. El MutexGuard se destruye antes de ejecutar la rama Panic, por lo que ese panic no envenena el lock. No hay await ni adquisición filesystem dentro del guard. Los unwrap nuevos quedan exclusivamente en código de tests.
3. **Se prueba el trabajo real.** `public_proof_and_reader_ports_preserve_injected_open_failures` configura el store y llama a `prove_access` y `open_verified`. La closure real de `spawn_blocking` consume la inyección después de la validación del registro y antes de adquirir el archivo; resultado y join pasan por las mismas ramas usadas en producción. No se llama directamente a los mappers para justificar esta cobertura.
4. **Resultados diferenciados.** AccessDenied da `Ok(Unavailable(AccessDenied))` mediante proof y `Err(StorageUnavailable)` mediante Reader. Para cada puerto, Fatal(NotFound) y Fatal(StorageUnavailable) permanecen errores con su código. El panic dentro de cada trabajo bloqueante también resulta en Err(StorageUnavailable). Las aserciones rechazan cualquier Ok en los casos fatales.
5. **Consumo y recuperación.** La última llamada, sin reinyección y después de ambos panics, exige `Available`. Acredita consumo de la inyección y recuperación del recorrido normal, y detectaría que el lock quedó envenenado. El control usa archivo, registro, root y tamaño sintéticos coherentes para esta operación.
6. **Retención intacta.** La closure sigue poseyendo root, registro y el Arc de test. La rama de éxito conserva `ManagedDocumentReadHandle` con ManagedFile; no cambia el lifetime de archivo/padres/pins ni Reader. El punto de inyección devuelve antes de adquirir el archivo o provoca panic antes de esa adquisición. No aparece una nueva fuga de handles ni una referencia prestada que sobreviva a su propietario.
7. **Documentación y evidencia.** El rustdoc nuevo en `reader_ports.rs` explica propiedad de handle/pins, disponibilidad limitada a causas OS, ausencia de garantía de pertenencia al fallar y propagación de errores fatales. El informe separa las pruebas de helpers de los recorridos públicos. `git diff --check BASE_original..HEAD` ya pasa sobre el delta completo, incluido EOF del informe.

La nueva fixture escribe los bytes `synthetic proof fixture` en un archivo llamado original.pdf. Esto basta para comprobar acceso y tamaño; no es un PDF estructuralmente válido ni acredita validación del formato. La frase del informe del autor sobre un PDF sintético válido debe entenderse con ese límite. No afecta a este gate, porque proof no analiza el PDF; import y parser tienen sus propias pruebas. La denegación sigue siendo inyectada en el límite de apertura del store, no una ACL real ni una llamada NtCreateFile fallida provocada por esa ACL.

## Validación

El rol de revisión Rust exige los cuatro comandos Cargo al ser invocado. Se ejecutaron sobre el nuevo HEAD, pese a la preferencia de no repetir gates ya verdes, con el helper local y sin nueva ejecución de UI/build/app:

```powershell
. .\scripts\development-env.ps1
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
```

Los cuatro terminaron correctamente; el shell comprobó `$LASTEXITCODE` tras cada paso y devolvió **exit 0** (sesión exec `27250`). **138 entradas Rust aprobadas**, incluidos helpers: 22 unitarias, 22 Reader, 4 protocolo y las demás suites existentes. No se presentan como 138 comportamientos independientes. La nueva prueba pública consta como PASS; las regresiones de retención hasta commit/rollback también pasan.

Se leyeron `work/task-04a/fix1-check.log`, `fix1-build-debug.log` y sus `.exit`, ambos 0: el autor ejecutó gate con 69 pruebas UI, 138 entradas Rust, Clippy all-targets y contratos, además de build debug sin bundle. Se distingue esta evidencia del autor de los comandos ejecutados por este revisor.

Inspección Git: diff focal completo, `git diff --check 9188ac5dc4f1cc5ad22347ef1422a2c51d795b1b..9ad0bc269909e8013f3add0dc9ac389290fb0b81`, status, entradas sin resolver y HEAD. Todos conformes; worktree limpio y sin conflictos. No se verificó un merge preparado.

## Entrega

Sin cambios de producto, subagentes, commits, merge ni configuración global. Única escritura: este informe central. No app, instalador, ACL real ni fallback no Windows; no se declara Workflow/PRE/P1 implementado. La aprobación de integración presupone las restantes revisiones y el gate del merge preparado en verde.

**DONE — PASS Rust focal de fix1; observación previa cerrada y sin nuevos bloqueos.**
