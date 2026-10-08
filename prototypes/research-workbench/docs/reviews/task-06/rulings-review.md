# T06 — revisión independiente de las siete decisiones de Sol

**Dictamen: PASS. Entrega: DONE.** 2026-10-03. Revisión documental de `task-06-rulings-proposal.md`, que prevalece expresamente sobre las alternativas del preflight. No hay una contradicción de seguridad, integridad o autoridad de datos que impida publicar estas decisiones como semántica en un ADR. No acredita implementación T05/T06/T07 ni congela la ABI propuesta.

## Alcance y autoridad

Leídos AGENTS, INTENT, STATUS, brief T06, propuesta y preflight centrales; contraste restringido a CONTRACTS Workflow/Knowledge/Concept, WORKFLOW_GATES P2, DOMAIN, DATA, TASK05_PORTS y ADR-017/020/022. El corte vigente de STATUS conserva T04c activa y T05/T06 pendientes. Se respeta el único autor de producto; no se leyó WIP T04c ni código de producto. Memoria auxiliar consultada únicamente como continuidad del método; el dictamen se apoya en fuentes locales actuales.

No se ejecutaron tests, build, red, migraciones ni biblioteca personal. No cambios de normas/producto, commits, merges, configuración o subagentes. Ownership: sólo este informe.

## Contraste de decisiones

| Decisión | Resultado y fundamento |
|---|---|
| 1. Set exige P2 iniciada; summary es lectura | Compatible. CONTRACTS:209 y WORKFLOW_GATES:67 separan edición de inicio/contexto; exigirlo también al set impide crear una respuesta candidata dentro de NOT_STARTED. WORKFLOW_GATES:81–83 permite lectura de archivados y conserva restore explícito/COMPLETED reservado. Captura previa mantiene asociaciones no seleccionadas sin iniciar P2 (TASK05_PORTS:11; CONTRACTS:340). |
| 2. Decisión completa y borrador PENDING/null | Compatible. WORKFLOW_GATES:36–49 fija la alternativa exclusiva, única justificación estructurada, explanation=null y PENDING incompleto. DOMAIN:70 permite explícitamente PENDING/null. Summary puede derivar selección/count de paper_items y null en ausencia de objeto sin inventar una segunda selección ni justificar ausencia; ese borrador jamás pasa el gate. Un objeto presente debe ser completo y comprobado, no parcial. El límite de 5000 y su comprobación antes de normalizar son una precisión a publicar, coherente con CONTRACTS:338/TASK05_PORTS:21; la longitud canónica requerida sigue excluyendo whitespace vacío. |
| 3. Campos irrelevantes null; ACTIVE al seleccionar; deselección de archivado | Compatible. CONTRACTS:219 conserva claim/question directo, prioridad y rationale. Rechazar priority/rationale al deseleccionar precisa el payload sin cambiar wire. El set modifica paper_items, no el contenido del KnowledgeItem: permitir quitar una selección de item archivado no viola la guarda de updateItem/updateConcept/linkConcept de CONTRACTS:344. Replay precede guardas; CAS precede no-op (DATA:50). |
| 4. Archive/restore conserva selección y count | Compatible y necesario. DOMAIN:129–130 y DATA:46 preservan enlaces históricos; WORKFLOW_GATES:63,109 conserva archivados en proyección, pero impide usarlos como artefactos vigentes. Summary debe reflejar toda la elección conservada, incluida la archivada; UI la identifica. Archive cambia lifecycle/artifact/clock conforme T05, sin fabricar una nueva decisión candidata si selección/count/justificación no cambian (DOMAIN:148; TASK05_PORTS:11 y apartado de pruebas Candidatos). |
| 5. Save no cambia asociaciones; set actualiza valores canónicos y auditoría | Compatible. DATA:28,50 y CONTRACTS:219 fijan ambas autoridades. WORKFLOW_GATES:17,69 fija comparación de cuatro valores canónicos, primera revision1, CAS incluso para stale no-op y receipt de no-op. DATA:75–79 preserva colisión de requestId, replay y registro interno de cambios sin payload sensible en logs. Cambio efectivo de respuesta/asociación/clock/evento/receipt es una sola UoW; cambiar body no incrementa answer candidata por sí solo (WORKFLOW_GATES:111). |
| 6. Preguntas PRE como contexto y enlace manual | Compatible. DOMAIN:88 pide relacionarlas con PRE, mientras WORKFLOW_GATES:27 fija Question asociada al Paper y relación explícita de la respuesta con sus IDs. Ningún contrato introduce identidad/FK por fragmento PRE. Mostrar PRE y registrar la decisión manual satisface esa conexión sin inferir contenido ni inventar wire/schema. |
| 7. Todos los save/evaluate/advance P2 de producción esperan T07 | Compatible como corte de entrega. CONTRACTS:10,219,223 y WORKFLOW_GATES:51 exigen UnsupportedCapability cuando falta capacidad/resolutor y aplazan la aceptación final hasta T07. No exigen habilitar saves por key antes. Candidatos/captura pueden ser reales con formularios formales P2 pendientes, capacidades existentes y errors honestos. Fixture completo sólo prueba policy; no demuestra capacidades de producción ni gate real. |

