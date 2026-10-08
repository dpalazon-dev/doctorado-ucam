# T04b — revisión independiente SPEC/calidad

Resultado: **BLOCKED para integración del corte revisado**. Revisión estática, 2026-10-03.

- BASE: `ce15c95dc6404aae1cb158ca6c4a1fe576b70505`.
- HEAD único inspeccionado: `5c35067f893aa52ec601cc7a02c18bb95b156145`.
- Autor: `.worktrees/task-04b-workflow-backend`; producto leído únicamente mediante `git show`/`git diff` del corte, sin leer WIP ni correcciones posteriores.
- Normas: AGENTS, INTENT, STATUS, briefs centrales T04b/review, TASK04_PORTS/ADR-018, WORKFLOW_GATES/ADR-017, CONTRACTS, DATA, DOMAIN, SPEC-003, QUALITY y ADR-020 actuales del repositorio principal.
- Diff: 31 archivos, 4342 inserciones y 69 eliminaciones. El diff local principal sólo mostraba instrucciones AGENTS ajenas a este producto; no se modificaron.
- No ejecuté tests, compilación, gate, GUI, instalador ni bibliotecas personales. No atribuyo resultados declarados por el autor. Las referencias de línea siguientes corresponden a HEAD, no al WIP posterior.

## Hallazgos HIGH / importantes

### S1 — PRE devuelve P1 sin iniciar ni activar P1 (conocido por Sol, abierto en este corte)

**Archivo:** `src-tauri/src/adapters/sqlite/workflow_repository.rs:766` (rama hasta 772).

**Escenario:** importar/inicializar Paper NEW; guardar seis respuestas PRE válidas; `advance(PRE)` con CAS vigente. El resultado anuncia `nextPhase=P1` y Paper ACTIVE, pero la DB conserva `current_phase=PRE` y P1 NOT_STARTED/revision0. Un save P1 falla GateBlocked por ADR-020; un advance P1 falla porque no está activo.

**Regla:** SPEC-003 y WORKFLOW_GATES, contexto/lifecycle: la aceptación PRE inicia el recorrido P1; el plan y el contexto deben confirmarse atómicamente. Brief §5 y recorrido IPC §7.

**Corrección mínima:** aplicar dentro de la misma TX el inicio de P1 si NOT_STARTED, conservar el estado/datos de una P1 previa y activar contexto P1 con clock fresco; no resetear fases anteriores ya iniciadas. Prueba por dispatcher que consulte Paper/P1 inmediatamente tras advance PRE, sin touch compensatorio; cubrir también reconfirmación de PRE después de invalidar una P1 ya iniciada.

**Evidencia de enmascaramiento:** `tests/workflow_ipc.rs:221` y `:277` llaman touch(P1) después del advance PRE. Sólo se comprueba nextPhase antes de ese touch. Esta prueba acepta el defecto que debería detectar.

### S2 — UNKNOWN PRE.review_type pasa aunque Paper tenga un tipo conocido (conocido por Sol, abierto en este corte)

**Archivo:** `src-tauri/src/domain/workflow.rs:55`; comparación con Paper sólo en `:62-76`.

**Escenario:** respuesta PRE.review_type UNKNOWN, `{reviewType:"unknown"}`, explicación válida; metadata Paper.reviewType cambia a survey. Tras el cambio, evaluate PRE vuelve a dar esa salida por procesada sin confirmar survey. También permite confirmar PRE NEEDS_REVIEW con el valor bibliográfico viejo.

**Regla:** WORKFLOW_GATES, definiciones/respuestas: la igualdad con Paper.reviewType actual aplica a esta salida tanto para UNKNOWN como para ANSWERED. Metadata obsoleta conserva respuesta y debe producir issue invalid hasta reconfirmarla.

**Corrección mínima:** validar concordancia estructurada antes de ramificar por resolución; conservar UNKNOWN+explicación sólo si el Paper sigue unknown. Pruebas conocidas→unknown y unknown→conocida, incluyendo save/evaluate/advance y preservación del snapshot anterior.

### S3 — El pin persistido no gobierna guardado, evaluación ni DTOs de transición

**Archivo:** `src-tauri/src/adapters/sqlite/workflow_repository.rs:554`, `:929`, `:653`, `:823`, `:1161`; contraste con getPhase `:842-853`.

