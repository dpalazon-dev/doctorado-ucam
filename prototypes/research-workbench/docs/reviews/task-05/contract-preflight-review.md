# T05 — revisión documental independiente de contratos

2026-10-03. **Resultado: BLOCKED para publicar el ABI tal como está escrito.** Revisión documental terminada; cinco correcciones importantes abajo. Sol comunicó durante la revisión que acepta su dirección, pero esa comunicación no sustituye la publicación normativa ni demuestra implementación. T05 no queda activada por este informe.

## Alcance y evidencia

Se leyeron AGENTS.md, INTENT.md, docs/STATUS.md, task-05-ports-proposal.md, task-05-orchestrator-resolutions.md, task-05-preflight.md y el brief canónico T05. Contraste acotado con DOMAIN (agregados, conocimiento, P2, relaciones, archive), DATA (representación, modelo, UoW, receipts, auditoría y migraciones), CONTRACTS (Knowledge/Concept/Provenance, límites y export), SPEC-004 y WORKFLOW_GATES (proyección y revisiones). Los encabezados/consumos de los briefs T06/T07/T08 se consultaron sólo para los límites consumidores.

Runtime leído exclusivamente mediante `git show 4162ac1355d214c7d5d47c4aa96695c526b2e3d3:<path>`: application/ports.rs, application/library_ports.rs, adapters/sqlite/receipts.rs y transport/dto.rs. Ese corte no es BASE de T05 ni demuestra helpers Workflow implementados. No se leyó runtime WIP T04b. TASK04_PORTS se usó como interfaz publicada, no como evidencia runtime.

No se ejecutaron SQL, tests, builds, instalaciones, commits ni merges. No se cambiaron normas, producto, configuración global o AGENTS.md, que estaba modificado externamente. No se crearon subagentes. OpenViking actor no verificado: sólo fuentes locales; sin recuperación ni escritura de memoria de proyecto. Karpathy se aplicó a correcciones mínimas y Superpowers verification-before-completion a la comprobación del informe, no a una afirmación de funcionamiento del producto.

## Hallazgos importantes que bloquean publicación

### R1 — Lifecycle no recibe identidad de petición para auditoría

**Referencias:** proposal:193–197 y 237; resolutions:13; DATA:67,79. `set_item_lifecycle_in_tx` recibe item/revisión/estado/fecha, pero no requestId, mientras KnowledgeRepository.audit_change lo exige. Una función de aplicación no puede producir su auditoría correcta sin recibir esa identidad. Sacar el evento al adaptador contradice la coordinación de la UoW en aplicación.

**Corrección mínima propuesta:** añadir `request_id: &str` a esa función, conservar la misma transacción prestada y el wrapper receipt existente. Los comandos genéricos y Concept usan esa única lógica; no agregar otro caso lifecycle, receipt o actor.

```rust
pub fn set_item_lifecycle_in_tx(
    tx: &Transaction<'_>, knowledge: &dyn KnowledgeRepository,
    workflow: &dyn WorkflowRepository, request_id: &str, item_id: &str,
    expected_revision: i64, next: KnowledgeItemDtoLifecycle, now: &str,
) -> Result<KnowledgeItemDto, AppError>;
```

CAS precede no-op para una petición nueva; receipt replay precede ambos. El caso audita sólo el cambio efectivo y conserva contenido/enlaces/procedencia. La fecha se usa sólo al cambiar datos. Prueba futura: archive/restore válido con requestId exacto en el evento; stale no-op Conflict; replay exacto sin evento/clocks adicionales.

### R2 — Puerto de auditoría carece del contenido que explica el cambio

**Referencias:** proposal:219–220,237; DATA:79; `4162ac1:application/library_ports.rs::LibraryRepository::audit_change` y `adapters/sqlite/receipts.rs::with_receipt`.

El puerto propuesto sólo recibe item/action/time. No puede conservar qué nombre, definición, confianza, lifecycle o enlace cambió. El evento automático de with_receipt tiene entity_id NULL y changes_json `{}`: acredita petición, no historia de contenido. No debe contarse como sustituto del evento de modificación.

