# Revisión independiente: espacio de trabajo y grafo

Fecha: 7 de octubre de 2026
Corte revisado: `028f6ec..d7ddf6f`
Alcance: propuesta documental y limpieza editorial. No es aprobación de implementación ni de lanzamiento.

## Veredicto

**DONE_WITH_CONCERNS para la propuesta.** No encontré un cambio ABI silencioso ni una contradicción que impida seguir con la revisión documental coordinada. Antes de cerrar los contratos del catálogo `paper_link`, hay que especificar qué evidencia o declaración humana admite el tipo `cites`.

## Hallazgo

**Medio: `cites` puede sonar a hecho bibliográfico aunque el diseño permite omitir procedencia.** La propuesta presenta `cites` como tipo inicial de PaperLink y permite que la procedencia sea opcional, mientras exige contexto y localizador para una cita literal. La advertencia general de que un enlace no prueba apoyo o causalidad no define qué significa `cites`, ni separa una referencia bibliográfica comprobada de una anotación del investigador. Esto podría hacer que el grafo comunique una cita verificada sin fuente asociada.

**Corrección concreta:** en el catálogo versionado y en el formulario, definir `cites` como una afirmación registrada por el investigador y mostrar su procedencia como pendiente cuando no haya localizador. Si el producto pretende mostrar una cita bibliográfica comprobada, exigir que el enlace apunte a una Reference o a procedencia con localizador. Mantener explícito que el vínculo por sí solo no valida el contenido citado. Cerrar esta distinción en CONTRACTS/SPECS antes de activar los tipos.

## Comprobaciones

- `PaperLink` tiene identidad y ciclo de vida propios, con extremos Paper. `Relation` conserva extremos KnowledgeItem y su contexto/procedencia. Las asociaciones por Concept se presentan como derivadas y no editables. No se colapsan las tres semánticas.
- La propuesta trata los enlaces simples como anotaciones personales sin tipo científico implícito. También impide que un PaperLink satisfaga gates P2 de Relation.
- Exportación y backup describen la inclusión de enlaces y sus referencias con límites de cierre finito. Se reconoce que la política de archivados y el formato de export necesitan cerrar contrato. No se afirma que la exportación actual ya lo haga.
- El alcance está marcado como diseño solicitado, no implementación. Se bloquea activar T05 con el catálogo anterior hasta reconciliar contratos, datos, especificaciones y briefs. Esto es coherente con el estado del proyecto.
- La maqueta SVG se identifica como estática y sintética. Sus etiquetas distinguen vínculo registrado de Concept compartido, aunque como artefacto por sí solo no demuestra teclado, foco o nombres accesibles por control. La especificación textual sí exige lista accesible y equivalentes de teclado. No tomar la imagen como evidencia de accesibilidad.
- No observé cambios en firmas, DTOs, enums o comandos ABI de CONTRACTS en el delta revisado. La propuesta difiere esos nombres a una futura revisión explícita del contrato.
- La limpieza editorial conserva bloques y fragmentos de código, URLs y literales contractuales según el informe presentado. Las modificaciones revisadas son reescrituras de puntuación y separación de frases. `git diff --check` figura como ejecutado en el informe editorial, sin pruebas de producto, apropiado para ese cambio.

## Límites

Esta revisión es documental. No valida contraste medido, lectores de pantalla, navegación real, tiempos de respuesta, componente de grafo, migraciones, backup/restore ni instalador. Esos resultados siguen como criterios pendientes en la propuesta.

## Revisión de seguimiento

**Veredicto final: DONE para el diseño documental, con validación de implementación y transferencia a contratos aún pendientes.** El nuevo párrafo de `cites` resuelve el hallazgo medio: define la arista como declaración humana, diferencia la procedencia pendiente de una fuente localizable, y evita presentarla como cita verificada o como validación del contenido. También mantiene explícita la transferencia pendiente a CONTRACTS/SPECS. No queda el hallazgo anterior abierto.

Los apuntadores en `INTENT.md`, `docs/development/WORKFLOW.md` y `docs/plans/IMPLEMENTATION.md` son coherentes con el diseño. INTENT registra la ampliación solicitada y condiciona la implementación a revisar contratos. WORKFLOW fija la política editorial y protege sintaxis, citas, evidencia histórica y datos del investigador. IMPLEMENTATION bloquea despachar T05/T07 con el catálogo anterior, exige cerrar ADRs/contratos/migración/export y preserva el gate nativo pendiente de T04c.

Sobre la limpieza editorial, la revisión inicial examinó el informe del autor y muestras/deltas de los contratos y especificaciones relevantes. No fue una comprobación independiente carácter por carácter de los bloques y enlaces en los 40 documentos. El informe editorial atribuye la conservación de bloques de código, fragmentos, URLs y literales a su propia comparación. La comprobación adicional del coordinador, que compara las secuencias alfanuméricas ignorando mayúsculas y puntuación en los 40 archivos, encontró solo adiciones de conjunciones «y» en ADRS y ninguna diferencia en los otros 39. Registro esa evidencia como comprobación del coordinador, no como trabajo independiente mío. La comprobación de puntuación y diff-check indicada por el informe sigue siendo documental, no una prueba del producto.
