# T04b fix1 — seguridad estática del runtime

**Dictamen final: PASS de seguridad estática en `5fb4e084ed943831e2d8aa2bc3584ae076590836`.** Cero riesgos nuevos concretos encontrados en extracción transaccional, replay, auditoría, validación por pin o propiedad de proof/permisos. La precisión de pin destino quedó corregida en el runtime final. Este PASS no sustituye aceptación SPEC, S6/S7, tests, gate, build ni integración.

## Corte y alcance

- Fecha: 2026-10-03.
- BASE de runtime previamente revisado: `a051d88ca4c8127fe4749293b0b03cecc2923a14`.
- Corte intermedio inicialmente revisado: `1941f45392e9221e4b26e1918cf57cf0d1a8000b`.
- HEAD inmutable final: `5fb4e084ed943831e2d8aa2bc3584ae076590836`.
- Paquete central leído: `task-04b-security-fix1-1941f45.diff`; contrastado con `git diff BASE HEAD -- src-tauri/src` y contenido mediante `git show HEAD:<ruta>`.
- Fuentes: brief correctivo `task-04b-fix1-brief.md`, TASK04_APPLICATION_CASES, ADR-021 e informe de seguridad anterior. El NEEDS_CONTEXT del informe original es histórico, no evidencia del resultado de esta corrección.
- Sólo se revisó el delta de los cinco archivos de runtime de abajo. Se consultaron las primitivas Library reutilizadas para interpretar su composición, sin ampliar a una auditoría general.
- Sin WIP, tests/build/cargo, GUI, instalador, nuevos scripts, cambios de normas/producto, commits, merges, subagentes o bibliotecas personales. La única escritura propia es este informe central; se conservan cambios ajenos.

| Ruta | Blob revisado en HEAD |
|---|---|
| `src-tauri/src/adapters/sqlite/workflow_repository.rs` | `63a5b356603ff47d34312fe01ca4910f9ea7ef40` |
| `src-tauri/src/application/workflow.rs` | `1eb7f474a50249ac32c215b9e7ddab4df6cef3b7` |
| `src-tauri/src/application/workflow_ports.rs` | `de576a727fbfdbdd9c5653b6331a1d4569388a9f` |
| `src-tauri/src/domain/workflow.rs` | `73bbce00ddf2baac152e72fe5f62bb3170798b96` |
| `src-tauri/src/modules/workflow/service.rs` | `d0434d4e19f683e1249727a7ba55eb67644d5ca1` |

Comparación final realizada mediante `git ls-tree` y `git diff 1941f45392e9221e4b26e1918cf57cf0d1a8000b 5fb4e084ed943831e2d8aa2bc3584ae076590836 -- src-tauri/src`: cuatro blobs permanecen idénticos; sólo cambia application/workflow.rs con tres líneas de guarda de pin destino. Si un corte posterior cambia el runtime, requerirá revisión del delta. Las referencias de la sección siguiente describen el primer corte fix1; después de la guarda nueva las líneas posteriores se desplazan +3.

## Comprobaciones

**Frontera TX/application.** `application/workflow.rs:110`, `:226`, `:348`, `:482` y `:568` reciben TX prestada y repositorios. No abren/cierran transacciones, consultan actor/receipt, ejecutan SQL/filesystem ni poseen handles/permisos. El adaptador conserva DbActor, with_receipt IMMEDIATE y evaluate DEFERRED; action llama al caso de uso y serializa el resultado antes de confirmar receipt. Las cuatro primitivas nuevas (`workflow_repository.rs:157`, `:177`, `:195` y answers) persisten/mapean valores decididos, con parámetros SQL; aceptación/contexto exigen exactamente una fila. No se añadió otra conexión, UoW o mecanismo de persistencia.

**Replay/capability.** Save ya no rechaza P2 fuera de with_receipt: la decisión de capacidad está en el caso de uso después del lookup. Advance conserva guardas de admisión y resuelve lookup en `workflow_repository.rs:446` antes de capability y referencia/proof. El lookup final sigue dentro de with_receipt. Un replay confirmado no llama a application ni crea eventos específicos; una colisión no llega al proof. El delta del servicio elimina únicamente la guarda de capability prematura, sin alterar spawn/oneshot o ensure_mutable.

**Proof/permisos.** `workflow_repository.rs:364`, `:479` y `:537` conservan admission y proof en la closure exterior alrededor de with_receipt o de la lectura DEFERRED, incluyendo commit y el retorno de error. La revalidación completa de referencia Available/Unavailable sigue antes del caso de uso, bajo esa misma TX. Application recibe referencia/bool ya verificados y no infiere disponibilidad desde errores. El trabajo propietario save/goBack continúa esperando al actor con sus permisos; la extracción no añade un punto de cancelación que abandone una mutación encolada ni mueve guardas al action.