**Corrección mínima propuesta:** añadir `changes: &serde_json::Value`, construido por el caso de aplicación con los valores pertinentes anteriores/nuevos. Reutilizar la primitiva SQL de auditoría existente cuando sea posible; no introducir una dependencia del repositorio Knowledge sobre todo Library únicamente para reutilizar un nombre.

```rust
fn audit_change(
    &self, tx: &Transaction<'_>, request_id: &str, item_id: &str,
    action: &str, changes: &serde_json::Value, now: &str,
) -> Result<(), AppError>;
```

La aplicación fija acciones cerradas y datos mínimos que expliquen cada operación; el adapter sólo persiste. Para link, registrar IDs del vínculo agregado y revisión anterior/nueva del item; para contenido/lifecycle, campos cambiados y sus valores. El historial permanece en biblioteca/backups, fuera de logs. No-op y replay no producen modificación de contenido. Prueba futura: rollback elimina datos, evento, receipt y clocks juntos; evento de una edición explica el valor anterior/nuevo.

### R3 — Faltan los casos TX de edición y enlace en el ABI de aplicación

**Referencias:** proposal:155–173 frente a 183–197,202–239; preflight:9,112; DATA:56,63,67; CONTRACTS:330,332.

Persistence ofrece update_item/update/link, pero application sólo declara captura, createConcept y lifecycle. Esa omisión permite que el autor resuelva CAS, restricciones de Concept, no-op, guardas archived, conjunto afectado e invalidación dentro del adapter SQL. La declaración de responsabilidades no deja una interfaz concreta revisable para esos tres comandos.

**Corrección mínima propuesta:** publicar estos tres casos, con los repositorios ya propuestos y sin nuevo framework. Args existentes llevan requestId. El wrapper de persistence abre una única TX con receipt y llama al caso; repositorios no hacen commits ni deciden política.

```rust
pub fn update_item_in_tx(
    tx: &Transaction<'_>, knowledge: &dyn KnowledgeRepository,
    concepts: &dyn ConceptRepository, workflow: &dyn WorkflowRepository,
    args: &KnowledgeUpdateItemArgs, now: &str,
) -> Result<KnowledgeItemDto, AppError>;

pub fn update_concept_in_tx(
    tx: &Transaction<'_>, knowledge: &dyn KnowledgeRepository,
    concepts: &dyn ConceptRepository, workflow: &dyn WorkflowRepository,
    args: &ConceptUpdateArgs, now: &str,
) -> Result<ConceptDto, AppError>;

pub fn link_concept_in_tx(
    tx: &Transaction<'_>, knowledge: &dyn KnowledgeRepository,
    concepts: &dyn ConceptRepository, workflow: &dyn WorkflowRepository,
    args: &ConceptLinkArgs, now: &str,
) -> Result<(), AppError>;
```

`update_item_in_tx` decide CAS del item, subtipo/patch, confidence-only para Concept, subset Insight, estado editable y no-op. `update_concept_in_tx` decide CAS compartido, canon/mirror/alias/domain y no-op. `link_concept_in_tx` decide existencia/subtipo/estado, adición o no-op; el wire linkConcept no lleva expectedRevision y no se amplía: lee la revisión actual dentro de la TX IMMEDIATE y sólo una adición real incrementa la revisión del item. Los tres recopilan alcance antes/después, auditan cambios y llaman una vez/Paper al helper publicado.

Los puertos repository actuales bastan: item/concept, write_item CAS, write_details, add_concept_link y affected_papers, más audit corregido. No hace falta un command repository paralelo ni guardar borradores en un plan genérico.

### R4 — Consulta inversa describe una expansión por relaciones prohibida por la resolución

**Referencias:** proposal:231 frente a resolutions:11; WORKFLOW_GATES:63; CONTRACTS:434; proposal:245–247.

La frase «closure incorpora conceptos de relaciones» permite llegar a un extremo externo por una relación. La resolución posterior fija que las relaciones sólo entran si ambos extremos ya están alcanzados; jamás expanden alcance. Si `affected_papers` usa el primer criterio y el snapshot usa el segundo, una edición externa invalidaría Papers cuyo snapshot no contenía el artefacto, o la consulta inversa omitiría dependencias transitivas de conceptos.

