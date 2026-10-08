# Dominio de Research Workbench

**Estado: baseline de ejecución v0.2, adoptada el 1 de octubre de 2026.** Este documento fija el lenguaje ubicuo de la baseline, límites e invariantes para contrastar con `ARCHITECTURE.md`, `DATA.md`, `CONTRACTS.md` y los SPECs antes de implementar. Las reglas marcadas como propuesta concretan decisiones que la especificación deja abiertas.

## Alcance y versiones

- **Piloto 0.0.1:** biblioteca local, importación y lectura de PDF, metadatos, reanudación, archive/restore y ciclo de vida del desktop. No demuestra PRE–P2 ni equivale a v0.1.
- **v0.1:** flujo manual PRE–P2 completo, captura tipada, conceptos globales, relaciones y procedencia, gates, búsqueda, export y backup/restore verificados.
- **Posterior:** P3, P4, reglas automatizadas y ayuda de IA; no alteran retrospectivamente el significado de datos guardados.

Los nombres de entidades y estados en inglés son claves estables de dominio/wire. La UI presenta etiquetas en español. SQLite es autoridad; PDF y otros binarios viven en biblioteca; índices y exports son derivados.

## Lenguaje ubicuo

| Término | Significado |
|---|---|
| Paper | Fuente bibliográfica y unidad de lectura; conserva metadatos, documentos y lifecycle. |
| Document | Versión concreta de un archivo fuente administrado, identificada por UUID y SHA-256. Los anclajes apuntan al Document, nunca solo al Paper. |
| Phase | Contrato versionado de trabajo (PRE, P1, P2, P3, P4) asociado a un Paper. No es el lifecycle editorial del Paper. |
| PhaseAnswer | Respuesta tipada a una pregunta/required output de una Phase. UNKNOWN y NOT_APPLICABLE son decisiones explícitas, no valores vacíos. |
| KnowledgeItem | Unidad semántica con UUID, type, contenido, origin, lifecycle y confidence. |
| Concept | KnowledgeItem global reutilizable entre Papers; no se duplica automáticamente por fuente. |
| Relation | Enlace dirigido y tipado entre KnowledgeItems, con origen/contexto propios. |
| Provenance | Rastro hacia un Document y localizador concreto o explícitamente pendiente. No implica que una afirmación sea verdadera. |
| Gate | Evaluación reproducible de salidas de una fase. Mide completitud declarada, no verdad científica. |
| Revision | Contador optimista del agregado afectado; se incrementa en cada mutación confirmada. |
| Archive | Retirada reversible de vistas activas, preservando identidad, contenido y enlaces. |
| Trash | Estado recuperable de retirada antes de una eventual eliminación; la eliminación física no está en el piloto ni en el contrato de v0.1. |

## Límites de agregado y consistencia

No son microservicios: estos límites organizan validación dentro del monolito. Propuesta operativa: un actor dedicado serializa comandos sobre una única conexión SQLite de escritura. Commands y queries son operaciones lógicas separadas, pero todos los cambios cruzados entre módulos usan el mismo `UnitOfWork` y una sola transacción/conexión; no abrir una transacción por módulo.

| Agregado | Raíz y propiedad | Invariantes y transacción |
|---|---|---|
| Paper | Paper | Metadatos, autores ordenados, venue, lifecycle y referencias a documentos/procesamiento; PhaseRecord posee sus respuestas y reglas propias. Crear/importar paper y autores/documento es una operación coordinada; no hay éxito visible hasta commit. Archive/restore preserva IDs y dependencias. |
| Document | Document | Hash inmutable por versión. La ruta relativa queda dentro de la raíz administrada. Sustituir archivo crea otra versión; no cambia silenciosamente anclajes existentes. |
| PhaseRecord | (paper_id, phase_code) | Versión de definición fijada al inicializar processing; respuestas e historial tienen revision. Gate de avance se recalcula bajo la misma transacción que avanza. Editar fase anterior conserva lo posterior y lo marca NEEDS_REVIEW si cambió una salida que afecta el gate. |
| KnowledgeItem | KnowledgeItem | UUID global. Subtipo/extensión coherente con `type_code`. Captura multi tabla e hipervínculos se confirman juntos. Archivar un item compartido no lo borra ni destruye procedencia. |
| Concept | KnowledgeItem de tipo `concept` | Nombre preferido y alias normalizados para sugerir búsqueda, no para probar equivalencia. Reutilización siempre seleccionada por la persona; cambiar nombre preserva UUID. |
| Relation | Relation | Extremos existentes, tipos compatibles, sin autoenlace; unicidad semántica conserva contextos distintos. Procedencia de la relación se guarda junto con ella. |
| Provenance | Provenance | Pertenece a un Document y guarda el hash visto al capturar. Cita literal y texto interpretativo permanecen separados. Un hash distinto hace locator `STALE`, no lo reescribe. |