**Escenario reproducible con fixture sintética:** añadir una fila válida `(PRE,2)` en phase_definitions y fijar una fase a version2, permitido por FK/esquema. getPhase devuelve definitionVersion2, pero save y evaluate seleccionan `definitions::get(...,None)` (v1), la evaluación escribe snapshot definitionVersion1 y advance devuelve version1. Touch también devuelve version1. No se exige implementar una definición v2 ahora: una versión fijada que el binario no soporte debe rechazarse explícitamente, nunca reinterpretarse como v1. La instalación posterior de otra definición no debe alterar los Papers fijados a v1.

**Regla:** REQ-003-01, CONTRACTS definición inmutable por Paper y DATA FK a versión exacta; brief §2 exige prueba de pin ante nueva definición. El pin debe ser la autoridad común de validación/snapshot/resultados.

**Corrección mínima:** cargar la fase/pin persistido y usar esa versión exacta para resolver la definición; devolver el pin en DTOs y usar el pin PRE en inputs de P1. Para pin no soportado, error seguro sin mutación/receipt exitoso. Añadir pruebas que instalen otra definición sin repinar el Paper y una fixture de pin no soportado que no pueda evaluarse silenciosamente con v1.

### S4 — Capability se comprueba antes de la colisión durable de requestId

**Archivo:** `src-tauri/src/modules/workflow/service.rs:245`; `src-tauri/src/adapters/sqlite/workflow_repository.rs:206`.

**Escenario:** confirmar un save PRE o advance PRE bajo requestId R; reutilizar R con el mismo comando pero phase P2. La forma wire es válida, pero retorna UnsupportedCapability antes del lookup, aunque el comando/hash almacenado exige Conflict. Para advance se evita incluso prepare_advance; para save se evita with_receipt. No se demuestra pérdida de datos, sí una respuesta contractual distinta y pérdida de la prioridad de colisión.

**Regla:** TASK04_PORTS §2 y brief §5: replay/colisión antes de capacidad actual, CAS/lifecycle/referencia/proof; mismo requestId con payload o comando distinto Conflict. P2 debe seguir indisponible para petición realmente nueva.

**Corrección mínima:** para advance, comprobar capability en prepare después del lookup y antes de referencia/IO; para save, comprobarla dentro de action tras lookup. Conservar admisión/validación wire y guardas de writability/recovery. Probar P2 nueva→UnsupportedCapability sin receipt y R confirmado reutilizado→Conflict sin proof.

### S5 — Falta historial identificable de respuestas y transiciones Workflow

**Archivo:** `src-tauri/src/adapters/sqlite/workflow_repository.rs:743`, `:986`; `src-tauri/src/adapters/sqlite/receipts.rs:57`.

**Escenario:** guardar una respuesta y luego cambiarla; aceptar PRE o P1 continue/light_read. Se sobrescriben respuesta/estado/contexto y sólo se añade la auditoría genérica heredada de with_receipt: `action=command`, `entity_id=NULL`, `changes_json={}`. No hay evento específico con Paper/fase/key/revisiones/cambio que explique el historial. Un payload_hash no permite recuperar la petición anterior. Sólo P1 archive obtiene adicionalmente el evento Library paper.archived.

**Regla:** DATA §3/§4 exige respuesta+revisión+evento y avance+historial atómicos; audit_events conserva identidad y cambios necesarios para explicar el historial. El brief pide no inventar modificación de contenido en no-op.

**Corrección mínima:** conservar el wrapper y su auditoría de petición legacy; añadir desde el caso de uso eventos específicos de cambio efectivo con identidad y valores/revisiones relevantes, dentro de la misma TX. No-op/replay no deben añadir un evento de cambio efectivo. Verificar eventos filtrando por su acción/entidad, sin confundirlos con la auditoría genérica de petición. Inyectar fallo tardío en esos eventos/receipt y comprobar rollback de respuesta, clocks, estado y contexto.

### S6 — La suite no acredita los escenarios de aceptación de CAS, cadena e historial

**Archivo:** `src-tauri/tests/workflow_ipc.rs:93`, `:251`, `:438`; `src-tauri/tests/workflow_gates.rs:35`.