**Corrección mínima propuesta:** sustituir esa frase por la definición ratificada: semillas = todos los paper_items del Paper; seguir item_concepts desde cada item alcanzado, incluidos Concepts usados como items; deduplicar por UUID hasta punto fijo, terminando ciclos. Relations sólo se incorporan con ambos extremos en el conjunto; no agregan endpoints. Insight.affectedConceptIds, por su subset, usa los mismos enlaces y no crea otro camino.

`affected_papers(item_id)` devuelve exactamente los Papers cuyo cierre anterior/posterior alcanza ese UUID, transitivamente y sin excluir archivados/NOT_STARTED. El helper decide qué clocks/estados renovar; una llamada por Paper. Pruebas futuras: cadena item→ConceptA→ConceptB y ciclo A↔B; editar B alcanza todos sus Papers; relación desde alcanzado A hacia C externo no incorpora C ni invalida ese Paper al editar C. Mantener archivados en la proyección/historia.

### R5 — Colisión de relación al cambiar contexto no tiene resultado definido

**Referencias:** proposal:129; resolutions:9; CONTRACTS:321–323,334; DATA:29; DOMAIN:41,125.

La unicidad activa propuesta es `(source,target,type,context)`. Se precisa Conflict en create/restore, pero updateRelation también edita contextText. Dos relaciones activas con contextos distintos pueden colisionar al cambiar el contexto de una. Si no se publica este caso, T07 puede devolver un error genérico de storage o cambiar datos/auditoría parcialmente.

**Corrección mínima propuesta:** la misma colisión en update devuelve Conflict y revierte la TX completa; no reemplaza/fusiona ni cambia identidad, provenance, revisions, clocks, eventos o receipt de éxito. La comprobación del caso T07 y el UNIQUE parcial se complementan; traducir sólo esa constraint conocida a Conflict, sin convertir todo error SQL en conflicto. Prueba futura: dos contextos activos; update de uno al canon del otro falla sin efectos; un registro archived no bloquea un contexto activo.

## Coherencia documental que sí queda sostenida

- **ABI pública:** seis métodos Knowledge y ocho Concept; exactamente catorce comandos del registry existente. DTOs/Args del corte estable permiten las firmas propuestas, incluido ConceptUpdateArgs.domain como Option<Option<String>> para preservar omitido frente a null. No se necesita cambiar wire ni generar nuevos comandos.
- **Atomicidad/idempotencia:** una TX IMMEDIATE; wrapper receipt durable ya existente; replay antes de CAS/guardas, CAS antes de no-op; no-op conserva contenido/revisiones/fechas/clocks/snapshots. Permisos en trabajo propietario hasta commit/rollback, aunque se pierda receptor. La publicación debe conservar ese orden en los casos añadidos; las pruebas siguen pendientes.
- **Captura:** padre y Provenance inicial revision0, Document real/hash registrado backend, sin expectedParentRevision inventado. Item/details/paper links/concept links/provenance/audit/receipt/clocks juntos. Paper association P2 clasifica sin iniciar/activar fase.
- **Concept:** nombre/title mirror atómico y definition/body único; attributes reconstruidos, misma revisión base; confidence editable por Knowledge con rechazo íntegro de otros campos de Concept. Homónimos permitidos, UUID estable, alias normalizado único intraconcepto. IDs son sets ordenados/deduplicados exactos según resolutions:6; esa decisión no permite deduplicar alias rechazables ni fusionar identidades.
- **Procedencia:** NONE conserva ausencia real sin fila ficticia; literatura NONE debe mostrarse pendiente de atribución en T06. Documento sin página = PENDING; página registrada válida con región opcional válida = LOCATED como registro de anclaje, sin prueba física/page count/cita. Locator cerrado, coordenadas finitas/bounds y página requerida para región; update futuro mantiene Document/capturedHash y STALE sticky. Discordancia con hash registrado impide LOCATED. checkHash no cambia Document/hash capturado ni revive estados.
- **Esquema/consumidores:** manifiesto0003 da una authority por atributo, doce tipos/trece relaciones/matriz core1.0.0, revision compartida, edges con FKs y sin cascada destructiva. Reserva validations(id) vacía no finge P3; ampliar exige migración nueva. 0001/0002/checksums se conservan; backup previo y0003 transaccional; FTS0004 pertenece a T08. Mapping export proyecta atributos y asociaciones canónicas, sin SELECT* ni filas ficticias.
- **Límites/normalización:** no se detecta una norma existente que obligue a otra forma de sugerencia léxica (prefix/substring); no justificar un cambio normativo con esa ausencia. Conservar máximos CONTRACTS:107, sin límites globales nuevos. El canon de IDs sirve a igualdad/no-op; no implica cambiar el algoritmo de hashing de receipts existente para considerar payloads distintos como replay. La UI no hace SQL/FS ni impone la authority del canon.
- **P2:** una edición de body de candidato afecta artefacto/clock, no necesariamente la proyección de candidatos ni PhaseAnswer.revision. Gate/aceptación final y candidatos siguen UnsupportedCapability hasta capacidades T06/T07; no declarar éxito P2 por captura o esquema.

