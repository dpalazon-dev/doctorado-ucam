# T04 — revisión independiente de la resolución

Fecha: 2026-10-02. Resultado: **DONE_WITH_CONCERNS — dos hallazgos importantes abiertos; no PASS.**

Alcance: borrador `work/reviews/task04-gate-resolution-draft.md`, tres JSON PRE/P1/P2, preflight y secciones pertinentes de DOMAIN/CONTRACTS/DATA/SPECS y briefs T04/T06. Revisión documental independiente, sin código/WIP T02, pruebas de producto, commits ni cambios normativos. Única escritura: este informe. Los cambios editoriales de los JSON comunicados por el autor no alteran keys, reglas, resoluciones ni asociaciones revisadas.

## Hallazgos requeridos

### R1 — importante: forward touch puede saltar la aceptación PRE y NEW→ACTIVE

**Referencias:** borrador §Snapshot, revisiones y no-op, líneas 55 y 63; §Contexto y lifecycle, líneas 69–71. CONTRACTS:208–210; DOMAIN:72,78,84; brief T04:14–15.

El borrador exige reconfirmar un prerequisito NEEDS_REVIEW, pero no exige aceptar uno que nunca estuvo COMPLETED. CONTRACTS:210 habilita P1 cuando el gate PRE pasa y :208 permite que touch inicialice una fase habilitada sin completarla. Por tanto, con un Paper NEW y respuestas PRE suficientes, touch(P1) puede abrir P1 sin advance(PRE). El snapshot aceptado PRE que P1 consume todavía no existe y NEW→ACTIVE no ocurrió; una posterior decisión P1 light_read contradice «Paper sigue ACTIVE». La misma omisión permite abrir P2 con decisión continue guardada sin aceptar P1.

**Corrección mínima:** definir habilitación hacia delante como prerequisito COMPLETED, snapshot aceptado existente y gate vigente satisfecho; P2 además requiere readingDecision=continue. Aplicarla al touch que inicializa/activa una fase posterior y al advance que la completa. Touch no acepta ni completa implícitamente su prerequisito. Mantener navegación/consulta de datos ya iniciados para reconfirmarlos; no borrar P2 al cambiar la decisión P1. Promover esta precisión a CONTRACTS/DOMAIN/SPECS y briefs antes del dispatch.

**Verificación de comportamiento requerida en T04:** con PRE IN_PROGRESS aunque el gate pase, touch(P1) no inicia P1 ni cambia NEW; tras advance(PRE), PRE aceptada y Paper ACTIVE, touch(P1) está permitido. Con P1 sin aceptar o NEEDS_REVIEW, touch(P2) no inicia una fase nueva; tampoco advance(P2) puede completar hasta reconfirmar la cadena.

### R2 — importante: snapshot heredado no observa disponibilidad documental actual

**Referencias:** borrador §Snapshot, revisiones y no-op, líneas 53–57 y 65. CONTRACTS:174–176,208–210; DOMAIN:76–78; DATA:40,95; preflight punto 4, línea 21.

P1 solo proyecta hash/definición PRE aceptados y P2 solo los de P1 más continue. La disponibilidad del PDF está exclusivamente en inputs PRE. Si el archivo desaparece después de aceptar PRE, no hay mutación DB, cambio del hash histórico ni NEEDS_REVIEW. Esas proyecciones permiten que el evaluador posterior omita el documento actual, pese a la garantía de :65 de bloquear si falta PDF. Comprobar el gate vigente del prerequisito debe incluir esa dependencia viva; mirar únicamente el snapshot histórico o PhaseState no lo demuestra.

**Corrección mínima:** incluir en inputs P1/P2 la proyección documental viva relevante, explícitamente `{id,status,sha256,available}`, junto al hash aceptado del prerequisito. Evaluar y advance posteriores comprueban el documento vigente disponible y devuelven gate bloqueado si falta; las lecturas históricas siguen preservadas. No inventar incremento de clock por una desaparición externa. Definir la comprobación mediante el puerto de documentos después de fijar la interfaz T03: prueba de acceso y retención del handle fuera de la TX, referencia/documento DB revalidados en la TX corta, sin leer/hashear un PDF de hasta 500 MiB bajo SQLite. El borrador no debe prometer detección de cualquier modificación externa sin esa evidencia.

**Verificación de comportamiento requerida en T04/T06:** completar PRE/P1, quitar acceso al PDF sintético sin cambiar DB y comprobar evaluate/advance posterior bloqueado, snapshot actual distinto por available, snapshot aceptado anterior intacto y ninguna mutación de fases/lifecycle/receipts de éxito. Tras recuperar acceso, reevaluación permitida. Un cambio DB de Document debe producir CAS/invalidación en su UoW, distinto del caso externo sin clock.

## Decisiones revisadas sin hallazgo importante adicional

