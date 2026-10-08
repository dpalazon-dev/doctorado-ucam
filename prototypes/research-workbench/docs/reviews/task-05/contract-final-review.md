# T05 — revisión documental final, corrección 1

2026-10-03. **PASS documental para R1–R5.** Los cinco hallazgos importantes quedan addressed; ninguno permanece abierto. Se detectó una regresión de enlaces de prioridad baja y dos detalles editoriales ya señalados por Sol, corregibles sin cambiar reglas. Este PASS no demuestra implementación ni activa T05.

## Corte y alcance

Revisión sobre `.superpowers/sdd/IMPLEMENTATION/task-05-documentary-fix1.diff`, SHA256 `E483E040794FA67F60C0DDB746393D26A6AC80BF81E48D9AE6B02C83BD413A4F`. El archivo contiene el diff de normas/briefs y texto completo de TASK05_PORTS. Repo principal congelado durante la revisión; cambios sin commit. Se contrastaron los documentos locales afectados para numeración, referencias y paridad de anexos. No se revisó producto WIP T04b.

La comprobación runtime adicional fue exclusivamente `git show 4162ac1355d214c7d5d47c4aa96695c526b2e3d3:src-tauri/src/transport/dto.rs` para confirmar el nombre existente KnowledgeItemDtoLifecycle. No se ejecutaron SQL, builds, tests de producto, instalaciones, commits, merges o subagentes. Única escritura del reviewer: este informe central.

## R1–R5: addressed / remaining

| Hallazgo | Resultado | Evidencia de corrección | Remaining |
|---|---|---|---|
| R1 requestId lifecycle | addressed | TASK05_PORTS:181–185 recibe request_id explícito; caso único para Item/Concept; auditoría efectiva lo consume en 207–208,297 | ninguno documental |
| R2 changes/auditoría | addressed | Puerto changes: &serde_json::Value en TASK05_PORTS:207–208; acciones cerradas y before/after/create/link en 297; DATA:166 conserva evento efectivo distinto del receipt genérico | ninguno documental |
| R3 casos TX edición/enlace | addressed | update_item_in_tx, update_concept_in_tx y link_concept_in_tx en TASK05_PORTS:278–292; repos reciben datos decididos, aplicación mantiene política, wrapper receipt/TX único en 275,295; ownership añadido en brief T05:19 | ninguno documental |
| R4 cierre P2/inversa | addressed | TASK05_PORTS:219; WORKFLOW_GATES:107–111; DOMAIN:148. Cierre transitivo de item_concepts con ciclos, Relations no expanden, inversa exacta antes/después y sin excluir archivados/NOT_STARTED | ninguno documental |
| R5 colisión contexto Relation | addressed | CONTRACTS:348; TASK05_PORTS:117,299; brief T07:20. Create/update-context/restore colisionando = Conflict con rollback completo; traducción de constraint conocida | ninguno documental |

Las firmas mínimas requeridas están presentes, usan Args/DTO existentes y una Transaction prestada. No nuevo wire/CAS para linkConcept ni actor/receipt/framework adicional. Link lee la revisión actual bajo IMMEDIATE; adición real incrementa el item una vez, no el destino. Las siete acciones de modificación son cerradas; lifecycle Concept usa acciones genéricas con identidad Item, evitando un segundo recorrido. No-op y replay no crean eventos efectivos ni renuevan clocks/revisiones/fechas/snapshots.

## Consistencia de la corrección

- La precisión de Sol sobre archivados coincide en CONTRACTS:344, TASK05_PORTS:14–15, DOMAIN:146 y anexos: padre ACTIVE + asociación ya existente + destino ARCHIVED = no-op; adición nueva a destino ARCHIVED = InvalidInput; padre ARCHIVED = InvalidInput incluso si no cambiaría nada. Replay durable precede esas guardas.
- La declaración conserva seis comandos Knowledge y ocho Concept. KnowledgeItemDtoLifecycle existe en el corte estable; Concept conserva revisión única en Knowledge, nombre/title mirror y definition/body única. Las nuevas funciones TX no cambian sus métodos wire.
- Anexos T05/T06/T07: comparadas las secciones Sobre y error, DTOs compartidos, Knowledge/Concepts/Relations y Provenance con CONTRACTS, las doce comparaciones dieron igualdad de todas las líneas no vacías. Workflow T06 también dio igualdad de contenido; esta copia introdujo la ruta relativa errónea indicada abajo.
- Normalización conserva límites y separa canon de contenido del hash de receipts. Sets de IDs no extienden deduplicación a alias rechazables ni implican fusionar entidades. El test excesivo de candidatura se reemplazó en T05:11; edición de artefacto no reescribe una proyección candidata idéntica.
- Mapping consumidor conserva atributos reconstruidos, canon Concept y edges paperItem/itemConcept/itemProvenance/relationProvenance/conceptAlias. Provenance locator_json se proyecta a locator. No apareció un nuevo campo exportable, authority paralela ni validación ficticia; validations sigue vacía. DDL/constraints concretos se revisarán antes de congelar0003.
- ADR-022 se declara decisión antes de implementación; normas/briefs diferencian ese estado de capacidades realizadas. La secuencia T05a/T05b conserva BASE pendiente de T04b/T04c, revisión independiente y límites consumidores T06/T07/T08. No se interpretó una prueba de schema como servicio Relation/Provenance o aceptación P2.

## Observaciones menores

**E1 — regresión de rutas relativas en T06 (baja, nueva).** En `docs/plans/tasks/task-06-brief.md:129,181`, el anexo copió `[WORKFLOW_GATES.md](WORKFLOW_GATES.md)` y en 129 `[PRE/P1/P2 v1](phase-definitions/)` desde CONTRACTS. Desde el directorio del brief apuntan a destinos inexistentes. Se verificó Test-Path: ambos destinos locales false; los destinos reales bajo docs/architecture true. Sol debe restaurar las rutas `../../architecture/WORKFLOW_GATES.md` (dos enlaces) y `../../architecture/phase-definitions/` (uno). La paridad semántica/literal del texto no exige duplicar rutas relativas incorrectas.

**E2/E3 — detalles editoriales ya identificados por Sol (bajos).** TASK05_PORTS:17 necesita línea en blanco antes del heading; en 117 sigue «Propuesta de índice parcial» aunque la misma frase declara decisión aceptada por ADR-022. Cambiar a «Índice parcial activo» y conservar el resto. Ninguno cambia el ABI o el comportamiento aprobado.

No hay otro hallazgo crítico/importante introducido por las correcciones en el alcance asignado. Las observaciones menores no reabren R1–R5 ni requieren ampliar producto/API.

## Verificación y límites

Lecturas con Get-Content y rg acotados al diff/documentos afectados. Verificación documental automatizada: extracción por headings y comparación ordinal de líneas no vacías (12/12 secciones Knowledge/Concept/Provenance compartidas + Workflow T06 iguales); Test-Path para los enlaces; Get-FileHash del diff; confirmación del enum DTO mediante git show estable. Se comprobó el informe guardado y whitespace final. No se atribuye a estas comprobaciones una compilación, migración, atomicidad ejecutada, navegación PDF, instalación o funcionamiento nativo.

**Autorrevisión:** se cerraron sólo R1–R5 y la consistencia de su corrección. Se conservaron normas/producto/AGENTS.md y los cambios ajenos. El PASS autoriza documentalmente seguir con la preparación/implementación según el orquestador y dependencias; publicación/commit y correcciones editoriales los ejecuta Sol, no este reviewer.