Las operaciones que atraviesan SQLite y filesystem usan una saga local recuperable, no una falsa transacción atómica: intención durable, staging en el mismo volumen, hash verificado, promoción, commit, y reconciliación idempotente al arrancar. Un archivo ambiguo no se borra automáticamente.

## Enumeraciones canónicas

Se persisten los siguientes valores exactos; presentación y traducción no modifican las claves:

```text
PaperLifecycle = NEW | ACTIVE | COMPLETED | ARCHIVED | TRASHED
PhaseCode = PRE | P1 | P2 | P3 | P4
PhaseState = NOT_STARTED | IN_PROGRESS | COMPLETED | NEEDS_REVIEW
AnswerResolution = PENDING | ANSWERED | UNKNOWN | NOT_APPLICABLE
QuestionState = OPEN | ANSWERED | DEFERRED | DISMISSED
Origin = literature | researcher_interpretation | researcher_hypothesis
Confidence = sufficiently_supported | context_dependent | uncertain | requires_validation
LocatorState = PENDING | LOCATED | STALE
ReviewType = survey | topical_review | slr | mapping_study | tutorial | other | unknown
Relevance = sufficient | use_with_caution | weak_for_my_purpose
ReadingDecision = continue | light_read | archive
```

**Estado editorial:** `COMPLETED` solo se asigna cuando el workflow completo hasta P4 esté implementado y satisfecho. En v0.1, completar P2 no cambia el paper a COMPLETED; la UI expresa “P2 completada” o “revisión inicial completa”. No representa certeza científica. Si el paper se archiva desde NEW/ACTIVE/COMPLETED, guardar `archived_from_lifecycle`; restore vuelve a ese estado. `TRASHED` es estado reservado para el futuro; no exponerlo en el piloto ni v0.1.

## Fases operativas y gates

Las definiciones incluyen `code`, `version`, `objective`, `key_questions`, `do_items`, `dont_items`, `considerations`, `required_outputs` y `completion_rules`. Los payloads canónicos v1 están en [phase-definitions/](phase-definitions/); cada Paper fija PRE/P1/P2 al inicializar processing. Son inmutables y pueden embeberse al compilar. [WORKFLOW_GATES.md](WORKFLOW_GATES.md), ADR-017, fija resoluciones, reglas por salida, habilitación, proyecciones, snapshots y revisiones sin alterar el wire.

Regla común: salida requerida procesada según allowedResolutions. ANSWERED textual exige contenido; UNKNOWN/NOT_APPLICABLE exige explicación no vacía sin duplicarla en answerText. PRE.review_type, P1.relevance_decision y P2.p3_candidates_or_justification se procesan por estructura válida sin prosa redundante; PENDING/null puede guardar una respuesta pendiente y nunca cierra gate. No hay puntuación epistemológica ni cuotas. Formas exactas, ausencia justificada y alternativa cerrada P2 por key en WORKFLOW_GATES.

**Invalidación determinista:** un cambio real en `answerText`, `structuredValue`, `resolution` o `explanation` de una fase `COMPLETED` la cambia a `NEEDS_REVIEW`, aunque siga pasando el gate; requiere `advancePhase` para volver a completarla. Fases posteriores instanciadas que no estén `NOT_STARTED` también pasan a `NEEDS_REVIEW`, preservando datos. Captura/edición de item, asociación de concepto, cambio de relación, procedencia o candidatura P3 que modifique el snapshot de P2 completada cambia P2 a `NEEDS_REVIEW`. Un no-op con mismo valor canónico no invalida. El snapshot aceptado al completar contiene definición versionada, respuestas normalizadas y proyección estable de salidas/entidades requeridas; el hash usa serialización canónica ordenada.

### PRE — contexto previo

Salidas requeridas PRE visibles: `purpose`, `uncertainty_target`, `baseline`, `expected_outcome`, `desired_depth`, `review_type`. Esta sexta respuesta confirma el metadato Paper.reviewType con `{reviewType: ReviewType}`: tipos conocidos ANSWERED; unknown UNKNOWN+explanation. El gate compara captura y metadato actual; import default no equivale a procesado ni la respuesta edita bibliografía. También requiere título no vacío y Document vigente ACTIVE disponible. Las cinco preguntas previas conservan su significado; no inferir conclusiones del título.

