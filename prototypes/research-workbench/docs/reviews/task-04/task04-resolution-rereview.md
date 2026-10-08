# T04 — rereview de cierre y promoción normativa

Fecha: 2026-10-02. Resultado: **DONE / PASS documental del delta revisado.** R1 y R2 ADDRESSED; ningún hallazgo importante nuevo introducido por la promoción.

Alcance: informe `task04-normative-promotion-report.md`, diff `task04-normative-promotion.diff`, nuevos `docs/architecture/WORKFLOW_GATES.md` y `phase-definitions/{PRE,P1,P2}.v1.json`, y concordancia de CONTRACTS/DOMAIN/DATA/SPECS/ADRS/briefs T04/T06. No se reabre baseline ajena al delta. Sin código, WIP T02, commits, subagentes ni cambios normativos. Única escritura: este informe.

## R1 — ADDRESSED

WORKFLOW_GATES:57–59 exige PRE COMPLETED, snapshot aceptado y gate vigente para P1; P2 exige esa condición de P1, toda la cadena previa vigente y continue aceptado. Lo aplica incluso al activar hacia delante una fase ya iniciada: leer/consultar datos no acepta implícitamente prerequisitos. Touch no completa el prerequisito ni produce NEW→ACTIVE. WORKFLOW_GATES:77 y 85 mantienen esa transición exclusivamente en advance PRE y prohíben que goBack inicie NOT_STARTED.

Concordancia: CONTRACTS:212,220; DOMAIN:78; DATA:71; SPECS:45,65; brief T04:19,21 y anexo :184,192; brief T06:19–20 y anexo :183,191; ADR-017:263. WORKFLOW_GATES:93–94 y los briefs exigen los escenarios que reproducen el bypass original. No queda la formulación anterior «gate suficiente sin aceptación habilita forward» en las secciones promovidas.

## R2 — ADDRESSED

WORKFLOW_GATES:57 incorpora `{id,status,sha256,available}` vivo en inputs PRE/P1/P2, además de los hashes aceptados de sus prerequisitos. :61 separa prueba de acceso/handle/guardas antes de TX de revalidación Paper.documentId/identidad/estado/hash registrado dentro de lectura consistente o IMMEDIATE corta; conserva handle/guardas hasta comprobación/commit y rechaza prueba obsoleta si DB cambió. No exige lectura/hash del PDF grande bajo DbActor/TX ni promete detectar toda modificación externa. ABI queda explícitamente pendiente de T03, sin aparentar una interfaz implementada.

Ausencia/inaccesibilidad produce available=false y gate bloqueado actual, conserva snapshot aceptado y no inventa clock ni éxito durable de advance. Cambio DB de Document sí invalida/renueva en su UoW. Concordancia: CONTRACTS:214; DOMAIN:78; DATA:40,71; SPECS:61,65; brief T04:20–21 y anexo :186; brief T06:19–20 y anexo :185; ADR-017:263. WORKFLOW_GATES:95 exige desaparición después de aceptación, recuperación de acceso y cambio DB entre prueba/TX como casos distintos.

## Controles sobre el delta

- **Candidatura y answerRevision:** WORKFLOW_GATES:47–49 y 67, CONTRACTS:216, DATA:50, DOMAIN:90, SPECS:69 y brief T06:13,20 coinciden: asociaciones gobiernan selección/priority/rationale; una única justificación en PhaseAnswer; cambio real de proyección incrementa answer.revision/updatedAt y clock P2 juntos. Save con cero candidatos no crea item ficticio ni cambia selección. No-op/CAS/replay siguen separados.
- **COMPLETED reservado:** WORKFLOW_GATES:81,101, CONTRACTS:220, DOMAIN:138, SPECS:65 y anexos preservan upgrade/read/Library archive/restore y archivedFrom; transiciones incompatibles devuelven UnsupportedCapability sin conversión a ACTIVE. No se añadió una migración especial para un estado que el piloto normativo no genera.
- **Definiciones/artefactos:** se leyeron también los tres archivos JSON nuevos, ausentes del diff tracked. Mantienen las claves, matrices y handlers revisados: sexta confirmación PRE, decisión P1 estructurada, seis respaldos P2 y alternativa cerrada candidatos/ausencia. WORKFLOW_GATES:23–51 y 63–65 conservan artefactos reales, nombres de proyección ligados a atributos existentes, ausencia justificada y falta de resolutor como UnsupportedCapability. No hay quota ni campo de candidato inventado. La aceptación P2 final sigue esperando T07.
- **Contexto e invalidación:** el detalle canónico :69–85 conserva clock único por Paper, máximo persistido, snapshots históricos, P1 activa terminal y P2 anterior preservada. La promoción no añade tabla/wire ni hace que navegar borre o descomplete contenido.

## Evidencia y límites

Relectura con `Get-Content` del informe, diff completo por secciones, documento canónico numerado y tres JSON; búsquedas acotadas con `rg -n` de las condiciones promovidas y lectura de briefs/anexos. `git status --short` confirma los siete documentos tracked modificados, WORKFLOW_GATES/phase-definitions nuevos y modificación ajena AGENTS, todos preservados. La skill verification-before-completion ya leída rige este veredicto limitado a evidencia documental.

No se repitieron hashes, copias byte a byte ni comparación de 22 fences: el informe del autor registra esas comprobaciones y no apareció discrepancia concreta que justificase repetirlas. No se atribuyen esos resultados a verificación independiente aquí. Sin pruebas de producto/IPC/UI/instalación ni commits. El PASS cierra los hallazgos documentales R1/R2 y permite la siguiente integración documental; no acredita implementación del proof/handle ni de los gates.
