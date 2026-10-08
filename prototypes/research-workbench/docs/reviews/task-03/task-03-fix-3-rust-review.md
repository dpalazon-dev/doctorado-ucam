# T03 fix3 — cierre Rust de R2a

Estado: **DONE**. Veredicto: **PASS** para el delta Rust. **R2a cerrado**, sin hallazgos Critical/Important/Minor nuevos. R2b y Library–Reader conservan el cierre de fix2; no se reabren.

- BASE: `30d532714d053c09d4021bc89bafbb0168b1602a`.
- HEAD: `5a512a7bba610e90ecb10d657ed1b36095df4078`, árbol limpio.
- Worktree: `.worktrees/task-03-reader`.
- Revisor: `/root/task03_rust_review`.
- Fuentes: brief e informe fix3, revisión Rust fix2 y diff Rust completo del delta. El único archivo Rust cambiado es `src-tauri/tests/reader_integration.rs`; producto, actor, puertos, store y protocolo permanecen sin cambios en este delta.

## Cierre preciso

En `ConfirmObservedPersistence::confirm_open`, líneas 113–123, el wrapper conserva el future real y lo sondea con el mismo `Context` dentro de `std::future::poll_fn`. Solo después de recibir su primer `Poll::Pending` envía `admitted`; `signaled` impide repetir la señal y el wrapper devuelve el `Poll` real sin sustituir el resultado ni alterar el despertar.

En este adaptador concreto, el primer Pending ocurre después de `DbActor.submit`, mientras espera el resultado del job. El test ya mantiene bloqueado el actor, confirma su entrada y espera la terminación del caller abortado. Por tanto recibir `admitted` acredita ahora la admisión en la cola, en lugar de la mera construcción del future.

Las líneas 453–465 observan entonces el handle adquirido y aún vivo y un competidor Library con el mismo requestId realmente sondeado y pendiente. Tras liberar el actor, el test espera idle, verifica un único Drop y contrasta los resultados durables: commit mantiene el receipt de apertura y provoca Conflict en el competidor; rollback deja la sesión intacta y permite la operación posterior. Se conservan la propiedad de mantenimiento y la barrera instalada antes de adquirir/observar el probe. Esto cierra el punto residual documentado en fix2 sin cambiar producto ni introducir una implementación alternativa.

## Evidencia propia

Después de `. .\scripts\development-env.ps1`:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml --test reader_integration dropped_reader_caller_keeps_handle_and_shared_request_owner_until_open -- --nocapture
```

Todos terminaron con exit 0. Los dos tests focales —commit y rollback— pasaron; 15 casos quedaron filtrados. Check/Clippy usaron el producto ya construido; no se ejecutó el gate completo, otro build nativo ni las suites cerradas. `git diff --check BASE..HEAD` pasó y el árbol siguió limpio.

La evidencia de gate 67 UI/128 entradas Cargo y build nativo 0 corresponde al autor y su verificación por el orquestador. El reporte fix3 identifica correctamente las dos entradas auxiliares de proceso hijo. No se atribuye esa ejecución completa a esta revisión focal.

## Alcance de la aprobación

R1, R2 y R3 de la revisión Rust original quedan cerrados mediante los cierres acumulados de fix1/fix2/fix3. Este PASS no sustituye las revisiones TypeScript/conformidad restantes ni el gate sobre el merge preparado, que debe seguir limpio y sin conflictos. No se hizo merge, se editó producto, se abrió GUI ni se modificó el ejecutable de QA. WebView2, picker e instalación siguen fuera del alcance de esta evidencia.
