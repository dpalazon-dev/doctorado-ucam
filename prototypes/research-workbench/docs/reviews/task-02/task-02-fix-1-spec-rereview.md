# T02 fix1 — rereview de SPEC y calidad

**SPEC: Issues found. Calidad: Needs fixes.** HEAD `922a0669d771d6778cd3d264bcc5be91f0e96ec6`, FIX_BASE `45c4c8c48269495b74241004f92f68acf4770708`. Revisión limitada a F1–F12/M1–M4 y defectos de su delta; no reabre Reader/UI T03 ni el conjunto inicial de cambios.

## Strengths

- Replay ya no renueva autoridad (`src-tauri/src/modules/library/service.rs:176`); su prueba reconstruye servicio y comprueba rechazo de confirm/cancel (`tests/library_integration.rs:772`).
- El resultado de recovery llega a un estado compartido con Settings (`src-tauri/src/lib.rs:61`, `application/settings.rs:49`, `adapters/sqlite/settings.rs:60`). Incidencia aislada y fallo global tienen políticas distintas sin cambiar wire.
- Hay ABI transaccional real para confirmar, editar y archivar/restaurar (`application/library.rs:17`, `:70`, `:136`), repositorio SQL explícito y prueba de rollback desde una TX del consumidor (`tests/library_integration.rs:1793`). T04 ya no necesita dos commits para aplicar archive; faltan las partes señaladas en F3.
- Recovery de reuseExisting valida y limpia antes de COMMITTED (`modules/library/service.rs:516`). El segundo buffer está acotado, verifica tamaño/hash y utiliza el handle retenido (`adapters/documents/store.rs:214`, `adapters/windows/managed_files.rs:256`).
- Las sagas tienen propietario independiente del Future del caller, y el registry de requestId mantiene el orden requestId→token; la adquisición/poda del Weak se hace bajo el mismo mutex (`modules/library/service.rs:101`, `:148`). Las pruebas de caller drop usan barreras y el Store real envuelto.
- Las fixtures de caja ahora regeneran xref/EOF, prueban herencia real y separan el ciclo Parent del ciclo Kids (`tests/library_integration.rs:258`, `:939`). Archive/restore comprueba una dependencia persistida real de 0001 (`:1829`).

## Cierre por hallazgo

