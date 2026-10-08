# Research Workbench — requisitos funcionales

Estado: especificación de la baseline de ejecución v0.2, adoptada el 1 de octubre de 2026. Define comportamiento esperado; no certifica código, instalador ni funciones implementadas. La arquitectura y firmas normativas de esta propuesta pertenecen a `ARCHITECTURE.md`, `DOMAIN.md` y `CONTRACTS.md`. Ningún control de UI puede aparentar éxito: opera una función disponible o se presenta inequívocamente como futura.

## Alcance por versión

- **Piloto 0.0.1:** producto Windows 11 x64 instalado; biblioteca local, importación/copia PDF, metadatos, lector PDF, posición, continuidad, edición y archivo/restauración de papers; migración protegida y verificada.
- **v0.1.0:** PRE/P1/P2 completos, conocimiento global trazable, búsqueda de metadatos/conocimiento, export JSONL/Markdown, backup restaurable y cambio de biblioteca. Añade archive/restore de papers, conceptos e items. No incluye P3/P4, LLM, fusión, papelera expuesta ni borrado permanente.
- Los escenarios Gherkin son criterios requeridos, no evidencia de implementación. Fases completadas son progreso editorial, no validación científica.

## SPEC-001 — Desktop e instalación del piloto

**REQ-001-01** Distribuir `setup.exe` NSIS x64 por usuario, con entrada en Aplicaciones instaladas, acceso Inicio, escritorio opcional, ventana propia y desinstalador. **REQ-001-02** Empaquetar UI y PDF.js/worker, incluir WebView2 offline y funcionar sin Node, Rust, Git, Codex, cwd específico, dev server o red. **REQ-001-03** Mantener biblioteca separada de binarios/código. Primer inicio muestra y confirma `%LOCALAPPDATA%\ResearchWorkbench\library`, explica la copia local de PDFs. **REQ-001-04** Una sola instancia escritora por biblioteca, cierre controlado. Actualización crea copia consistente antes de migrar; esquema futuro/downgrade incompatible se rechaza sin escritura. Desinstalar conserva datos y backups.

```gherkin
Given Windows 11 x64 sin WebView2, red ni herramientas de desarrollo
When instalo el setup.exe offline y abro desde Inicio
Then abre la ventana propia sin servidor de desarrollo y el instalador proporciona WebView2

Given una biblioteca con cambios confirmados
When actualizo de forma compatible, desinstalo y reinstalo
Then migración y reinstalación conservan UUID, documentos y posición; desinstalar retira binarios pero mantiene datos
```

**Errores:** runtime/instalación sin espacio o permisos, biblioteca ocupada, esquema incompatible, fallo de backup/migración, cierre con escritura pendiente. Explicar recuperación y no afirmar guardado/actualización sin commit. **Salidas:** instalador real/checksum, guía breve y evidencia de equipo limpio/offline; separar evidencia de build, mocks y QA instalado.

## SPEC-002 — Biblioteca, importación y lector

**REQ-002-01** Selector nativo; copiar PDF a biblioteca, UUID/hash estables, independiente del original. **REQ-002-02** Metadatos manuales título, autores opcionales, año nullable, venue, DOI, tipo de revisión/dominio; validar sin inventar desconocidos. **REQ-002-03** Detectar duplicado por DOI normalizado/hash; ofrecer abrir existente/cancelar, sin fusionar o sobrescribir silenciosamente. **REQ-002-04** Lista, búsqueda de metadatos, filtros básicos, detalle, edición, archivo/restauración, vista vacía y archivados. **REQ-002-05** Lector interno conserva página física desde uno y zoom; reanuda paper/contexto/posición confirmados. **REQ-002-06** “Guardado” solo tras respuesta persistida; error mantiene cambios pendientes visibles.

```gherkin
Given un PDF legible y metadatos revisados
When confirmo, muevo el original, cierro la aplicación y reabro el paper
Then abre la copia administrada con el mismo UUID y posición confirmada

Given un candidato con DOI normalizado o hash coincidente
When importo el archivo
Then puedo abrir el existente o cancelar y no se crea ni fusiona sin decisión
```

**Errores:** cancelación de selector, PDF inválido/protegido, disco lleno, DOI inválido, conflicto de revisión, documento desaparecido, render/guardado fallido. Errores recuperables, sin acceso arbitrario a paths. No prometer OCR/texto PDF buscable. **Salidas:** paper/documento registrados, recuperación de importación y estados de carga/vacío/error/pendiente/éxito en español.

## SPEC-003 — PRE/P1 versionadas y gates