**Carencia de evidencia, no afirmación de fallo de todos esos escenarios:** el test llamado `save_answer_persists_with_cas_replay_and_phase_clock` comprueba primera escritura y replay, pero no CAS stale ni no-op nuevo. Las ramas P1 insertan respuestas por SQL (`tx_insert_answer`) y compensan PRE con touch. El fixture goBack crea P1 IN_PROGRESS/revision0 por SQL, estado que la inicialización coherente considera inválido. Ningún recorrido IPC comienza con import real ni completa las salidas mediante save; DesktopState del suite tiene Library/Reader=None. No se cubren en estas suites:

- no-op normalizado con request nuevo y stale no-op, conservando clocks/timestamps/snapshot;
- metadata entre preview/advance→Conflict y authors-only sin invalidación;
- PRE suficiente IN_PROGRESS que no habilita touch(P1), y P1 no aceptada/NEEDS_REVIEW que no habilita P2;
- continue→P2→volver→editar P1→light_read/archive, preservando P2 NEEDS_REVIEW y sus datos;
- archive/restore vía Library sin volver a ejecutar archive guardado, y mutaciones COMPLETED reservado sin efectos;
- PDF inaccesible después de aceptación que cambia hash disponible, bloquea transición y conserva snapshot aceptado; acceso recuperado sin clocks ficticios;
- save/evaluate/advance P2 y ambos candidatos indisponibles sin receipt exitoso;
- snapshots invariantes por orden de claves/contexto/confirmación y sensibles a cada input real; overflow del clock y rollback sin consumirlo.

**Regla:** escenarios obligatorios de SPEC-003/WORKFLOW_GATES y brief §1/§4/§5/§7. Son pruebas de integridad y recorrido esenciales, no tests de detalles triviales.

**Corrección mínima:** añadir una fixture compuesta real de dispatcher con Library/Reader/Workflow y registry compartido; importar PDF sintético y llenar PRE/P1 mediante save. Derivar CAS desde respuestas reales; reservar SQL para estados futuros/inyección de fallos. Añadir escenarios anteriores con comprobaciones durables y reopen pertinentes. Los tests existentes sí acreditan fragmentos por dispatcher; no son una sesión Tauri instalada ni una prueba completa de import→PRE→P1.

## Hallazgos MEDIUM

### S7 — Upgrade/reopen tiene comparaciones parciales respecto al gate fijado

**Archivo:** `src-tauri/tests/workflow_migration.rs:238`, `:279`, `:329`.

**Carencia de evidencia:** fixture piloto contiene NEW, ARCHIVED desde NEW, COMPLETED y ARCHIVED desde COMPLETED; falta ACTIVE. La comparación principal cubre title/lifecycle/revision/updatedAt y size/status, pero no compara el UUID Document seleccionado (`actual.4`), hash/ruta, reviewType/domain, autores/venue, reading_positions/app_session. El reopen sólo comprueba versión/integridad; no vuelve a consultar fases/respuestas/snapshot aceptado. El backup tras rollback sólo se abre para user_version=1, no se compara su contenido con el piloto previo.

**Regla:** brief §2 exige matriz completa, igualdad bibliográfica/documental/lectura, backup anterior conservado y snapshot aceptado tras reopen. El SQL revisado conserva esas columnas; no se ha identificado aquí una pérdida concreta.

**Corrección mínima:** ampliar la matriz a ACTIVE y capturar/comparar las filas bibliográficas, asociaciones, documento, lectura y biblioteca antes/después/backup; cerrar y abrir de nuevo el actor/biblioteca y consultar contenido. Persistir aceptación real y comprobar payload/hash/clocks/contexto tras reopen. No sustituirlo por conteos ni reescribir SQL/checksums históricos.

### S8 — goBack/touch no rechazan expectativas por encima del entero seguro JS

**Archivo:** `src-tauri/src/adapters/sqlite/workflow_repository.rs:1023`, `:1093`.

**Escenario:** enviar por dispatcher `expectedActivePhaseRevision=9007199254740992`. DTO i64/validate_wire admiten ese entero y los helpers sólo rechazan negativos; devuelven Conflict sobre una revisión ordinaria en vez de InvalidInput. save/advance sí comprueban el máximo. No se demuestra overflow durable desde una DB sana; es una inconsistencia real del límite de entrada wire.

**Regla:** WORKFLOW_GATES clock y brief §4/§7: revisiones exactamente representables hasta 2^53−1; entradas fuera del límite inválidas.