| ID | Estado en SPEC/calidad | Evidencia y límite |
|---|---|---|
| F1 | ADDRESSED | `modules/library/service.rs:176`; replay devuelve receipt sin remember_token. Prueba `tests/library_integration.rs:772`. |
| F2 | ADDRESSED | `lib.rs:61`, `application/settings.rs:49`, `:81`, `adapters/sqlite/settings.rs:48`; prueba COMMITTED ausente `tests/library_integration.rs:1907`, fallo global y enumeración real fallida. Hash cambiado/ambigüedad siguen el mismo informe retenido; no hay una prueba nueva independiente para cada variante. |
| F3 | NOT ADDRESSED, parcial | ABI *_in_tx sí existe y rollback está probado. Persisten reglas de preparación en adaptador y los helpers no aplican validación/auditoría completas; detalle Important 1. |
| F4 | ADDRESSED | `modules/library/service.rs:516` aplica validate_cleanup_plan y cleanup antes del commit; prueba de corte tras preparar reuseExisting en `tests/library_integration.rs:1150`. |
| F5 | ADDRESSED | `adapters/documents/store.rs:214` verifica buffer MAX+1/tamaño/hash sobre el mismo ManagedFile; prueba determinista de crecimiento después de copia `:431`. No se interpreta como cuota total RAM del parser. |
| F6 | ADDRESSED en alcance SPEC | `adapters/documents/pdf_probe.rs:4` resuelve herencia Parent con detección de ciclo; fixture válida/ausencia/degeneración/ciclo con xref correcto `tests/library_integration.rs:939`. El límite propio del parser no se convierte en soporte universal. Juicio de seguridad del parsing corresponde al gate Rust. |
| F7 | NOT ADDRESSED, parcial | Se retiró cleanup DONE ficticio y se propagan remove/sync/read. Sigue faltando resultado durable/cierre de intención cuando la ausencia sí está probada; detalle Important 2. |
| F8 | ADDRESSED | `modules/library/service.rs:148`, `:211`, `:318`, `:419` trasladan saga/permits al task dueño; pruebas caller-drop y recovery en `tests/library_integration.rs:1356`, `:1415`. No se repitió lifecycle suite. |
| F9 | ADDRESSED en casos exigidos | Exclusión asíncrona `modules/library/service.rs:101`; antes de efectos llama previous_selection/confirmation/cancel. Pruebas compatibles/incompatibles `tests/library_integration.rs:1463`, `:1522`. |
| F10 | ADDRESSED en conformidad de composición; gate seguridad/Rust pendiente | Actor liga raíz antes del probe (`adapters/sqlite/actor.rs:49`) y comparte esa instancia; Store exige binding (`adapters/documents/store.rs:24`). Handles retenidos y no-clobber `adapters/windows/managed_files.rs:48`, `:505`; pruebas preservan padre/destino. No se certifica aquí FFI/TOCTOU ni pérdida de energía. |
| F11 | NOT ADDRESSED por regresión de delta | Namespace/referencias ahora se comprueban, pero la condición de destino rechaza un plan legítimo de import nuevo con UUID/destino reservados; detalle Important 3. |
| F12 | ADDRESSED | `tests/desktop_bootstrap.rs:193` registra plugin dialog real, permite Library con selector sintético y verifica motivo específico de rechazo. fs es handler ausente, no se afirma ACL aislada ni GUI. |
| M1 | ADDRESSED | `tests/library_integration.rs:258`, `:939`; Root ya no se invalida también por borrar EOF. |
| M2 | ADDRESSED | `tests/library_integration.rs:1829` comprueba document/autores/reading_position y describe exactamente su alcance. |
| M3 | ADDRESSED | `adapters/sqlite/library_repository.rs:192`, `:211` usa mapping exhaustivo de enums al filtrar. |
| M4 | ADDRESSED en alcance API | `adapters/documents/store.rs:195`, `:350`, `:355` propaga sync; helper Windows no descarta la barrera. No prueba pérdida de alimentación. |

## Issues

### Critical

Ninguno identificado en esta revisión de SPEC/calidad.

### Important

1. **F3 sigue incompleto: el puerto transaccional permite composición, pero no concentra todas las reglas/auditoría exigidas.** `src-tauri/src/adapters/sqlite/library_repository.rs:380` mantiene `prepare_import` privado con comprobación de estado/expiración/payload, decisión reuseExisting y reserva de UUIDs; su wrapper continúa llamando esa política en infraestructura, en lugar de un helper de aplicación. Además `src-tauri/src/application/library.rs:17` acepta `PaperMetadataInput` y escribe sin normalizar/validar, y los helpers de mutación (`:17`, `:70`, `:136`) carecen de emisión de auditoría y de un puerto/contexto para ello. La única emisión es el wrapper externo `adapters/sqlite/receipts.rs:62`: el consumidor cruzado que usa una Transaction prestada, como el test `tests/library_integration.rs:1793`, puede confirmar cambios sin el audit Library. TASK02_PORTS asigna a los helpers validación/duplicados/transiciones/auditoría; el fix brief exige reglas fuera de adaptador. Extraer también la preparación detrás del LibraryRepository, establecer input validado o validar dentro del helper y emitir audit de entidad/cambio dentro de la misma TX del consumidor (sin duplicarlo en replay). Ampliar prueba de rollback a audit y un rechazo de metadatos inválidos. El problema original de dos commits se ha corregido; este incumplimiento restante no se dispensa por ello.