**REQ-003-01** Fijar PRE/P1/P2 version=1 canónica por Paper; nuevas plantillas no reescriben historia. **REQ-003-02** Respuesta pendiente/respondida/desconocida/no aplicable según matrices JSON; UNKNOWN/NA explicado sin prosa redundante, decisión estructurada P1 ANSWERED y draft PENDING/null. **REQ-003-03 PRE** seis salidas visibles: `purpose`, `uncertainty_target`, `baseline`, `expected_outcome`, `desired_depth`, `review_type`; la sexta confirma el tipo canónico, no edita bibliografía; además título y PDF vigente disponible. Unknown default no equivale a confirmación explicada. **REQ-003-04 P1** `scope`, `out_of_scope`, `review_type`, `literature_cutoff`, `core_message`, `relevance_decision`; review_type textual caracteriza fuente/metodología; continue aceptado habilita P2. **REQ-003-05** Reevalúa bajo advance sin confiar en UI; forward exige prerequisito COMPLETED+snapshot aceptado+gate vigente, touch no acepta implícitamente. **REQ-003-06** Edición efectiva preserva historial y exige reconfirmación de fase COMPLETED/posteriores iniciadas aunque su gate de contenido aún pase. Detalle cerrado en [WORKFLOW_GATES.md](WORKFLOW_GATES.md), ADR-017.

```gherkin
Given PRE aceptada y vigente y un campo textual P1 desconocido con motivo
When confirmo P1 con advancePhase y decisión continue
Then cuenta como procesado y P2 queda habilitada

Given PRE tiene una salida pendiente y la UI intenta avanzar con estado obsoleto
When el servicio procesa advancePhase
Then rechaza la transición, devuelve pendientes y conserva fase activa

Given P2 guardada y una edición P1 invalida su requisito
When se confirma la edición
Then P2 conserva sus datos pero queda marcada para revisión
```

La invalidación conserva datos, accepted snapshot y completedAt históricos; state NEEDS_REVIEW exige reconfirmación. NOT_STARTED sin iniciar y no-op conserva revisiones. Guardar decisión P1 no archiva. Continue acepta P1/activa P2; light_read completa P1 y conserva active P1/Paper ACTIVE; archive completa P1/archiva atómicamente con active P1. Ambas terminales nextPhase=null; P2 anterior conserva NEEDS_REVIEW, no se resetea. NEW→ACTIVE sólo primer advance PRE, no import/upgrade/lector/guardar borrador. goBack/touch preserva contenido/completitud y renueva token destino sólo al cambiar contexto. PDF vivo en inputs de todas las fases, proof/handle fuera TX y referencia DB revalidada dentro, sin hash masivo en DbActor. Casos exactos R1/R2 en WORKFLOW_GATES.

**Errores:** definición/version no disponible, gate incompleto, valor inválido, revisión obsoleta o fallo de persistencia. Diferenciar estados en UI y conservar borrador ante error. **Salidas:** definición versionada, respuestas, gate reproducible, fase activa y flags de revisión; no inferir verdad científica.

Escenarios obligatorios adicionales: touch(P1) bloqueado aunque PRE IN_PROGRESS tenga gate suficiente; touch(P2)/advance(P2) bloqueados con P1 sin aceptar/NEEDS_REVIEW; consulta de fases previas iniciadas preservada; PDF inaccesible tras aceptar PRE/P1 bloquea evaluate/advance posteriores sin mutación ni receipt de éxito, available cambia hash actual y snapshot aceptado se conserva. Recuperar acceso permite reevaluar; cambio DB Document renueva CAS/invalida dentro de UoW. COMPLETED reservado y ARCHIVED desde COMPLETED se conservan en upgrade/lectura/Library archive/restore; transiciones Workflow incompatibles UnsupportedCapability sin efectos.

## SPEC-004 — P2, conceptos globales, procedencia y relaciones

