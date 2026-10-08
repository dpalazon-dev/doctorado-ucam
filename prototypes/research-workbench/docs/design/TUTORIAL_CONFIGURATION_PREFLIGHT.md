# Preflight: tutorial y configuración de vocabularios

**Estado:** solo lectura. **Cortes contrastados:** producto `5d35f03` (T04c fix2) y corte documental `702d08b`. No se modificó producto ni se ejecutaron pruebas. `AGENTS.md` ya aparecía modificado al iniciar la inspección; se preservó.

## Qué existe en el producto congelado

- Shell de React: `src/app/App.tsx`, `src/app/ShellView.ts`. Navegación visible Inicio/Biblioteca/Conocimiento (deshabilitado, “próximo”)/Configuración. ShellView contempla `Knowledge`, pero App aún no lo renderiza. Hay un diálogo inicial de una sola aceptación (“Esta es tu biblioteca local”), no un tutorial guiado ni opción visible para repetirlo.
- Workflow UI presente para PRE/P1: `src/features/workflow/PhaseWorkspace.tsx`, `PhaseAnswerForm.tsx`; P2 conocimiento aún capability-gated/no utilizable. El conocimiento no está en navegación activa.
- Configuración en App muestra identidad, ubicación amigable, esquema, estado de recuperación y operaciones de biblioteca. No hay API/DTO para preferencias de usuario, vocabularios editables, símbolos, tutorial ni onboarding repetible.
- Library/Edit metadata ya modela título, DOI, año, tipo bibliográfico, dominio, venue y autores. En `src-tauri/src/adapters/sqlite/migrations/0001_library.sql` del commit `5d35f03`, `venues.kind` existe pero es nullable y clasifica el venue; no es tipo de artículo. `papers.review_type` persiste el `ReviewType` cerrado y `PaperMetadataInput` lo expone como campo separado de `venue` (`docs/architecture/CONTRACTS.md:63-72`). El enum es: `survey`, `topical_review`, `slr`, `mapping_study`, `tutorial`, `other`, `unknown`. `domain` es texto nullable. Venue usa nombre/identificador y asociación de paper; su `kind` no equivale a ReviewType. `ReviewType` es la clasificación bibliográfica del paper; P1 `review_type` describe textualmente el enfoque/metodología que declara la fuente (`DOMAIN.md:76,82`; `WORKFLOW_GATES.md:11`). No hay catálogo configurable de revistas ni de tipos bibliográficos, ni un campo separado de “tipo editorial de artículo”.
- El commit `5d35f03` incluye corrección de Workflow UI/estado, no Knowledge ni esquema de ontología. En el corte de producto el esquema es 2: están migraciones `0001_library.sql` y `0002_workflow.sql`; no existe migración 0003 ni tablas `ontology_types`, `relation_types`, `knowledge_items`, `concepts` o `relations`. El corte `702d08b` añade evidencia documental, no implementación de esas tablas.

## Lo ya fijado normativamente

- `docs/architecture/DOMAIN.md:94-125`: doce tipos core de KnowledgeItem (`concept`, `claim`, `evidence`, `question`, `gap`, `assumption`, `condition`, `limitation`, `method`, `example`, `insight`, `reference`); Dataset/Metric/Task/System y domain packs quedan posteriores. Los formularios pueden desplegarse gradualmente.
- `docs/architecture/ADRS.md:129-144` (ADR-008): ontología pequeña, tipada y ampliable con revisión; no editor genérico de ontologías ni sustitución de tipos core por texto libre. Extensiones con versiones/migraciones.
- `docs/architecture/DOMAIN.md:107-125`: 13 relaciones y compatibilidad de extremos: supports evidence→claim; contradicts claim↔claim simétrica; extends method→method y claim→claim; causes claim→claim/concept; requires method→condition; depends_on method→concept/method; works_when y fails_when method→condition; compares_with method↔method simétrica; part_of concept→concept; similar_to concept↔concept simétrica; limits limitation→claim/method; improves method→method. No autoenlaces; contexto/origen requeridos para causes/improves; contexto distinto conserva relación distinta.
- `docs/architecture/CONTRACTS.md:225-346` y `docs/plans/TASK05_PORTS.md` precisan DTO discriminado, atributos con autoridad única, Concepts globales con nombre/definición/alias/domain y códigos semánticos canónicos; no definen un mapping de símbolos gráficos de notas. TASK05_PORTS fija persistencia seed core `1.0.0`; es contrato aprobado para consumidor T05, no evidencia de que esté implementado en `5d35f03`.
- `docs/architecture/DATA.md:119-164`: migraciones liberadas son inmutables; esquema y versión de ontología son independientes. Se prevé 0003 para Knowledge/Concept/Relation; 0004 para FTS. DATA declara `app_settings` como almacenamiento de preferencias validadas, pero no define claves/contratos de configuración de vocabularios ni onboarding.
- No aparece en los contratos revisados un mapping de símbolos gráficos de notas a tipos/relaciones. Esa necesidad requiere aclarar si “símbolos” significa presentación/UI o un vocabulario persistido; no asumir que amplía el núcleo.

## Discrepancias/localizadores para la coordinación

1. `ShellView` enumera `Knowledge` y `Architecture.md:47` lo incluye entre vistas, aunque `App.tsx` marca Conocimiento como próximo y no ofrece esa vista. Es estado/arquitectura prevista, no capacidad entregada.
2. `CONTRACTS.md` describe los módulos Knowledge/Concept/Relation para v0.1 y el registry cerrado de comandos (`CONTRACTS.md:508`), pero al commit de producto existen solo permisos/configuración de comandos y migraciones hasta workflow. El backend futuro queda capability-gated según contrato.
3. El baseline acepta texto de `domain`, catálogo de Venue y ReviewType enum fijo. Configurar libremente revistas, tipos de artículo o taxonomy/domains por la persona no figura en los comandos Settings cerrados. Incorporarlo puede tocar contratos, esquema, export/import, search y compatibilidad; definir el alcance antes de implementar.
4. Tutorial repetible no está en las interfaces existentes. El único onboarding observado es el diálogo local de bienvenida que desaparece tras “Entendido”; persistencia/repetición no está especificada.

## Alcance de comparación

La revisión fue local y estática: `DOMAIN.md`, `CONTRACTS.md`, `DATA.md`, `ADRS.md`, `TASK05_PORTS.md`, fuente UI y migraciones del commit indicado. No se inspeccionó web, GUI ni se ejecutó test/build; no se infieren prestaciones instaladas.