2. **F7 transforma rechazos normales en incidencias STAGING permanentes sin motivo durable.** `src-tauri/src/modules/library/service.rs:196` devuelve el error de stage sin registrar su resultado. `adapters/documents/store.rs:191` y `:202` ya saben si eliminar el handle propio tuvo éxito, pero esa prueba se pierde al devolver solo AppError. Incluso source inexistente (sin copia), PDF inválido/cifrado eliminado o exceso de tamaño eliminado queda `STAGING`, hash y error_json NULL. Al reiniciar no hay confirmación ni token autorizable, y recovery devuelve siempre recoveryRequired: no puede completar ni cancelar esa intención desde ninguna operación normal. `tests/library_integration.rs:1038` fue cambiada para afirmar ese estado y el issue permanente; no demuestra una corrección del flujo. Es adecuado retener la intención cuando sync/read/remove o propiedad son inciertos, pero no cuando el propio adaptador conoce la ausencia/eliminación inequívoca. DATA permite terminar esos incompletos con resultado registrado; TASK02_PORTS reserva los drafts pendientes para casos que no pueden cancelarse inequívocamente, y F7/MANAGED_FILES exigen conservar la prueba. Devolver un resultado interno tipado de staging fallido con cleanup probado/no probado, o reconciliar ausencia de forma verificable; persistir FAILED/resultado/error y cleanup DONE solo con prueba, mantener PENDING/issue para fallo real. Cubrir invalid/cifrado/source ausente/>MAX y reopen, además de fallo de eliminación. No es plan-mandated: el test verde actual consolida una limitación no autorizada como dispensa.

3. **Nueva regresión F11: cancelación legítima después de reservar destino falla antes del filesystem.** `src-tauri/src/adapters/sqlite/library_repository.rs:705` calcula `no_destination_is_consistent` como `reserved_document_id.is_none() || reuseExisting`, y `:737` lo exige incondicionalmente. Después de preparar una confirmación nueva, la fila tiene reserved_document_id válido y destination_path=`documents/<id>/original.pdf`, con metadata kind=confirmation sin reuseExisting: la expresión es false aunque staging exista con su hash correcto y el destino aún esté ausente. `prepare_cancel` marca FAILED y validate_cleanup_plan devuelve ImportRecoveryRequired, bloqueando cancelación/cleanup y también su recovery posterior. Aplicar la regla de ausencia solo cuando destination_path sea None; con Some verificar UUID/path reservado y referencias vigentes como ya hace destination_is_consistent. Regresión requerida: preparar confirmación nueva sin promover (o inyectar fallo antes de promoción), cancelar y comprobar receipt/FAILED cleanup DONE, staging retirado, original intacto; conservar rechazo si destino realmente existe o está referenciado.

### Minor

1. **Precisión del informe de composición para esquema futuro.** `docs/reports/task-02-fix-1-report.md:45` dice que writable=false no construye Store; `src-tauri/src/lib.rs:33` sí construye LocalDocumentStore antes de decidir si inicia recovery. Su constructor únicamente valida binding/UUID y no hace I/O, por lo que no se observó un incumplimiento de preservación de raíz. Corregir el informe a «construye Store sin I/O y omite recovery»; no presentar ausencia de construcción como evidencia.

## Cannot Verify / evidencia

Se leyeron los logs y exitcodes finales locales: fmt 0, cargo-test 0 (81 entradas: 6 unit + 75 runner, incluido helper), Clippy 0 sin warnings y build Tauri Windows debug 0. `library_integration` pasa 41/41; los escenarios de Important 1/3 no están cubiertos y el caso de Important 2 afirma precisamente la limitación. Evidencia UI de T02 sin cambios TypeScript: no se atribuye un nuevo gate agregado completo a fix1. Root ejecutará la validación integrada pertinente tras aprobar; no se repitieron suites para esta revisión.

No se observó GUI del selector, instalación/equipo limpio, pérdida de energía ni render total. La conformidad anotada de F10/F6 no sustituye sus revisiones especializadas. No se verificó un log rojo por cada fix; el informe del autor detalla verdes reales y límites. No se usó memoria auxiliar como autoridad.

Paquete de delta leído por porciones, sin regenerar Git. Se recuperó el hunk de service truncado en la salida. Dos comprobaciones focales fuera de los hunks completos: `prepare_import` actual (la función estaba cortada por el delta, riesgo F3 preparación aún en adaptador) y `receipts.rs` (riesgo concreto nuevo de audit en helpers prestados, solicitado por root). Búsqueda `rg -n` únicamente para localizadores. Normativa principal consultada: fix brief/context/report, MANAGED_FILES y TASK02_PORTS resolución 7. Ningún subagente, edición de producto/Git o suite ejecutada. Única escritura: este informe.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH / Important | 3 | warn |
| MEDIUM | 0 | info |
| LOW / Minor | 1 | note |

**Verdict: WARNING. F3/F7/F11 siguen NOT ADDRESSED; SPEC Issues found / calidad Needs fixes.**