**Pins y límites.** `application/workflow.rs:46` resuelve la versión de la fila; pin no soportado se traduce a UnsupportedCapability, sin reinterpretar como v1. Save/evaluate/aceptación usan esa resolución, y P1 resuelve también el pin PRE de su cadena. Snapshot/Gate/PhaseDto usan la versión persistida. Touch resuelve el pin destino antes de mutarlo. Los casos nuevos mantienen CAS previo al no-op y rechazan expectativas fuera de 0..2^53−1; overflow de clocks/revisión falla dentro de la TX. Este foco no sustituye el cierre contractual integral S3 por el revisor SPEC.

**Auditoría y atomicidad.** `application/workflow.rs:318`, `:391`, `:547` y `:714` usan acciones constantes, Paper ID y requestId. Los cambios de respuesta conservan before/after y fases; aceptación conserva revisión/state/completedAt/hash, decisión, contexto/lifecycle y destino; navegación conserva contexto/destino. Los contenidos científicos permanecen en el audit local solicitado, no en logs/errores. El nuevo no-op retorna antes del put/invalidation/audit específico; el no-op de contexto no crea evento efectivo. Cualquier error al insertar estos eventos se propaga antes de devolver, por lo que with_receipt no confirma éxito y su TX revierte las escrituras previas. La auditoría genérica legacy permanece fuera del caso de uso.

**Lifecycle/plan.** El plan puro decide sólo destino/estado preservado y NEW→ACTIVE. Application conserva cadena vigente y CAS, asigna clocks, aplica aceptación/contexto y usa update_lifecycle o archive_paper_in_tx sobre la TX común. Archive no crea un wrapper Library/receipt anidado; conserva su auditoría Library en la misma operación. El nuevo acceso a LibraryRepository::paper no filtra el documento por ACTIVE, por lo que un registro coherente MISSING/SUPERSEDED continúa pudiendo evaluarse como indisponible. No se debilitan las restricciones de archivados/COMPLETED ni el mecanismo existente de recovery/esquema futuro.

## Hallazgos y evidencia

Hallazgos nuevos de seguridad/integridad: **CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 0**. No hay una fila nueva de severidad/path/línea/escenario que deba añadirse al brief correctivo. Las precisiones y escenarios SPEC ya gestionados conservan su revisión especializada; este informe no los recodifica ni los declara cerrados por ausencia de una vulnerabilidad adicional.

**Precisión SPEC identificada y coordinada con Sol, cerrada estáticamente en el delta final:** el corte 1941f45 aplicaba el plan de destino sin resolver su pin antes de aceptar origen. Escenarios: PRE v1→P1 con pin v2 no soportado, o P1 continue→P2 con pin v2 no soportado. Sol confirmó que ambos deben fallar antes de cualquier aceptación/mutación y precisó TASK04_APPLICATION_CASES en `main4923500`, manteniendo ABI. En HEAD final, `application/workflow.rs:630` llama `pinned_definition(&records, phase)?` cuando hay destino, antes del plan, clock, aceptación, lifecycle/contexto o audit. Usa las filas leídas dentro de la TX y el mismo helper que devuelve UnsupportedCapability para pin no soportado; el error vuelve a with_receipt sin éxito. Las ramas light_read/archive no tienen destino y conservan su comportamiento. No se alteraron las closures de proof/permisos, replay ni revalidación documental.

Se leyeron las dos regresiones añadidas en el diff final de `workflow_ipc.rs`: `advance_rejects_unsupported_p1_destination_pin_before_accepting_pre` vuelve realmente a PRE y usa su revisión vigente; `advance_rejects_unsupported_p2_destination_pin_before_accepting_p1` usa P1 activa/continue y pin P2 futuro. Ambas esperan UnsupportedCapability y comparan lifecycle/contexto, estados/revisiones/completedAt/snapshots y conteo de aceptaciones, además de ausencia de receipt del request rechazado. Son pruebas sintéticas leídas, no ejecutadas por este revisor. Las demás ampliaciones finales de acceptance/migration no se presentan como aprobación de seguridad o cierre SPEC por este informe.

**Pruebas ejecutadas por este revisor: ninguna.** No se revisó íntegramente la suite final ni sus logs y no se atribuye PASS ejecutable a las aserciones leídas. Los resultados anteriores de gate/build eran declaraciones transcritas del autor según el informe histórico. Para este corte final, Sol comunica gate/build exit0 con logs reales comprobados por root; se registra como evidencia comprobada por el orquestador, no como ejecución o inspección independiente de logs de este revisor. La aceptación/merge corresponde a Sol tras integrar todas las revisiones y comprobaciones exigidas.

Entrega final: **PASS de seguridad estática sobre los cinco blobs de `5fb4e084ed943831e2d8aa2bc3584ae076590836` indicados**, incluido el delta de pin destino; ninguna aprobación de merge o producto instalado.
