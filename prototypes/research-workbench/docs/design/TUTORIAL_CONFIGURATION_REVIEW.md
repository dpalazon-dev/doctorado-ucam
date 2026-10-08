# Revisión independiente: tutorial y configuración científica

Fecha: 7 de octubre de 2026  
Resultado: **DONE_WITH_CONCERNS — no recomiendo aprobar el diseño como brief implementable todavía.** Es una propuesta documental útil y explícita sobre sus límites científicos; mantiene separadas relevancia, origen, evidencia y confianza. Quedan decisiones importantes que afectan el contrato vigente y el próximo corte T05.

## Hallazgos

### H1 — Las relaciones configurables contradicen el alcance y los consumidores ya aceptados de T05 (importante)

La propuesta permite crear relaciones propias entre tipos core y conservar definiciones personales por `DefinitionRef=(namespace, code, version)` ([propuesta](../../.worktrees/tutorial-ontology-design/docs/design/TUTORIAL_AND_SCIENTIFIC_CONFIGURATION.md), secciones 5–6). Sin embargo, DOMAIN fija exactamente 13 relaciones y sus matrices como allowlist v0.1; DOMAIN y ADR-008 excluyen el editor genérico de ontologías. Más concretamente, el diseño aceptado de T05 ya especifica en `TASK05_PORTS.md` la migración 0003, `relation_types` con trece filas seed y catálogo `core 1.0.0`, y exige catálogo exacto/matriz en sus criterios de aceptación. Añadir definiciones personales cambia esa migración y su criterio, no es solo una pantalla de Settings.

La propuesta reconoce que hace falta aprobar una ampliación y replanificar T05; por tanto, no es una contradicción inadvertida ni una licencia para implementar extensiones en el contrato actual. Sigue faltando la decisión normativa explícita: ¿se retira la restricción de exactamente 13 tipos para v0.1 y se modifica ADR-008/DOMAIN, DATA y T05, o se limita v0.1 a presentación/favoritos/orden de las trece relaciones y se posponen las relaciones nuevas? No despachar T05a con el plan actual y este diseño simultáneamente.

### H2 — Identidad versionada propuesta no está cerrada en persistencia ni en interoperabilidad (importante)

La identidad semántica histórica está bien motivada, pero `DefinitionRef` no especifica cómo una relación guarda su namespace/código/versión, cómo se conserva cada definición publicada y su matriz para lectura futura, ni cómo se representa en los DTO/contratos. `TASK05_PORTS.md` actualmente fija `relations.type_code` contra `relation_types.code`, mientras que el export v1 tiene `RecordType` cerrado y una forma `Relation` con `typeCode`; el manifiesto declara `ontologyVersions`, pero no define la forma/versión de definiciones personalizadas. La propuesta pide incluir definiciones históricas y revisar allowlists/versiones, pero lo deja para una fase posterior.

Antes de aprobar relaciones personalizadas como capacidad v0.1, añadir un contrato de lectura/import/export: claves estables y reglas de unicidad, referencia exacta a definición inmutable, almacenamiento de versiones retiradas, serialización/manifiesto, y comportamiento determinista ante definición ausente/desconocida (solo mostrar como desconocida o rechazar import atómicamente). También decidir si “publicar v2” crea nueva definición con historial de ambas matrices o una nueva identidad; esa diferencia determina qué queda FK-referenciado. No basta con incrementar `ontologyVersion` si hay varios namespaces/perfiles.

### H3 — El backup no define qué estado de personalización/tutorial se restaura (moderado)

TU-03 separa progreso tutorial de fases/respuestas, y la sección 6 dice que preferencias visuales/progreso no se mezclan con evidencia científica compartida; en la misma sección se dice que el backup local conserva configuración pertinente para recuperar la experiencia. Esas frases dejan abierta una política observable: si el backup incluye perfil, catálogos personales y progreso; si incluye vocabulario semántico pero omite preferencias/tutorial; y qué pasa al restaurar una biblioteca en otro perfil o versión de app. El contrato de backup v0.1 actualmente incluye snapshot SQLite, PDFs y ontología/manifest, mientras que el export excluye `app_settings` y solo tiene records cerrados.

Separar explícitamente datos necesarios para interpretar registros históricos (definiciones referenciadas, que sí deben viajar con export/backup) de preferencias de presentación y progreso tutorial (decidir si son globales al equipo, locales a la instalación o parte del backup). Fijar el comportamiento en restore y en upgrade antes de diseñar las tablas/DTO; de otro modo un backup válido puede recuperar evidencia, pero cambiar silenciosamente la experiencia personal.

## Comprobaciones positivas y límites

- La propuesta distingue venue, tipo documental y `ReviewType`, y advierte que `ReviewType` es el enum que participa en PRE. El preflight confirma que `venues.kind`, tipo bibliográfico y `papers.review_type` son campos distintos; el diseño no los confunde.
- No encontré una afirmación que convierta `FACT`, `supports`, una cita, completar el tutorial o completar P2 en verdad científica. La propuesta limita explícitamente lo que significan `OBS`, confianza humana y procedencia. Las fuentes citadas se presentan como inspiración acotada, no como validación de la UI.
- El modo de práctica está condicionado a switch/recovery y aislamiento verificados, y la guía sobre espacio no guarda automáticamente. La política exacta de progreso/restore queda pendiente según H3.
- La inspección fue documental y estática de `INTENT.md`, `docs/STATUS.md`, propuesta/preflight, `DOMAIN.md`, `CONTRACTS.md`, `DATA.md` y `TASK05_PORTS.md`. No ejecuté código, pruebas ni QA; este informe no aprueba comportamiento implementado.

## Recomendación al orquestador

Mantener el documento como propuesta y cerrar H1–H3 en ADR/matriz de impacto antes de activar T05a. Luego actualizar de forma coordinada DOMAIN, CONTRACTS, DATA, ADR-008/ADR nueva, TASK05_PORTS y el contrato de export/backup. Si H1 se resuelve posponiendo nuevas relaciones, la parte de personalización visual de las trece relaciones puede continuar bajo el contrato actual, siempre que no cambie semántica ni matriz.
