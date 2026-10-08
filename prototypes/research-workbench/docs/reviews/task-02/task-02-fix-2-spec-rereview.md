# T02 fix 2 — rerevisión independiente SPEC y calidad

Estado: DONE_WITH_CONCERNS. SPEC: **NOT APPROVED**. Calidad: **NEEDS FIXES**. Hay 2 Important abiertos; 0 Critical y 0 Minor nuevos. No integrar este HEAD hasta corregir y revisar esos dos casos.

BASE `922a0669d771d6778cd3d264bcc5be91f0e96ec6`; HEAD congelado `21fd14ed20ae391518a7dabaaece3dc551a92fa2`; commit de código `3cdeae2bfc936e5447ae9910a8f7bc0ea16a730d`. Checkout `.worktrees/task-02-library`. Revisor `/root/task02_spec_review`; responsabilidad SPEC/calidad, sin implementación. Única escritura: este informe. No tests, probe, Git, configuración, merges ni subagentes.

## Alcance y veredictos por hallazgo

Se revisaron brief/contexto/reporte fix2, normativa vigente del principal (TASK02_PORTS, MANAGED_FILES y WINDOWS_DIRECTORY_GUARDS), paquete completo `review-922a066..21fd14e.diff` por porciones y evidencia final existente. Se completaron lecturas focales cuando una función estaba cortada en el diff. El informe del autor no se tomó como prueba de conformidad. F1/F2/F4/F5/F6/F8/F12 y M1–M4 permanecen cerrados; no se reabre Reader T03 ni se exige UI nueva.

| Hallazgo | Veredicto | Evidencia y límite |
|---|---|---|
| F3 | **NOT ADDRESSED completamente** | Extracción y helpers corregidos; falta auditoría de entidad al completar cancellation por recovery (Important 1). |
| F7 | **ADDRESSED** | StageFailure distingue error de Pending/Done; persistencia registra FAILED/version1/stageFailure; recuperación solo completa pendientes cuando inspect prueba ausencia. Conserva parciales e incidencia. |
| F11 | **ADDRESSED** | Consistencia de reserva sin destino se limita a None; Some exige namespace/destino/IDs actuales, referencias y snapshot del plan. Preparado ordinario y cancellation PENDING ya terminan. El defecto nuevo C1 se registra por separado. |
| M5 | **ADDRESSED** | Reporte fix2:18 rectifica explícitamente constructor Store sin I/O y recovery omitido para esquema futuro, conservando el reporte anterior como historia. |
| C1 | **NOT ADDRESSED completamente** | Candidates correctos y cleanup postpromote ordinario corregido; retry puede convertir JSON de cancelación desconocido en autoridad nueva válida (Important 2). |

F9/F10/F13 tienen revisores especializados. Se examinó su delta para regresiones de conformidad/calidad y no se añaden hallazgos independientes sobre ABI, concurrencia o streaming. Este informe no sustituye esos veredictos.

## Important 1 — Recovery completa la cancelación sin su auditoría de entidad (F3)

Archivo: `src-tauri/src/adapters/sqlite/library_repository.rs:1011`, función `mark_recovered_cancelled` (1004). Ruta consumidora: `src-tauri/src/modules/library/service.rs:601`.

La cancelación normal registra `import.cancelled`, con operationId como entity_id, dentro de la misma TX de cleanup DONE y receipt (library_repository.rs:934). Recovery ejecuta el mismo cambio de negocio PENDING→DONE y registra receipt mediante with_receipt, pero su closure termina tras UPDATE sin audit_change. El único evento nuevo es el de ejecución `library_cancel_import`, cuyo entity_id es NULL en receipts.rs; no sustituye el evento de cambio requerido por F3/TASK02_PORTS:112. Por tanto, cerrar/reabrir antes del cleanup cambia el historial de entidad de una misma cancelación exitosa.

Reproducción focal sin inventar un resultado ejecutado: extender `durably_promoted_cancel_pending_is_recovered_after_database_reopen` (library_integration.rs:981). Tras reconcile y su receipt/DONE exitosos, consultar `count(*) FROM audit_events WHERE request_id=<cancellation_request> AND entity_id=<operation_id> AND action='import.cancelled'`: el código actual da 0; cancelación normal da 1. El test nuevo comprueba receipt/estado/papers pero no ese evento. No se ejecutó aquí.

Corrección: usar la misma finalización transaccional de cancelación, o añadir audit_change de entidad en el closure de recovery, con receipt y DONE en un único commit. Probar evento exactamente una vez, replay sin duplicarlo y rollback conjunto. No requiere segundo commit, DTO ni framework de eventos.

## Important 2 — Retry normaliza una cancelación de versión desconocida antes de validarla (C1)

Archivos: `src-tauri/src/application/library.rs:90`–94 y `src-tauri/src/adapters/sqlite/library_repository.rs:737`–739. Requisito: MANAGED_FILES:53 conserva JSON desconocido como ambiguo.

