# T04a — revisión independiente SPEC/calidad

Fecha: 2026-10-03. Revisor: `/root/task04a_spec_review`.

- Worktree revisado: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04a-document-proof`.
- BASE: `9188ac5dc4f1cc5ad22347ef1422a2c51d795b1b`.
- HEAD: `20c7da9f9c9cb1757bd9755aceb00906d6f56239`, verificado; árbol limpio.
- Alcance: diff completo de cinco archivos, ABI proof, apertura compartida, guardas, compatibilidad Reader y suficiencia de la evidencia exigida por el brief y `TASK04_PORTS`.
- Revisión de producto sólo lectura. No compilación, ejecución de pruebas, aplicación, instalador, merge ni subagentes. Única escritura: este informe central.

## [HIGH] Faltan las inyecciones exigidas a través del puerto del store

Ubicación: `src-tauri/src/adapters/documents/store.rs:721` y `:743`; requisito: `docs/plans/TASK04_PORTS.md:162`.

`injected_access_denial_is_unavailable_but_guard_errors_stay_fatal` llama directamente a `map_document_open_failure`; la comprobación legacy llama directamente a `legacy_error`. `blocking_panic_remains_a_storage_error` crea su propio trabajo bloqueante y llama directamente a `flatten_blocking_result`. Ninguno invoca `LocalDocumentStore::prove_access` ni `open_verified`. El test NTSTATUS de `managed_files.rs` también es una prueba aislada del mapper, apropiada para ese nivel.

El contrato exige por separado «pruebas del store para ausencia real, denegación inyectada en el límite de apertura y fallo genérico/panic inyectado». Por tanto, aprobar los helpers no cierra esa aceptación. Una regresión en la conexión de la apertura al resultado tipado o del join al resultado público podría dejar esas pruebas verdes. La separación `Unavailable`/`Err` es precisamente el comportamiento que necesita Workflow para no convertir fallos internos en disponibilidad documental negativa.

Corrección requerida: introducir una inyección privada, acotada y exclusiva de tests, por instancia, en el límite de apertura usado dentro del trabajo real de `prove_access`. Invocar el puerto público con un registro y raíz sintéticos válidos para comprobar AccessDenied → `Ok(Unavailable(AccessDenied))`, fallo genérico → `Err(StorageUnavailable)`, panic real de ese trabajo → `Err(StorageUnavailable)` y fallos artificiales de guardas `NotFound`/`StorageUnavailable` → `Err` sin degradación. Comprobar el wrapper `open_verified` para las mismas causas, no sólo `legacy_error`. Conservar los tests reales actuales y los mappings unitarios; no se necesita otro framework ni una ACL real. Precisar después el informe de entrega según la evidencia realmente ejecutada.

Este hallazgo describe una carencia verificable del gate, no un fallo runtime demostrado. El código inspeccionado hace actualmente la traducción correcta; la corrección no requiere cambiar su comportamiento de producción.

## Conformidad observada

| Requisito T04a | Evidencia en el corte | Evaluación |
|---|---|---|
| ABI `DocumentProofAccess` y enums sin cambios wire/Reader | `application/reader_ports.rs:31`; firmas previas conservadas | Conforme |
| Una sola apertura y guardas | `open_verified` delega a `prove_access`; núcleos compartidos de directorio/archivo | Conforme |
| Sólo tres causas OS como Unavailable | `classify_ntstatus`, fallback de apertura y `map_document_open_failure` | Conforme por inspección; mappings unitarios presentes |
| Guardas/metadata/tamaño y join siguen siendo errores | Conversión `From<AppError>` a Fatal; tamaño devuelve IntegrityFailure; join se aplana a StorageUnavailable | Conforme por inspección; falta inyección por puerto indicada arriba |
| Ausencia real de archivo y de directorio intermedio | `reader_integration.rs:891`, `prove_access`; además wrapper legacy para archivo ausente | Cubierto |
| Available contiene handle retenido | `reader_integration.rs:925`; test NTFS en `:961` conserva ese handle y verifica bloqueo de la segunda apertura | Cubierto |
| SharingViolation real y error Reader legacy | `reader_integration.rs:961` y prueba nativa de `managed_files.rs` | Cubierto con fixture sintética |
| Tamaño divergente y registrado fuera de rango | `reader_integration.rs:925` | Cubierto por puerto |
| Regresión Reader/propiedad/protocolo | Suites existentes siguen en el corte; wrapper usa el mismo recorrido | Evidencia de ejecución aportada por revisión Rust/orquestador; no repetida aquí |
| Sin nuevos permisos, share flags, filesystem UI ni lectura/hash completa para proof | Diff de producto y recorrido revisados | Conforme |
| Alcance acotado | Cuatro archivos de código y reporte; sin Workflow, schema, composición ni UI | Conforme |

La prueba existente de junction (`document_protocol.rs:129`) sólo exige que Reader devuelva error; es una regresión de rechazo, no prueba por sí sola la distinción tipada de un error fatal frente a Unavailable. La documentación del puerto debería dejar clara esa distinción y que una apertura fallida no acredita las guardas que requieren el handle. No elevo la ausencia de rustdoc a un hallazgo independiente de bloqueo.

Los nombres internos `ManagedOpenError::{Os,Fatal}` y `*_for_document` difieren de los ejemplos de `TASK04_PORTS`, pero mantienen la semántica y el núcleo único requeridos; no identifico una incompatibilidad de contrato por esa elección.

## Validación y límites

Se leyeron AGENTS, INTENT, STATUS, brief central, TASK04_PORTS, ADR-018 y criterios QUALITY relevantes; se inspeccionaron el diff BASE..HEAD, el recorrido completo de apertura, funciones llamadas y pruebas relacionadas. `git diff --staged` y `git diff` no produjeron cambios. Se verificó de nuevo HEAD y limpieza al finalizar la inspección.

Por instrucción del orquestador, no repetí compilaciones ni pruebas: los resultados de check/clippy/fmt y las 137 entradas Rust proceden de la revisión Rust ya comunicada. Este informe acredita revisión estática y suficiencia de cobertura, no una nueva ejecución de esos gates. Tampoco acredita fallback no Windows, ACL real, instalación ni Workflow PRE/P1.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 1 | warn |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: **WARNING** — un hallazgo HIGH de cobertura requerida. **Gate SPEC T04a: BLOCK**, porque AGENTS exige cerrar los hallazgos importantes antes del merge. Revisión terminada: **DONE_WITH_CONCERNS**. Cerrar la inyección por el puerto real y revisar su evidencia permite reevaluar el gate; no se exige reimplementar la apertura.