**REQ-004-01** Formularios habilitados para Concept, Claim, Evidence, Question, Gap, Assumption, Condition, Limitation, Method, Example, Insight y Reference según contrato; futuros tipos no aparecen como controles activos. `item.body` v0.1 es `plain_text`, sin editor rich text. **REQ-004-02** Concepto UUID global con preferred name, definición, alias/dominio; permitir reutilizar existente o crear otro sentido, sin unión automática. Vista por paper y concepto consulta entidades compartidas. **REQ-004-03** Todo item especifica origin (`literature`, `researcher_interpretation`, `researcher_hypothesis`); captura literaria requiere fuente y localizador o muestra pendiente. Cita y lectura interpretativa son campos separados. **REQ-004-04** Provenance admite documento, página física, sección, cita/snippet, hash y estado stale/pending/located. **REQ-004-05** Relation tipada valida extremos y semántica, conserva contexto/procedencia; conectar P2 con preguntas PRE. **REQ-004-06** Gate P2 usa la lista versionada de outputs requeridos: `main_questions_processed`, `field_synthesis`, `relevant_concepts_reviewed`, `meaningful_relations_reviewed`, `contradictions_reviewed`, `references_classified`, `p3_candidates_or_justification`. Matrices de resolución y p2ArtifactPresent por key en WORKFLOW_GATES: respuestas enlazan entidades/relaciones persistidas, síntesis como Insight, ausencia de artefactos explicada sin cuotas; candidata usa alternativa cerrada artefactos OR ausencia justificada. Justificación cero candidatos única en PhaseAnswer estructurada, selección en paper_items y proyección comprobada; cambio real de proyección incrementa answer.revision más clock P2 en misma UoW. **REQ-004-07** Ediciones preservan historial y marcan progreso posterior para revisión si invalidan salidas obligatorias; nunca borran los datos. P2 no implica P3/P4.

```gherkin
Given dos papers y un concepto de sentido compartido
When enlazo una afirmación del segundo paper al concepto
Then se reutiliza el UUID global y la afirmación conserva fuente, localizador y origen propios

Given una captura literaria sin localizador resuelto
When la guardo como borrador
Then figura como procedencia pendiente, no como trazable/validada

Given un gate P2 sin candidatos P3 pertinentes
When guardo la decisión razonada de ausencia
Then el requisito se procesa sin inventar candidatos ni un conteo mínimo
```

**Errores:** localizador inválido/stale, documento cambiado, extremo ausente, relación no permitida, conflicto y rollback. No destruir conocimiento compartido ni presentar incompatibilidades como consenso. **Salidas:** fichas por paper/concepto, procedencia, relaciones, cola P3 persistible y gate; ningún claim de validación epistemológica.

Completar P2 requiere cadena aceptada/vigente, PDF vivo y resolutores reales T06/T07; capacidad requerida ausente devuelve UnsupportedCapability, aceptación final espera T07. Confirma su gate y devuelve nextPhase=null en v0.1; no se llama a un comando P3 que impediría cerrar P2. Paper permanece ACTIVE y muestra P2 completada; COMPLETED global queda reservado al workflow completo posterior. La cola P3 usa la asociación paper/item con selección, prioridad y motivo, consultable aunque el workspace P3 esté deshabilitado. Editar efectivamente artefactos usados por P2 invalida su completitud sin perderlos.

## SPEC-005 — Búsqueda FTS5

**REQ-005-01** Indexar metadatos de paper y título/cuerpo textual de KnowledgeItem; no indexar contenido PDF/OCR. **REQ-005-02** Filtros por tipo, concepto, paper, dominio, confianza y estado según contrato; archivados fuera por defecto y consultables explícitamente. TRASHED no se expone en v0.1. **REQ-005-03** Resultados identifican tipo, contexto y origen y abren objeto/localizador. **REQ-005-04** Índice derivado reconstruible y coherente con cambios confirmados, no autoridad canónica.

```gherkin
Given paper y conocimiento capturado en índice válido
When busco texto con filtro paper/tipo
Then recibo solo coincidencias aplicables y cada resultado navega al objeto correcto

Given una interrupción durante actualización de FTS
When reconstruyo desde SQLite
Then los resultados coinciden con la fuente y se excluyen archivados por defecto
```

**Errores:** índice corrupto o rebuild fallido se distingue de cero resultados y ofrece reintento; query válida sin matches da vacío. **Salidas:** resultados filtrables, estado de índice. No se ofrece búsqueda dentro del PDF.

## SPEC-006 — Export JSONL/Markdown

**REQ-006-01** Export versionado incluye manifest (schema/ontology versions, timestamp UTC, counts, hashes), UUIDs, entidades, relaciones, fases/respuestas, asociaciones y procedencia. **REQ-006-02** JSONL una entidad por línea; Markdown vistas por paper/concepto. **REQ-006-03** Opción explícita de incluir PDFs; export compartible excluye paths privados/logs por defecto. **REQ-006-04** Destino validado contra traversal y sobrescritura; export no muta SQLite; editar Markdown no cambia canon.