Sólo advance PRE exitoso acepta su snapshot y cambia NEW→ACTIVE en la misma UoW. Habilitar P1 hacia delante requiere PRE COMPLETED, snapshot aceptado y gate vigente satisfecho; habilitar P2 requiere cadena aceptada vigente y P1 readingDecision=continue. Touch no acepta prerequisitos. Consulta de fases ya iniciadas conserva datos sin aceptar/completar implícitamente. Todas las fases observan disponibilidad documental viva según WORKFLOW_GATES.

### P1 — orientación

Salidas requeridas P1: `scope`, `out_of_scope`, `review_type`, `literature_cutoff`, `core_message`, `relevance_decision`; `field_organization` opcional. P1.review_type caracteriza textualmente el enfoque/metodología según la fuente; no es segundo editor del metadato confirmado en PRE. Literatura/metodología no declaradas se registran UNKNOWN explicado donde lo permite la definición, sin inventarlas.

`relevance_decision` combina valoración (`sufficient`, `use_with_caution`, `weak_for_my_purpose`) y decisión (`continue`, `light_read`, `archive`). ANSWERED requiere ambos enums; UNKNOWN/NA no inventa una rama. Sólo continue aceptado habilita P2. Light_read completa P1, active P1, Paper ACTIVE, nextPhase=null. Archive sólo en advance completa P1 y archiva Paper atómicamente, conserva lifecycle anterior/respuestas y active P1, nextPhase=null. P2 nunca iniciada queda NOT_STARTED; P2 previa conserva datos/NEEDS_REVIEW. Ninguna decisión completa todo el workflow.

### P2 — modelo conceptual

Salidas P2 canónicas: `main_questions_processed`, `field_synthesis`, `relevant_concepts_reviewed`, `meaningful_relations_reviewed`, `contradictions_reviewed`, `references_classified`, `p3_candidates_or_justification`. Preguntas se relacionan con PRE. Cada salida enlaza entidades/relaciones consultables; síntesis se conserva como Insight. Arrays vacíos requieren justificación explícita de ausencia, sin mínimos arbitrarios. UNKNOWN/NA sólo donde la definición lo permite. Mapping por key y proyección persistida exactos en WORKFLOW_GATES; un checkbox narrativo no sustituye artefactos.

Completar P2 procesa el mapa manual y cola P3 sin validar afirmaciones. Asociaciones paper_items gobiernan selección/priority/rationale de Claims/Questions. La única justificación de cero candidatos reside en PhaseAnswer(P2,p3_candidates_or_justification).structuredValue.noCandidatesJustification; candidates/count es proyección comprobada. setP3Candidate y savePhaseAnswer de ausencia operan sobre esa fuente en la misma UoW; un cambio de proyección renueva answer.revision y clock P2. P3 workspace puede estar deshabilitado. Gate P2 real/aceptación final espera todas las capacidades T06/T07; ausencia de resolutor es UnsupportedCapability.

P3 y P4 tienen código reservado, pero no gate ejecutable en v0.1. No marcar Paper COMPLETED por alcanzar P2 si la definición de producto aún exige otras fases operativas; en el alcance v0.1 la UI debe nombrar “P2 completada / revisión inicial completa”.

## KnowledgeItem, origen, confianza y procedencia

Tipos core del catálogo: `concept`, `claim`, `evidence`, `question`, `gap`, `assumption`, `condition`, `limitation`, `method`, `example`, `insight`, `reference`. El catálogo admite los tipos, pero los formularios pueden desplegarse gradualmente. Dataset/Metric/Task/System y domain packs quedan posteriores.

- `literature`: el contenido describe lo que la fuente expresa. Para presentarlo como trazable, exigir Provenance `LOCATED`; capturas en borrador pueden persistir sin fuente (`NONE`, pendiente de atribuir) o con fuente sin anclaje (`PENDING`), visibles como pendientes.
- `researcher_interpretation`: interpretación del investigador; no presentarla como cita textual. Puede enlazar a la fuente que la motivó.
- `researcher_hypothesis`: hipótesis propia, explícita y atribuida al investigador; puede carecer de cita, pero el contexto/razón se registra.
- Quote/snippet es literal y separado de body/interpretation. `page_index` es página física desde 1; `page_label` puede diferir. Hash capturado igual al documento actual puede marcar LOCATED; cambiar archivo/hash marca STALE.
- Confidence inicial es `requires_validation`. Solo una decisión humana explícita puede cambiarla. La app no la calcula contando citas, relaciones, coincidencias de texto o estados de fase.
- Evidencia describe fragmento o resultado inspeccionado; su estado no se confunde con confianza de Claim. DOI y cita secundaria por sí solos no son evidencia evaluada.

## Relaciones permitidas en v0.1

La relación siempre guarda `source`, `target`, `type`, `context`, `origin`, `confidence`, fechas y procedencia separada. Propuesta de allowlist inicial para formularios/validación:

