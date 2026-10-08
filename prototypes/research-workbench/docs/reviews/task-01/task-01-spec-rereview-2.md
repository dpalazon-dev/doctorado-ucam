N1 — ADDRESSED.

# Task 01 · Re-review de SPEC y calidad transversal · ronda 2

**Spec Compliance: Pass en el alcance de N1 y su fix diff.**

**Task quality: Approved en ese mismo alcance.**

BASE `892e4e63f4c1b8fef3de054e8f510a1e1926d95a`; HEAD `203a31258427141856f0a37eba72cbbf3cfbcca2`. Checkout: `.worktrees/task-01-scaffold`. Leídos el brief/informe centrales de fix 2, el hallazgo N1 en `task-01-rust-rereview-1.md` y el paquete completo `review-892e4e6..203a312.diff` en una pasada. Aplicado el formato de re-review Superpowers ya consultado.

Alcance limitado a N1 y nuevas roturas de los dos archivos del paquete: `src-tauri/src/desktop/lifecycle.rs` y `docs/reports/task-01-report.md`. No se reabrieron F1–F8 ni código intacto. Sin subagentes, suites adicionales ni mutaciones de producto/Git. Único archivo escrito: este informe.

## N1 · Important · Panic al crear el drenador — ADDRESSED

`src-tauri/src/desktop/lifecycle.rs:125` conserva la firma pública de `request_exit`; `:130` delega a un helper privado. El camino de producción usa `std::thread::Builder::spawn` en `:131`/`:133`, por lo que el rechazo del SO se representa como `Result` y no se transforma en panic.

`src-tauri/src/desktop/lifecycle.rs:190` maneja el error de creación: `:191` restaura `running=false`, registra el evento cerrado existente y `:195` entrega `StorageUnavailable` al diagnóstico. Devuelve false y no publica ready ni ejecuta on_ready. El estado conserva el actor y su lock; destruir la closure no sustituye el shutdown ni libera el estado principal. Así, el callback de ventana puede completar su política de impedir salida y un intento posterior puede iniciar el drenador.

`src-tauri/src/desktop/lifecycle.rs:156` comparte el callback mediante `Arc<Mutex<_>>` para disponer de él tanto en el error de creación como en el hilo. El helper permanece privado (`:136`), conserva el bound público Send y no introduce flags de fallo, configuración de producción, nuevos IPC o un framework. El diagnóstico recibe un AppError cerrado; no expone el error OS original.

La prueba local `src-tauri/src/desktop/lifecycle.rs:233` usa una biblioteca sintética, un trabajo bloqueado y otro admitido. `:258`/`:267` comprueban que el error de creación inyectado no causa unwind. El primer intento devuelve false, no anuncia salida, mantiene ready=false, restaura running=false y conserva el lock (`:275`). El reintento llama al camino de producción (`:277`), sigue impidiendo salida y reteniendo lock mientras hay trabajo bloqueado (`:284`); después de liberarlo drena ambos trabajos, comprueba el cambio persistido y permite adquirir el lock (`:290`). `:305` comprueba que el log no contiene el mensaje OS sintético. La prueba cubre el fallo y la recuperación exigidos por el brief sin agotar recursos reales.

## Evidencia contrastada

Logs inspeccionados en `work/` del checkout:

| Evidencia | Resultado observado |
| --- | --- |
| `fix2-spawn-red.log` | La prueba falla con el panic de creación inyectada y la aserción «thread creation failure must not panic». Es reproducción determinista de la semántica previa, no un rechazo real del SO. |
| `fix2-spawn-green.log` | Misma prueba: 1 passed, 0 failed. |
| `fix2-clippy-final.log` | Compilación de comprobación terminada, sin warnings/error; el informe declara exit 0 para all-targets con `-D warnings`. |
| `fix2-format-final.log` | Sin salida, consistente con fmt --check exitoso declarado. El archivo vacío por sí solo no demuestra el exitcode. |
| `fix2-unit-final.log` | Prueba unitaria de fallo/reintento: 1 passed, 0 failed; otra unit filtrada. |
| `fix2-lifecycle-final.log` | Integración existente `desktop_close_keeps_exit_and_lock_until_admitted_jobs_finish`: 1 passed, 0 failed. |

No queda una duda concreta que justifique repetir las pruebas. La sección versionada `docs/reports/task-01-report.md:186` documenta el alcance y los comandos; `:196` declara el helper privado y ausencia de firma nueva; `:221` distingue el smoke previo de este HEAD y reserva gate completo/build nativo para integración. Las afirmaciones de la entrega coinciden con el diff y la evidencia inspeccionada.

## Nuevas roturas y observaciones fuera de alcance

**Critical 0 / Important 0 / Minor 0** nuevos confirmados. La corrección es local, conserva la política y firmas del cierre exitoso y añade una prueba significativa del camino de error/reintento.

Ninguna observación nueva de código intacto. El diálogo Win32 bajo rechazo real del SO, gate completo y construcción nativa de este HEAD no fueron observados en esta re-review. Son límites explícitos de la evidencia y pasos asignados a integración, sin convertir el smoke anterior en verificación actual ni reabrir QA futura.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH / Important | 0 abiertos; N1 resuelto | pass |
| MEDIUM / Minor | 0 | pass |
| LOW | 0 | pass |

Verdict: **APPROVED para el gate de código en este ámbito**. **N1 ADDRESSED**, sin nuevas roturas confirmadas. La revisión especializada Rust y las comprobaciones de integración conservan sus responsabilidades propias; este informe no afirma haberlas ejecutado ni sustituido.