```gherkin
Given dos papers comparten concepto y hay relaciones/procedencias
When exporto sin documentos binarios
Then referencias UUID resuelven, manifest contiene conteos/versiones/hashes y rutas locales se omiten

Given título malicioso o destino no escribible
When exporto
Then la operación rechaza o pide destino y nunca escribe fuera ni altera SQLite
```

**Errores:** cancelar destino, espacio, serialización/hash inconsistente o conflicto. Confirmar éxito solo tras cerrar y verificar el paquete. **Salidas:** paquete documentado y resultado; Markdown round-trip no incluido.

## SPEC-007 — Backup, restore y cambio de biblioteca

**REQ-007-01** Snapshot SQLite consistente incluyendo WAL, documentos referenciados, ontología y manifest con hashes/versiones; política v0.1: backup manual y antes de migración. No ofrecer programación automática/retención configurable sin contrato. Nunca retirar última copia verificada antes de verificar otra. **REQ-007-02** Verificar integridad, FKs y hashes; escoger una copia externa con selector nativo y poder elegir otro disco como destino. **REQ-007-03** Restaurar en ubicación nueva; abrir/verificar antes del switch y mantener activa anterior recuperable hasta confirmar. **REQ-007-04** Cambiar a biblioteca existente o preparada con escritor detenido, rutas relativas, comprobación completa y operación recuperable. Trasladar automáticamente una biblioteca existente requiere contrato posterior; en v0.1 puede obtenerse en otro destino mediante backup/restore. **REQ-007-05** UI muestra ubicación activa/resultado; no hay controles ficticios.

```gherkin
Given backup verificado y biblioteca activa distinta
When restauro en una carpeta nueva
Then valido DB/FKs/hashes, abro la restaurada y solo tras confirmación la activo; la anterior sigue recuperable

Given la biblioteca tiene escritor activo o el destino se queda sin espacio
When cambio su ubicación
Then se espera el cierre o falla con explicación y ninguna ruta parcial pasa a ser activa
```

**Errores:** WAL/inconsistencia, recurso ausente, hash incorrecto, versión/permisos/espacio, destino ocupado, cancelación/interrupción. Informar cuál biblioteca sigue activa. **Salidas:** backup verificado restaurable, informe de operación y ubicación activa; crear archivo no basta para llamar backup satisfactorio.

## SPEC-008 — Mantenimiento reversible y progreso

**REQ-008-01** Lifecycle editorial separado de workflow. Archive/restore de paper conserva UUID, documentos, metadatos, respuestas y conocimiento. **REQ-008-02** Archive/restore de Concept y KnowledgeItem conserva relaciones y procedencia y oculta de vistas activas; restaurar no cambia UUID. **REQ-008-03** Renombrar concepto preserva identidad/enlaces. **REQ-008-04** Editar fase previa conserva historial/datos posteriores; progreso se marca para revisión si dejó de cumplir requisitos. **REQ-008-05** Retirar paper muestra dependencias y nunca cascada silenciosa sobre concepto/item global. `TRASHED` puede existir como valor reservado/legado del modelo, pero en piloto y v0.1 no se expone, no hay operación de trash/purge y no se debe inferir borrado físico. La papelera reversible requiere ADR/contrato posterior antes de habilitarla.

La especificación fuente describe fusión revisada y borrado reversible; el alcance acordado con DOMAIN/CONTRACTS deja **merge, TRASHED expuesto y borrado permanente fuera de piloto/v0.1**. No crear controles para ellos. `TRASHED` reservado en un enum no implica que la operación esté implementada; cualquier papelera posterior necesita ADR, dependencias, recuperación y contrato explícitos.

```gherkin
Given paper archivado con posición y KnowledgeItems enlazados
When lo restauro
Then conserva UUID/documento/datos y vuelve a su lifecycle anterior

Given concepto archivado y relaciones existentes
When lo restauro
Then el mismo UUID y todas sus relaciones/procedencias vuelven a vistas activas

Given una edición P1 invalida una salida obligatoria posterior
When guardo la edición
Then historial y datos posteriores permanecen y el progreso se señala para revisión
```

**Errores:** revisión obsoleta, recurso dependiente, referencia global, rollback fallido o restauración no disponible. Rechazo atómico con dependencias; nunca borrar datos compartidos. **Salidas:** lifecycle/estado de fase diferenciados, impacto visible e historial; solo acciones implementadas.

## Trazabilidad de la propuesta

Los ADR y contratos son referencias normativas de esta propuesta. Las pruebas listadas son previstas, no ejecutadas.