## Hallazgos bloqueantes

**Ninguno.** No se observa segunda autoridad, pérdida de enlaces al archivar, bypass de CAS/replay, no-op que ignore validación, aceptación por tablas vacías, expansión por relaciones ni alteración unilateral de wire/JSON v1.

La propuesta conserva el cierre/inversa normativo del preflight: semillas paper_items, item_concepts transitivo hasta punto fijo, relaciones sólo con ambos extremos alcanzados y archivados incluidos. WORKFLOW_GATES:109–111, DOMAIN:148 y TASK05_PORTS `affected_papers` exigen el mismo alcance antes/después y una invalidación por Paper. Estas normas no se reemplazan por las siete decisiones.

## Precisiones al promover la norma y el brief

Son sincronización del corte decidido, no condiciones nuevas ni rediseño:

1. Promover la decisión 7 en el brief T06 y documentos operativos afectados: T06 entrega captura/cola y policy probada; todo save/evaluate/advance P2 de producción espera T07. La frase del preflight:126 «candidatos y saves permitidos reales» y su alternativa parcial quedan superadas por la propuesta. La aceptación completa de P2 del brief:22 se verifica tras T07, como ya fija brief:14/19; no atribuirla al hito T06.
2. El permiso semántico de save PENDING/null y save de justificación cero descrito en decisiones 2/5 es futuro hasta T07. En T06, con cero items, no puede guardarse esa justificación mediante save; la UI debe presentar esa disponibilidad pendiente y no fabricar itemId para llamar set. No hay obligación de habilitar parcialmente una key para remediarlo.
3. Publicar expresamente la excepción de borrador PENDING/null en summary y el límite de justificación junto a las reglas de candidatos. La excepción no acepta objetos estructurados discordantes ni sustituye una justificación que el gate necesita.
4. Antes de despachar T06, contrastar ABI, decoder, guardas y cierre/inversa con T05 integrada. Los traits, firmas, nombres de archivos y visibilidades Rust del preflight son propuestas, no contrato congelado ni implementación comprobada.

## Evidencia y autorrevisión

Método: lectura local y contraste línea a línea; Karpathy para supuestos explícitos/correcciones mínimas. `using-superpowers` contiene SUBAGENT-STOP, por lo que no se ejecutó su harness desde esta revisión delegada.

Comandos de lectura ejecutados incluyeron exactamente:

```powershell
Get-Content AGENTS.md; Get-Content INTENT.md; Get-Content docs/STATUS.md
Get-Content .superpowers/sdd/IMPLEMENTATION/task-06-rulings-proposal.md
Get-Content docs/architecture/WORKFLOW_GATES.md
Get-Content docs/plans/TASK05_PORTS.md
rg -n "P2|candidat|Archive|archiv|KnowledgeItem|Workflow" docs/architecture/DOMAIN.md
Test-Path .superpowers/sdd/IMPLEMENTATION/task-06-rulings-review.md
```

También se leyeron el preflight, brief, WORKFLOW y SKILL.md locales, y rangos numerados de CONTRACTS/DATA/ADRS con Get-Content y búsquedas rg. La ruta del informe no existía antes de crearlo. Único cambio: este archivo nuevo; ningún commit. La revisión documental tiene alcance semántico: no valida DDL, runtime, snapshots ejecutados, reopen, IPC, UI, instalador o equipo limpio. No requiere detener al autor T04c ni autorización adicional entre tareas.

## Revisión de la promoción documental ADR-023

**PASS**, 2026-10-03, sobre el delta documental aún sin commit indicado por Sol: nuevo `docs/plans/TASK06_DECISIONS.md`, append ADR-023/CONTRACTS, dos sustituciones y append WORKFLOW_GATES y anexos briefs T06/T07. Las siete decisiones promovidas son idénticas a las siete decisiones revisadas: comparación textual por líneas numeradas mediante `Compare-Object`, sin diferencias. Los anexos reflejan las cuatro precisiones anteriores: habilitación conjunta tras T07, ausencia de guardado cero sin items durante T06, PENDING/null explícito y ABI pendiente de contraste tras T05. Los enlaces Markdown nuevos a TASK06_DECISIONS resuelven la ruta relativa correcta; ese archivo y `docs/reviews/task-06/rulings-review.md` existen.

No hay claims nuevos de implementación ni contradicciones con el PASS original. El estado Accepted se refiere a semántica, y los documentos distinguen las capacidades por entregar, fixtures y aceptación real posterior. Conforme para consolidación documental; no se reabre ABI ni se valida producto. Lecturas: `git diff -- docs/architecture/ADRS.md docs/architecture/CONTRACTS.md docs/architecture/WORKFLOW_GATES.md docs/plans/tasks/task-06-brief.md docs/plans/tasks/task-07-brief.md`, `Get-Content docs/plans/TASK06_DECISIONS.md`, `Test-Path` de los tres informes/documentos y comparación de las líneas `^[1-7]\. ` de propuesta y documento promovido. Único cambio de esta ronda: la presente nota; sin tests, build, red, producto o commit.