`has_durable_promoted_confirmation` acepta un record FAILED si error_json tiene kind=cancellation y promotionConfirmed=true; no valida la versión, cleanup o receipt de esa cancelación anterior. prepare_cancel usa ese resultado para promotion_confirmed y sobrescribe error_json con una cancelación nueva version1/PENDING antes de validate_cleanup_plan. La validación posterior y Store ven únicamente el wrapper nuevo válido. La revalidación posterior, por tanto, no conserva el rechazo que debía producir el JSON original desconocido.

Caso end-to-end concreto: intención con ConfirmIntent v1 íntegra, promoción durable previamente cancelada a FAILED/PENDING, staging ausente, destino reservado propio con hash/tamaño correctos, sin documentos/intenciones/receipt de confirmación que lo referencien. Cambiar solo la version de error_json de cancellation a 99, conservando promotionConfirmed=true. Reintentar cancelImport con el token aún admitido y el requestId de cancelación sin receipt. previous_cancel no tiene éxito previo; prepare_cancel vuelve a conceder promotionConfirmed=true y reemplaza version99 por version1; validate_cleanup_plan pasa sobre el nuevo record; cleanup puede eliminar el destino y commit_cancel publica DONE/receipt. Debería conservar archivo/intención desconocida con ImportRecoveryRequired. Es un camino introducido por la recuperación de autoridad C1, no un defecto hipotético de un helper aislado.

Corrección: validar la intención de cancelación previa dentro de la TX y antes de reescribirla. Para FAILED, preservar promotionConfirmed solo desde una cancellation v1 íntegra, PENDING y ligada a request/token/receipt; JSON desconocido/inconsistente debe fallar sin convertirlo a una cancelación reconocida ni borrar. Añadir regresión basada en el fixture postpromote real, alterando versión a 99 y comprobando bytes conservados, ausencia de receipt exitoso y wrapper no normalizado. Los controles actuales sin promoción y con referencia no cubren esta variante. No se ejecutó aquí.

Los dos hallazgos son incumplimientos de la decisión vigente; no son defectos impuestos por el plan ni están dispensados.

## Fortalezas verificadas

- `application/library.rs:149` contiene prepare_import con estado, expiración, payload original canónico, duplicados/reuse y reserva. SQL queda en el adapter y ningún helper posee commit. update_metadata normaliza/valida por sí mismo (339); confirm/edit/archive/restore emiten auditoría en la TX prestada. La regresión del consumidor comprueba metadata inválida, rollback de entidad y rollback del evento; receipt replay distingue evento de ejecución de evento de entidad.
- F7 registra code seguro y cleanup como datos distintos. Store conserva Pending ante fallos de copia/sync/lectura; rechazo PDF con eliminación acreditada y sobrelímite retirado quedan Done. No convierte el rechazo en cancelación humana o receipt de éxito. Recovery de ausencia registra reconciliación; parcial sin hash permanece como recurso/issue.
- C1 entrega candidates desde la TX que detecta DOI/SHA, DISTINCT por paperId y orden determinista. Combina razones una vez; late commit también devuelve detalle accionable. Los positivos postpromote/cancel/replay y reapertura conservan el ganador y retiran solo original.pdf reservado, con tamaño/hash comprobados y staging ausente. El namespace y las referencias siguen siendo condiciones independientes del hash.
- El delta Windows usa adquisición relativa de componentes/pins y comprobación NTFS, pin de lectura con READ_DATA y sin DELETE sharing. Bootstrap mantiene clasificación por handle y FILE_CREATE de DB tras LibraryLock; el futuro esquema usa DB existente sin crear sentinela. Los tests de producto contienen FSCTL con control positivo, delete sharing/control tras cierre, no-clobber y SQLite/WAL/reopen; no se confunden con el probe C#.

## Evidencia revisada y Cannot Verify

`work/evidence/fix2-final-check2.log` y su exitcode 0: typecheck, UI 14/14, build Vite, Clippy sin warnings y Rust con 97 entradas (96 comprobaciones de comportamiento + helper pending_wal_fixture_child), sin filtrados en las suites finales. El gate incluye fmt/contract check conforme al comando registrado. `fix2-native-debug-build.log` y exitcode 0 muestran cargo build nativo debug completado. No se volvió a ejecutar ninguna suite; filtros anteriores con cero tests no se usan como evidencia.

El verde actual no comprueba auditoría de cancellation recovery ni rechazo de una cancellation previa version99. Sus reproducciones son deducciones directas del camino de código revisado, no resultados de nuevas ejecuciones. La corrección y sus regresiones deben revisarse en el nuevo HEAD; este informe no acredita un merge ni gate integrado.

No se verifica selector GUI, Reader T03, NSIS, instalación en equipo limpio, resistencia frente a administradores/drivers arbitrarios o durabilidad ante pérdida de alimentación. Tampoco se sustituye la revisión Rust/seguridad independiente. Esos límites no añaden requisitos fuera de la tarea.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH / Important | 2 | needs fixes |
| MEDIUM | 0 | pass |
| LOW / Minor nuevos | 0 | pass |

Verdict SPEC: NOT APPROVED (F3 y C1 parciales). Verdict calidad: NEEDS FIXES. Las reglas del proyecto impiden integración con Important abiertos.