| SPEC | ADR a reconciliar | Contratos/operaciones semánticas | Pruebas planificadas |
|---|---|---|---|
| 001 | ADR-001 local; ADR-002 Tauri; ADR-011 NSIS/offline; ADR-012 single writer/recovery | `getAppInfo`, `getLibraryInfo`, `DesktopLifecycle`, `MigrationGuard` | `desktop_clean_offline_install`, `upgrade_preserves_library`, `uninstall_retains_data`, `single_writer`, `migration_failure_recovery` |
| 002 | ADR-002; ADR-006 documentos | `LibraryService`, `openPaper`, `ReadingPosition`, `ReaderProtocol` | `import_survives_original_move`, `duplicate_decision`, `reader_resume_after_restart`, `protocol_rejects_traversal` |
| 003 | ADR-005 gates | `evaluateGate`, `advancePhase`, `PhaseDefinition`, `PhaseAnswer`, `GateResult` | `unknown_requires_explanation`, `gate_rechecked_in_domain`, `invalidated_phase_marks_later_review`, `template_version_stable` |
| 004 | ADR-003 canon SQLite; ADR-004 conceptos; ADR-005 gates; ADR-008 ontología; ADR-010 IPC versionado | `createItem/updateItem`, `createConcept/updateConcept`, `createRelation`, `attachLocator` | `concept_shared_uuid`, `literature_locator_or_pending`, `relation_endpoints_valid`, `p2_no_arbitrary_minimum`, `plain_text_roundtrip` |
| 005 | ADR-003 canon SQLite; ADR-009 monolito modular | `searchLibrary`, `searchKnowledge`, FTS projection | `fts_reference_match`, `fts_rebuild_matches_source`, `archived_excluded_by_default` |
| 006 | ADR-003; ADR-006 | `exportLibrary`, `exportPaper` | `export_references_resolve`, `private_paths_omitted`, `export_hashes_verify`, `failed_export_not_success` |
| 007 | ADR-003; ADR-006; ADR-012 | `createBackup`, `verifyBackup`, `restoreBackup`, `switchLibrary` | `backup_restore_fixture`, `wal_snapshot_consistent`, `restore_failure_keeps_active`, `move_reconciles_paths`, `retain_last_verified_backup` |
| 008 | ADR-003; ADR-004; ADR-005; ADR-012 | `archiveConcept/restoreConcept`, archive/restore paper/item; `evaluateGate` | `archive_restore_preserves_links`, `archive_paper_no_cascade`, `phase_edit_marks_review`, `trash_not_exposed_v01` |

## Definition of Ready antes de implementar

1. `ARCHITECTURE.md`, `DOMAIN.md` y `CONTRACTS.md` concuerdan en autoridad canónica, límites, IDs, DTO, errores, versión IPC y transacciones; IDs ADR coinciden con arquitectura revisada.
2. Cada REQ tiene dueño de servicio, persistencia/DTO, resultado observable, error y prueba; referencias cruzadas pendientes se etiquetan, nunca se inventan como contratos cerrados.
3. Se distinguen gates del piloto 0.0.1 y v0.1.0; se acuerdan labels para fases, unknown, decisiones P1 y progreso invalidado.
4. PRE/P1/P2 tienen campos obligatorios exactos, `PhaseDefinition` versionada, gate reproducible, política de historial y revalidación de dominio; P2 admite ausencia justificada de candidatos sin mínimos arbitrarios.
5. Restore/switch y migraciones especifican WAL, archivos, manifest, bloqueo, retención, fallo/interrupción y recuperación. Archive/restore especifica dependencias. `TRASHED` está reservado y no expuesto; no se habilitan papelera, merge ni borrado permanente en v0.1.
6. Límite del lector/protocolo/rutas, capacidades Tauri/CSP, sanitización, logs privados y exportación están especificados; hay pruebas traversal/documentos desconocidos.
7. Fixtures y pruebas son pendientes identificadas. Se separan pruebas con mock, integración real, QA Windows instalado y revisión manual cuando sea necesaria.
8. Presupuestos NFR son propuestas hasta medirse; plan de versión/migración prueba snapshot previo, checksums, downgrade y esquema futuro.

Si un punto sigue abierto, se registra en el documento dueño antes de implementar su comportamiento dependiente.



Precisión SPEC-003 / ADR-020: savePhaseAnswer en NOT_STARTED devuelve GateBlocked sin efectos ni receipt nuevo (replay durable primero). Editar una fase ya iniciada aunque no sea la activa conserva activePhaseCode y aplica CAS/invalidation; guardar nunca inicia ni acepta fases. Cubrir ambos escenarios y el stale no-op.
