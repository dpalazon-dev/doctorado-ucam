# T04 — promoción normativa de gates

2026-10-02. **DONE — promoción documental realizada; scoped rereview y commit a cargo de Sol.** HEAD al iniciar: 2a4ffc726029c29afd981ed6bbfb752d105a2862. Sin commits propios.

## Archivos

Nuevos: docs/architecture/WORKFLOW_GATES.md y docs/architecture/phase-definitions/{PRE,P1,P2}.v1.json. Actualizados con cambios acotados: DOMAIN.md, CONTRACTS.md, DATA.md, SPECS.md, ADRS.md (ADR-017 Accepted), docs/plans/tasks/task-04-brief.md y task-06-brief.md, incluidas explicaciones anexas. Única evidencia adicional escrita: este informe. AGENTS/STATUS/producto y cambios ajenos preservados; ningún subagente nuevo.

JSON copiados byte a byte de la revisión editorial, declarados canónicos/inmutables v1 y embebibles al compilar:

| Fase | definitionHash SHA-256 |
|---|---|
| PRE | ee2f0e54c352475dc5896080b3a7ed8f5425b637a3712e4a77f721a942cc4a7f |
| P1 | 8e2c71f5d3f8578ff940c9903086f982aedca69adbb704c8fa3afe6381359150 |
| P2 | 2c8059fce6d4beadf442ea9d9f5d063906b5e1542f6fe0972e1c6f85be509393 |

## Inconsistencias resueltas en el texto normativo

- Sexta salida visible PRE review_type confirma metadato mediante PhaseAnswer existente; unknown default no cuenta como procesado. P1 describe enfoque/metodología de la fuente. Resoluciones/texto/drafts y matrices cerrados sin alterar wire.
- R1: todo touch que inicializa/activa hacia delante y advance que completa requiere cadena COMPLETED, snapshot aceptado y gate vigente, además de continue aceptado P1 para P2. Lectura de fases iniciadas disponible sin aceptación implícita; NEW→ACTIVE sólo primer advance PRE.
- R2: documento vivo {id,status,sha256,available} en inputs PRE/P1/P2; proof/handle/guardas antes TX y referencia DB revalidada dentro. Sin leer/hashear 500MiB bajo actor ni revisión ficticia por desaparición externa.
- Snapshot/normalización/no-op y clock max+1 por Paper/phase/contexto, primera respuesta revision1, replay previo CAS. Terminal P1 mantiene active P1; P2 previa NEEDS_REVIEW preservada.
- P2 usa artefactos/enlaces por salida, síntesis Insight y ausencia justificada sin cuotas. OR cerrado de candidatos/ausencia; selección en asociaciones y única justificación en PhaseAnswer. Proyección cambiada incrementa answer.revision más clock en misma UoW. Payload P2 fijado; resolutor ausente UnsupportedCapability y aceptación final espera T07.
- COMPLETED reservado preservado en upgrade/get/lectura/Library archive/restore; transiciones Workflow incompatibles UnsupportedCapability sin efectos, sin normalización ni migración especial. Añadidos escenarios del reviewer a detalle/briefs/SPEC.

## Comprobaciones ejecutadas

1. `git diff --check -- docs/architecture/DOMAIN.md docs/architecture/CONTRACTS.md docs/architecture/DATA.md docs/architecture/SPECS.md docs/architecture/ADRS.md docs/plans/tasks/task-04-brief.md docs/plans/tasks/task-06-brief.md`: exit0, sin errores de diff. Git advierte normalización CRLF→LF ya asociada al estado mixto observado; no se hizo limpieza general.
2. Relectura local y comparación de bloques fenced `ts` frente al contenido previo: CONTRACTS 12, brief T04 4, brief T06 6; **22/22 intactos**, DTOs/firmas/enums/nullability sin cambios.
3. `Get-FileHash -Algorithm SHA256 -LiteralPath` para copia origen/destino: **3/3 copias byte idénticas**. Parseo JSON/shape cerrado y recálculo SHA-256 UTF-8 de serialización canónica sin definitionHash mediante `[System.Security.Cryptography.SHA256]::Create()`: **3/3 coinciden**. Conteos PRE6/6, P1 7/6, P2 7/7 salidas/obligatorias.
4. Comprobación de links Markdown locales con `Join-Path` y `Test-Path -LiteralPath`: **35 válidos**, ninguna referencia rota. Nuevos documentos sin whitespace terminal nuevo; 15 líneas preexistentes de ADRS con whitespace conservadas, fuera del delta.
5. Diff propio inspeccionado y búsquedas acotadas de formulaciones anteriores contradichas; referencias al detalle canónico en todos los documentos/briefs. Revalidación de cambios concurrentes antes de escribir: ninguna divergencia en archivos de ownership exclusivo. Estado Git ajeno AGENTS sin editar.

## Límites y autorrevisión

No suites de producto, implementación, SQLite, bibliotecas personales, navegación/UI, instalación, merge ni publicación. La disponibilidad de acceso y gates se especifica, no se demuestra aquí. ABI de prueba/handle y composición espera T03; T02/T03 y posteriores deben cumplir las responsabilidades acordadas. La adopción de Sol autoriza normativa; este informe no sustituye scoped rereview ni cierra por sí solo el informe independiente R1/R2. Root hará revisión/commit.

Autorrevisión: ownership cumplido, anexo wire intacto, no regla/enum/tabla nueva, payloads canónicos preservados, sin cuotas ni artefactos ficticios, R1/R2 completos y pruebas requeridas visibles. No se extendió diseño a tareas ajenas.