- La sexta respuesta PRE usa PhaseAnswer y ReviewType existentes; diferencia confirmación humana del default unknown sin añadir editor bibliográfico. Requiere la promoción normativa ya identificada por el autor. P1.review_type conserva su semántica textual distinta.
- P1.relevance_decision puede ser estructurada y carecer de prosa redundante. PENDING/null es borrador; un objeto parcial no determina una rama. UNKNOWN/NOT_APPLICABLE no inventan decisiones humanas.
- El clock max(revision) por Paper bajo IMMEDIATE evita colisiones entre revisiones activas de fases distintas, conservando el máximo en filas no borradas. Asignar tokens distintos por fase, renovar destino solo en cambio real de contexto y excluir phaseRevision del hash separa CAS de contenido. El snapshot aceptado y completedAt históricos sobreviven invalidación y navegación; state explica su vigencia. Primera escritura revision=1, no-op tras CAS y replay previo a CAS son coherentes con DATA:24,40,54,71–73.
- Mantener P1 activa en light_read/archive conserva un token identificable para navegación. Preservar P2 previa NEEDS_REVIEW y entender NOT_STARTED como recorrido nunca iniciado respeta DOMAIN:72,129 y SPECS:61. NEW→ACTIVE en advance PRE es coherente después de cerrar R1.
- Las referencias P2 no requieren tipos inventados: Question e Insight asociados al Paper; Concept directo o alcanzado por item_concepts; Relation con extremos pertenecientes al cierre y contradicts con matriz claim↔claim; Reference usa attributes.state/reason existentes (CONTRACTS:215–232,260–272; DOMAIN:112). Las seis listas son respaldo de artefactos existentes, no creación al guardar ni cuotas. Arrays vacíos justificados procesan ausencia; no sustituyen un artefacto que se afirma haber capturado.
- La selección/priority/rationale P3 queda en paper_items; la única justificación de cero candidatos cabe en structured_value_json de PhaseAnswer existente. candidates/count son una proyección comprobada y no segunda selección. El guardado con cero candidatos no necesita item ficticio. T06 debe actualizar la respuesta proyectada y su revision cuando cambie realmente, además del clock P2 en la misma UoW, conforme a la regla general de revisiones. Claims/Questions vinculadas, prioridad nullable 1..5 y rationale obligatorio corresponden a CONTRACTS:177–179,201–208; no se añadió otro campo persistente de candidato.
- La alternativa cerrada por key candidatos/ausencia puede implementarse con los ruleKeys existentes y política de dominio; completionRules no puede interpretarse como AND global. Integridad inválida bloquea ambas ramas y capacidad ausente no se convierte en ausencia justificada. Los JSON no incorporan operadores ejecutables.
- El cierre P2 y sus asociaciones permiten invalidar todos los Papers afectados por artefactos compartidos. Nombres abreviados del snapshot interno como type/body/source/target son proyecciones de typeCode/bodyText/sourceItemId/targetItemId, no nuevos atributos; mantener esa correspondencia al fijar el esquema interno. Provenance pendiente se conserva y no se transforma en calidad suficiente. No se requiere hash del cuerpo por separado si el cuerpo está dentro del snapshot.
- Pinar/leer P2 en T04 con evaluadores/guardado/candidatos UnsupportedCapability respeta migraciones 0002/0003 y despliegue progresivo. El habilitado posterior exige resolutores reales: la aceptación P2 completa sigue esperando artefactos/capacidades T07, según brief T06:14; T06 no obtiene PASS final por listas vacías mientras falta una capacidad requerida.

## COMPLETED reservado y compatibilidad

DOMAIN:64 y CONTRACTS:210 reservan Paper COMPLETED para un alcance posterior y el piloto normativo no lo produce. No hace falta construir ahora una migración de lifecycle o un recorrido especial para un «piloto legado COMPLETED» hipotético. Sí conviene fijar un guard limitado para ese enum conocido: upgrade conserva lifecycle/archivedFrom/UUID/revisiones bibliográficas; get/lectura y Library archive/restore conservan el estado; las nuevas transiciones Workflow incompatibles devuelven UnsupportedCapability con explicación segura y sin efectos. No convertir a ACTIVE ni cambiar archivedFrom silenciosamente, tampoco después de restore. Probarlo con un fixture sintético COMPLETED y ARCHIVED desde COMPLETED; no abrir bibliotecas personales. Esta decisión defensiva no bloquea T04 una vez documentada y no requiere comando/schema nuevo.

## Evidencia, comandos y autorrevisión

Lecturas locales y extracción numerada con `Get-Content`; búsquedas acotadas con `rg`. Comandos empleados para el alcance principal:

```powershell
Get-Content AGENTS.md
Get-Content INTENT.md
Get-Content docs/STATUS.md
Get-Content docs/development/WORKFLOW.md
Get-Content work/reviews/task04-gate-resolution-draft.md
Get-Content work/reviews/task04-gate-preflight.md
Get-Content work/reviews/task04-definitions/PRE.v1.json
Get-Content work/reviews/task04-definitions/P1.v1.json
Get-Content work/reviews/task04-definitions/P2.v1.json
Get-Content docs/plans/tasks/task-04-brief.md
Get-Content docs/plans/tasks/task-06-brief.md -TotalCount 23
rg -n "Phase|phase|P2|P1|PRE|Reference|reference|candidate|Candidate|ReviewType|KnowledgeItem|revision|hash|Hash|COMPLETED|touch|Concept|Relation" docs/architecture/DOMAIN.md docs/architecture/CONTRACTS.md docs/architecture/DATA.md docs/architecture/SPECS.md
git status --short
```

Se leyeron las skills Karpathy y verification-before-completion. Las secciones normativas se extrajeron con bucles PowerShell sobre rangos de líneas; no se consultó docs/session ni se exploró producto T02. No se repitió el cálculo de hashes ya ejecutado por el autor: ningún hallazgo aquí depende de discrepancia de bytes/hash y los ajustes editoriales tienen su nueva evidencia en el borrador. No se afirma verificación independiente de hashes, gates ejecutables, instalación ni UI. Estado Git observado: modificación ajena de AGENTS.md, preservada.

Autorrevisión: ambos hallazgos tienen ruta concreta, causa y corrección mínima; no se exige wire nuevo, tabla de clock, cuotas ni normalización silenciosa de lifecycle. Las correcciones propuestas por Sol aún no son normativa ni eliminan los hallazgos. Sin commits.