## Sincronización obligatoria antes del despacho

Esta revisión evalúa decisiones propuestas, no trata toda diferencia con normas aún no publicadas como un defecto nuevo. Para publicar, Sol debe integrar R1–R5 y sincronizar CONTRACTS/DATA/DOMAIN/WORKFLOW_GATES según las decisiones que cambien semántica, además de los anexos/briefs T05/T06/T07 afectados. El brief T05:11 todavía contiene `candidate_revision_increments_with_artifact`; T05:13 todavía mezcla creación con parent revision y PENDING universal. La política archived debe quedar normativa: restore explícito para editar/enlazar desde padre archived; nuevas capturas/enlaces hacia Concept archived rechazados, referencias previas navegables/exportables.

Precisar al publicar el caso de requestId nuevo que repite un vínculo ya existente a Concept ahora archived: distinguir si la guarda rechaza todo intento o sólo una adición efectiva. Ambas lecturas caben en la redacción «nuevo enlace»; conservar replay previo en cualquier elección. No ampliar API para resolverlo.

Quitar `modules/knowledge/capture.rs` como mera delegación si application posee el caso único; transferir ownership application/domain-provenance/migrations/registro explícitamente. La secuencia T05a/T05b no requiere dos authorities ni un harness adicional. Limpiar en la versión publicable las decisiones que proposal:286–288 aún llama pendientes y resolutions ya cierra. DATA:36 debe precisar que el estado previo extra no es necesario para Item/Relation, cuyo único estado restaurable es ACTIVE.

El DDL/índices/CHECK exactos y sus fixtures requieren revisión del corte T05a antes de congelar0003. Esta revisión valida el manifiesto consumidor y señala los casos faltantes; no acredita un DDL ejecutable ni una migración probada. El nuevo autor recibe BASE sólo después de T04b/T04c integradas y contrastadas, nunca este corte estable por inferencia.

## Entrega y límites

Único archivo escrito por este reviewer: `.superpowers/sdd/IMPLEMENTATION/task-05-contract-review.md`. Commits: ninguno. Comandos de lectura principales: Get-Content por documento, `rg -n` por términos/secciones, `git show 4162ac1355d214c7d5d47c4aa96695c526b2e3d3:<path>`, `git status --short`. Se comprobó después el archivo guardado y el diff de whitespace. No hay resultados de producto que atribuir a esta revisión.

**Autorrevisión:** correcciones limitadas a responsabilidades existentes, argumentos mínimos y comportamiento consumidor derivado de sus campos actuales; sin framework, comandos nuevos, nuevo SQL ni decisión de producto externa. BLOCKED es estado de publicación documental, no de la ejecución completa del proyecto. Tras publicar las correcciones y sincronizar normas, corresponde una revisión documental final del conjunto publicado.