**Corrección mínima:** aplicar la misma guarda de rango en goBack/touch antes de CAS y probar límite MAX, MAX+1 y negativo por dispatcher, sin cambios ni receipt nuevo para entradas inválidas.

### S9 — Los casos de uso de transición/guardado residen en el adaptador SQL

**Archivo:** `src-tauri/src/adapters/sqlite/workflow_repository.rs:673`, `:914`, `:1022`, `:1088`; `src-tauri/src/application/workflow.rs:1`.

**Desviación actual:** application contiene sólo initialize/invalidation. El adaptador toma la TX y decide CAS, habilitación, aceptación, cambio editorial, contexto, eventos y la llamada Library archive. `AdvanceDecision` del dominio decide la rama, pero no produce un plan de transición; no hay caso de uso application que componga ese plan con los repositorios. Esto deja las reglas de transición acopladas a SQLite y ya permite diferencias como S1/S3/S5. El dominio leído sí es puro (sin SQL/Tauri/filesystem/reloj), y WorkflowService no contiene SQL.

**Regla:** arquitectura explícita del brief y TASK04_PORTS: domain puro; application compone reglas/repositorios con TX prestada; adaptador SQL persiste y mapea DTOs. No es una petición de cambiar puertos/wire ni crear otro framework.

**Corrección mínima:** extraer casos de uso con la TX prestada al módulo application existente, y un plan mínimo de reglas/transición en domain; mantener las primitivas SQL en el adaptador y las firmas públicas aceptadas. Coordinar sólo las primitivas de repositorio indispensables con Sol, sin modificar contratos unilateralmente. Añadir pruebas del plan de primera aceptación y reconfirmación que no requieran instalar SQLite para decidir la rama.

## Comprobaciones estáticas y límites

- D1 implementa unión/dedupe de inputs y diferencia input directo/dependiente; asigna clocks en orden PRE/P1/P2 y conserva snapshot/completedAt. Los tests incluyen inputs solapados y P1 directo IN_PROGRESS.
- ADR-020 guarda NOT_STARTED después del replay de with_receipt, y editar PRE con P1 activa conserva contexto. El suite contiene ambos casos.
- Migración0002 es SQL-only, seed completo igual a JSON canónicos mediante test, backfill en runner transaccional; 0001 no cambia. Inicialización ya progresada no se resetea en el código revisado.
- Prepare advance/touch consulta receipt antes de referencia dentro del adaptador; la excepción anterior a ese prepare está en S4. Recheck final compara referencia completa para Available/Unavailable; no hay read_all/hash PDF masivo en workflow.
- Proof/permisos de advance/touch/evaluate permanecen fuera de action alrededor de with_receipt/lectura hasta commit/rollback. La composición reutiliza actor/store/RequestRegistry. Tests concurrency tienen abort/join efectivo y Drop que consulta el estado durable y adquiere un escritor tras commit/rollback; no les atribuyo un resultado de ejecución nuevo.
- P2 save/evaluate/advance y candidatos están capability-gated en código; lectura P2 disponible. Settings/manifest anuncian backend, no UI T04c/P2 completo/instalación.
- TypeScript cambiado es sólo el test de encaminamiento de ocho comandos mediante invoke simulado; mantiene candidatos sin invoke. Revisión especializada del lenguaje y ejecución del gate corresponden a sus revisores/orquestador.
- Los dos defectos conocidos permanecen abiertos **en 5c35067**, aunque un WIP/corte posterior los corrija. Este informe no revisa ni aprueba ningún delta posterior ni el reporte del autor aún pendiente.

Estado posterior comunicado por Sol: el delta `a051d88ca4c8127fe4749293b0b03cecc2923a14` corrige S1/S2 y conserva P1 previa en reconfirmación de PRE, con prueba nueva. No lo inspeccioné independientemente; esta información de coordinación no reemplaza la revisión del corte ni deja S1/S2 pendientes sobre ese delta. S3–S9 se entregan como hallazgos del corte original para su resolución y revisión posterior.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 6 | warn |
| MEDIUM | 3 | info |
| LOW | 0 | pass |

Verdict: **WARNING — 6 HIGH issues should be resolved before merge.** Estado de entrega de esta revisión: **BLOCKED**, conforme al gate del proyecto que impide integrar con hallazgos importantes abiertos. Ningún PASS de build/test/instalación se atribuye a esta revisión estática.