| type | source → target permitido | Simétrica |
|---|---|---|
| supports | evidence → claim | no |
| contradicts | claim ↔ claim | sí, normalizar extremos |
| extends | method → method; claim → claim | no |
| causes | claim → claim o concept | no |
| requires | method → condition | no |
| depends_on | method → concept o method | no |
| works_when | method → condition | no |
| fails_when | method → condition | no |
| compares_with | method ↔ method | sí |
| part_of | concept → concept | no |
| similar_to | concept ↔ concept | sí |
| limits | limitation → claim o method | no |
| improves | method → method | no |

La allowlist v0.1 habilita exactamente estos 13 tipos y combinaciones; el catálogo persistido y `RelationType` usan la misma versión. `causes`/`improves` requieren contexto y origen explícitos; elegirlos no demuestra causalidad o mejora. Toda relación admite contexto; contextos diferentes no se deduplican. No aceptar extremos fuera de la matriz. Relaciones dirigidas conservan orden; las simétricas lo normalizan.

## Archive, restore, delete y merge

- **Paper archive/restore:** reversible; preserva documentos, metadatos, phases, KnowledgeItems y procedencia. Archivar no archiva contenido global ni borra dependencias compartidas.
- **Item/Concept archive/restore:** reversible; se excluye de listas activas por defecto, pero enlaces históricos continúan visibles y puede restaurarse.
- **Delete Paper/KnowledgeItem/Concept:** piloto 0.0.1 y v0.1: no hard delete ni estado TRASHED expuesto. Paper y KnowledgeItem/Concept usan archive/restore reversible; relaciones y procedencia quedan conservadas. `TRASHED` queda reservado a una posible papelera posterior.
- **Merge Concept:** fuera de piloto 0.0.1 y v0.1; requiere ADR de alcance posterior a v0.1. Si se aprueba más adelante: operación explícita con preview de enlaces/alias/relaciones afectados, destino elegido y confirmación. La transacción reasigna referencias seguras, deduplica solo relaciones equivalentes conservando contexto, marca origen como redirigido al destino y registra auditoría. UUID de origen permanece; rechazar ciclos, autorrelaciones o colisión ambigua. Si no se puede preservar semántica/procedencia, rechazar.

## Fuera del dominio v0.1

Usuarios/roles, sincronización, API remota, microservicios, OCR, análisis automático, LLM/agentes, RAG/embeddings, P3/P4 operativos, score de confianza automático, editor genérico de ontología y grafo visual complejo. Las reglas futuras solo informan sobre carencias del registro; no convierten falta de cobertura personal en gap de toda la literatura.

COMPLETED reservado se conserva en upgrade/get/lectura y Library archive/restore, también ARCHIVED desde COMPLETED. Transiciones Workflow incompatibles devuelven UnsupportedCapability sin efectos; no conversión silenciosa ni migración especial. Upgrade no cambia metadatos/lifecycle ni restaura archivados; borradores PRE NEW siguen editables y sólo advance PRE los activa editorialmente.



Precisión de edición ADR-020: guardar no inicia una fase; NOT_STARTED rechaza escrituras de respuesta, mientras una fase iniciada puede editarse sin mover el contexto activo. Replay, CAS, clocks e invalidación se rigen por WORKFLOW_GATES.

## Precisiones de captura compartida (ADR-022)

Concept conserva nombre/definición canónicos y UUID; Knowledge sólo modifica su confidence, ConceptApi controla el resto. Insight.affectedConceptIds clasifica un subset explícito de conceptos vinculados: eliminar esa clasificación no elimina vínculo ni linkConcept la añade. No se fusionan homónimos. Edición/enlace desde padre archivado exige restore; nuevas asociaciones hacia Concept archivado se rechazan, asociaciones históricas resuelven. Repetir asociación existente desde padre activo es no-op aunque destino se haya archivado.

La proyección P2 parte de paper_items y sigue item_concepts transitivamente, con deduplicación por UUID hasta terminar ciclos. Sólo incluye relaciones cuyos dos extremos ya están alcanzados; relaciones no expanden alcance. Archivados permanecen en historia/proyección. La invalidación inversa usa el mismo cierre antes/después. Una edición de artefacto renueva clock P2 cuando corresponde; PhaseAnswer candidata sólo cambia si cambia selección/count/justificación, no por editar body sin cambiar esa proyección.

Procedencia sin source es NONE, no una fila ficticia. LOCATED es ubicación registrada conforme CONTRACTS, no cita verificada ni prueba de página física; T07 valida navegación. Editar locator conserva Document/hash y no elimina STALE; cambiar fuente requiere attach nuevo explícito. Firmas, restricciones completas y criterios en CONTRACTS y TASK05_PORTS.
