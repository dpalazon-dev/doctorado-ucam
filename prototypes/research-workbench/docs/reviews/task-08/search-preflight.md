# T08a — preflight acotado Search/FTS

**Entrega: DONE_WITH_CONCERNS.** Investigación documental y lectura de código, 2026-10-04, agente `/root/task08a_search_preflight`. No es un brief activado ni una norma aceptada. Las recomendaciones siguientes requieren decisión de Sol y publicación coordinada en los documentos pertinentes antes del despacho.

**Corte:** `130b261f4bb4841a8c0ed1c990ebbedc6acde5ff`, `integration/v0.1`, lectura en `.worktrees/integration`. Durante la lectura avanzó únicamente documentación de checkpoint/README a `2452e352bbcce7d0bb9e478d00370f33d039d5f5`; `git diff --name-only 130b261..HEAD -- src src-tauri docs/architecture docs/plans/TASK05_PORTS.md docs/plans/tasks/task-08-brief.md` devolvió vacío. Las referencias de producto de este informe corresponden al primer corte. T05/T06/T07 no están implementadas. T04c fix1 conserva la única autoría de producto activa.

## Conclusión ejecutable para Sol

Existe el contrato de transporte Search completo y autorizado, pero no su implementación. No falta un framework: se pueden reutilizar actor, composición, permisos, envelopes y transacciones existentes. Faltan decisiones concretas para que las pruebas tengan un resultado esperado único: semántica de scopes y comandos, filtros entre clases, tokenización literal, ranking global/cursor, contexto del hit y recuperación del índice. También faltan transferencia explícita de archivos compartidos para los hooks transaccionales y el corte de migración 0004.

La combinación lifecycle/includeArchived **ya está fijada**, no es una decisión abierta. T05 define un futuro esquema y ABI aceptados, pero aún no demuestra tablas, servicios, decoders ni puntos de integración disponibles. No activar un worker T08a sobre el schema2 de este corte ni construir servicios futuros para simular dependencias.

## 1. Normas consultadas y alcance

- `AGENTS.md`, `INTENT.md`, `docs/STATUS.md`, `docs/plans/tasks/task-08-brief.md` (apartado 08a), `docs/development/WORKFLOW.md`.
- `CONTRACTS.md`: Search, límites/envelope, Provenance para navegación, registry cerrado y **regla transversal final** de lifecycle/includeArchived.
- `DATA.md`: tablas canónicas, FTS, secuencia 0001–0004 e inmutabilidad de migraciones; `SPECS.md`: SPEC-005; `DOMAIN.md`: identidad Concept/archivo sin cascada; `QUALITY.md`: coherencia FTS/fault injection/corpus; ADR-002/003 para FTS5 y canon.
- `docs/plans/TASK05_PORTS.md`: ACCEPTED/ADR-022, implementación pendiente; normalización, autoridades Concept, schema0003, consultas y alcance de mutaciones. El closure transitivo P2 allí definido no se convierte automáticamente en semántica de filtros Search.

No se investigaron export, backup/restore ni sus operaciones. Se menciona 0004 únicamente porque su numeración e inmutabilidad afectan el despacho de búsqueda.

### Lo que ya está fijado

| Tema | Norma vigente que debe preservar el brief |
|---|---|
| Corpus FTS | `papers_fts`: entity_id UNINDEXED, title, authors_text, venue, domain. `items_fts`: entity_id/type_code UNINDEXED, title, body_text. Tokenizer `unicode61 remove_diacritics 2`. No PDF/OCR. |
| Concept | Es KnowledgeItem de tipo concept; preferred_name es canon y title su mirror atómico; definición solamente base.body_text. UUID global, homónimos permitidos, sin merge. Búsqueda Concepts puede usar esa proyección, sin otra tabla FTS contractual. |
| Consulta | Texto literal tokenizado, todos los términos requeridos/AND; sin operadores avanzados ni SQL del usuario. Query hasta 1000 caracteres. |
| Archivo | Lifecycle ACTIVE exige includeArchived=false; ARCHIVED/ALL exigen true. Las otras tres combinaciones dan InvalidInput. Paper ACTIVE agrupa NEW/ACTIVE/COMPLETED; TRASHED excluido incluso ALL. Items/Concept sólo ACTIVE/ARCHIVED. |
| Integridad | Índice derivado; proyección en la misma TX de la mutación canónica; filtros lifecycle después del match. Rebuild desde canon bajo mantenimiento, confirmado antes de declarar utilizable. Corrupto/fallo de rebuild distinto de vacío. |
| Resultados | UUID público; rowid FTS interno. Empate de relevancia por UUID. Excerpt máximo 500 caracteres con HTML neutralizado. Rank no expresa confianza ni calidad científica. |
| Paginación/wire | PageDto con nextCursor opaco y total opcional; limit 1..500, 100 por defecto de cliente. Request.limit es requerido, no tiene default serde. Lecturas sin receipt durable. |
| Límites de indexación | La lista exacta no incluye DOI/año/reviewType, aliases, attributes_json, citas de Provenance, relaciones ni respuestas de fase. No añadir esos campos interpretando libremente «metadatos». Alias/nombre normalizado sí tienen uso en suggest de Concept; eso no prueba indexación de alias en Search. |

Normalización T05: canonical_text cambia CRLF/CR por LF y trim exterior; normalized_name separa whitespace Unicode, une con espacio y lowercase, **conservando diacríticos**. Dominio Knowledge se compara por igualdad de esa normalización en Concept propio o cualquier Concept vinculado, OR existencial, sin duplicar items ni inferir dominio desde Paper. La equivalencia de tokens FTS no debe modificar ese canon: búsqueda `agua` puede encontrar `Água`, mientras filtro domain `agua` no equivale a `água` bajo la regla T05.

## 2. Mapa de código realmente integrado

| Superficie | Evidencia en el corte | Disponibilidad real |
|---|---|---|
| DTO Rust | `src-tauri/src/transport/dto.rs:1334–1408`, Args 2098/2105 | Request, scopes, filters, hit y Args existen; deny_unknown_fields, nullable requerido, UUID. Rank f64; lifecycle del hit String. No origin/documentId/provenanceId en SearchHitDto. |
| TS y adaptación | `src/shared/contracts/generated/contracts.ts`, `ports.ts` SearchApi; `wire.ts:81/83/153/154`; `adapters/tauri/client.ts` search | Dos métodos exactos; schema estricto/límites y envelope común. Generador Rust `bin/generate_contracts.rs:90–95,165–166`. No reescribir DTO TS a mano. |
| Registry/ACL | `transport/commands/mod.rs:59–60,374–383`; `build.rs:58–59`; `permissions/research-read.toml`; `capabilities/main-local.json` | `search_library`/`search_knowledge` registrados y autorizados a main. Dispatch sólo deserializa y retorna UnsupportedCapability. Permiso no acredita servicio implementado. |
| Capability | `modules/settings.rs:18` | search=false; no activación por crear tabla o por existir DTO. |
| Search interno | Inventario de archivos/módulos | No domain/search.rs, application/search_ports.rs, modules/search ni search_repository.rs, commands/search.rs ni tests/search.rs. No SearchPersistence/SearchRepository implementados. |
| Actor | `adapters/sqlite/actor.rs:164–245` | Una conexión en hilo research-db, cola 64, submit genérico FnOnce(&mut Connection), try_send no bloqueante, Busy por saturación/cierre. WAL/FULL, FK ON y timeout 5 s. Trabajo admitido sigue aunque se abandone el receptor. |
| Maintenance | `desktop/maintenance.rs` | begin_operation cuenta lecturas/mutaciones admitidas; begin_maintenance sólo funciona si active=0 y sin cierre/otro mantenimiento. Devuelve Busy si hay operaciones; **no** bloquea admisión y drena automáticamente. wait_for_idle espera, no reserva mantenimiento. |
| Composición | `desktop/lifecycle.rs:79–163`, `lib.rs:8–104` | DesktopState contiene actor/maintenance y servicios Library/Reader/Workflow, sin Search. Mismo actor y RequestRegistry compartido; RecoveryStatus protege mutaciones; import recovery usa permiso de mantenimiento. No index health/rebuild coordinator específico. |
| TX y receipt | `application/unit_of_work.rs`; `adapters/sqlite/receipts.rs` | with_transaction IMMEDIATE + commit; with_receipt hace replay antes del caso, canon/audit/receipt en esa TX. Repositorios existentes reciben TX prestada; no abrir otra conexión/commit en proyección. Search lectura no usa with_receipt. |
| Mutaciones Paper | `application/library.rs:382,435,452,471,558`; `adapters/sqlite/library_repository.rs` | Helpers update_metadata/confirm/archive/restore; update autores y venue en misma TX, import recovery reutiliza confirm. Ningún hook FTS. Workflow también puede modificar lifecycle (p.ej. P1 archive); lifecycle futuro debe leerse del canon. |
| Consultas Paper | `library_repository.rs:1251–1369` | list_papers usa LIKE parametrizado, filtros y orden updated_at DESC/id ASC, cursor hex de updated_at\|id. No es FTS ni tiene filtro/query fingerprint o generación. Reusar patrón actor/parametrización/limit+1, **no** copiar cursor/LIKE como Search. |
| Schema/FTS capacidad | `migrations.rs:9–22`; Cargo.toml; `tests/desktop_bootstrap.rs:111–121` | SCHEMA_VERSION=2, sólo0001/0002; rusqlite0.40.2 bundled+backup. Existe test que crea FTS5 temp con tokenizer requerido y match investigación/investigacion. Sólo leído: no ejecutado aquí y no acredita proyección persistente Search. |

El adapter general traduce cualquier rusqlite::Error a StorageUnavailable (`transport/error.rs`). La búsqueda necesitará traducción local de corrupción FTS a IntegrityFailure y validación previa de query/cursor; no presentar errores sintácticos generados por el adapter como «sin resultados».

## 3. Ausencias y decisiones para Sol, con alternativas mínimas

**Todas las recomendaciones de esta sección son propuestas, no cambios de norma.** Una decisión puede documentarse ahora, pero el worker sólo se despacha después de revalidar dependencias integradas.

### D1. Scopes y diferencia entre searchLibrary/searchKnowledge

Mismo RequestDto en ambos comandos; no está fijado si searchKnowledge rechaza papers, lo ignora o es alias de searchLibrary. Tampoco si knowledge incluye Concept, cómo evitar duplicar Concept con scopes knowledge+concepts, o qué significa scopes=[].

Propuesta: searchLibrary admite las tres clases; searchKnowledge admite knowledge/concepts y rechaza papers expresos; scopes no vacío, deduplicado y canonizado para cursor. Elegir una sola representación de Concept cuando scopes se solapan. Sol debe escoger entre knowledge inclusivo (coherente con KnowledgeType=concept, clasificar hit como concept y deduplicar por identidad) o clases disjuntas (más fácil de explicar, pero knowledge solo ya no incluye los doce tipos). No convertir el segundo comando en alias silencioso. Fixture con concept en ambos scopes debe retornar una entidad, no dos; ningún rowid es identidad.

### D2. Matriz de filtros entre clases y referencias archivadas

Regla lifecycle/includeArchived cerrada; falta semántica por clase para conceptId/typeCodes/confidence en papers y paperId en conceptos globales. Ignorar filtros inaplicables devuelve resultados que aparentemente no los satisfacen; inferir confianza/tipo de Paper desde items añade semántica no autorizada.

Propuesta mínima: filtros de Knowledge sobre hits Item/Concept se resuelven en canon; Paper usa su domain canónico y paperId propio. Para clases sin esos atributos, excluirlas cuando se solicite un filtro inaplicable, o rechazar scope+filtro incompatible completo; Sol debe elegir explícitamente. No inventar confianza de Paper. Para paperId/item y conceptId/item elegir asociación directa o closure P2 transitivo: no asumir closure porque exista para gates. conceptId puede significar self Concept más enlazados; fijarlo también.

Conservar regla domain de T05 para Item/Concept y decidir igualdad para Paper (list_papers hoy es trim/igualdad exacta; no imponer normalización T05 a Library incidentalmente). Implementar joins como existencia para no multiplicar hits por varios links/provenances. Lifecycle del **resultado** gobierna su inclusión; archivar Paper no archiva el conocimiento global. Decidir si un filtro paperId explícito permite consultar items activos del Paper archivado sin ocultarlos; recomendación sí, con contexto archivado visible al abrir. Enlaces históricos a Concept archivado deben seguir resolviendo, sin archivar al padre por propagación.

### D3. Literal tokenizado sin convertir puntuación en frases

AND literal está fijado, falta el procedimiento exacto para extraer términos conforme unicode61. Parametrizar MATCH no neutraliza operadores. Encerrar toda query en comillas impone frase/orden; partir sólo por espacios y citar `agua/residual` también produce una frase de varios tokens, aunque los términos normativos son independientes.

Elegir y probar un único adaptador de extracción compatible con unicode61: posible uso del tokenizer SQLite del binario o proyección temporal/vocabulario de prueba; una segmentación Rust sólo es alternativa si demuestra paridad, no si sustituye silenciosamente categorías Unicode. No hay helper/dependencia de tokenización existente en el corte. El brief debe fijar query vacía/whitespace y texto sin tokens; propuesta devolver vacío sin MATCH, en lugar de listar toda biblioteca. AND/OR/NOT/NEAR escritos por usuario son términos, comillas/asterisco/colon no operadores, duplicados no alteran semántica. No agregar prefix/stemming ni frase por accidente.

### D4. Ranking global y cursor estable

UUID resuelve empate; faltan orientación/pesos y comparación entre papers_fts e items_fts. Propuesta sencilla para revisión: BM25 sin pesos especiales, ascendente, mezcla de ambos corpus con advertencia de ranking léxico local; decidir si ese orden global es aceptable, frente a prioridad por clase (menos comparable, pero predecible). Sin tercer índice ni «confianza por relevancia». Definir desempate final entityType sólo si una colisión UUID entre clases pudiera existir; Concept deduplicado primero.

Cursor propuesto versionado y acotado: comando/scopes/query efectiva/filtros canónicos, último rank/UUID/clase y libraryId. Rechazar incompatible/alterado con InvalidInput, valores no finitos y payloads enormes; formato interno no expone SQL/rowid. Mantener precisión del rank sin redondeo que cambie frontera.

**Problema adicional:** insertar/editar filas puede cambiar BM25 de todo el corpus; sólo rank+UUID no garantiza estabilidad entre páginas tras commits. Elegir contrato de páginas sobre corpus sin cambios, o generación de búsqueda actualizada atómicamente en mutaciones relevantes/rebuild que haga Conflict/reinicio cuando cambie. Recomendación generación si se exige estabilidad frente a cambios; coste una metadata interna y mantenerla en cada TX relevante. No mantener snapshot/transaction largo entre IPCs. Si se elige estabilidad sólo sin cambios, declararlo y probarlo sin prometer más. La generación/fingerprint no existe en el código actual.

### D5. Excerpt, origen visible y navegación

Discrepancia verificable: REQ-005-03 pide resultados con tipo/contexto/**origen**, pero SearchHitDto no contiene origin. Además paperId/pageIndex no identifica Document ni Provenance cuando un item tiene varias fuentes. No autoriza extender DTO unilateralmente.

Alternativa mínima: navegar siempre por entityType/entityId al detalle canónico; allí se muestran origin y lista de procedencias T07, y se elige el localizador antes de abrir Reader. paperId/pageIndex sólo se emiten cuando tienen un significado documentado inequívoco; en ambigüedad null, sin elegir la menor página como si fuera la fuente. Coste: origen no visible en lista de búsqueda hasta consultar detalle; Sol debe precisar aceptación de REQ-005-03 o aprobar hidratación de detalles para presentación. Alternativa ampliar DTO con origin/localizador inequívoco exige contrato/generación/fixtures y nuevas reglas de multiplicidad; no es necesaria sólo para navegar al objeto.

Excerpt debe venir del texto indexado canónico; no sustituirlo por cita literal de Evidence/Provenance ni insinuar atribución científica. Propuesta texto sin markup de resaltado, React text rendering y neutralización acordada para DTO; seleccionar ventana determinista y limitar **después** de neutralizar HTML, sin cortar caracteres/surrogates ni exceder wire 500. Decidir si 500 cuenta caracteres Unicode Rust o unidades UTF16 TS (hoy Rust chars y Zod string length difieren para no-BMP). Es una precisión de frontera, no permiso para truncar inputs: sólo excerpt derivado tiene límite de salida.

### D6. Proyección y hooks transaccionales

Propuesta mínima: tablas FTS ordinarias con contenido derivado propio, dos tablas exactas; una función interna de reemplazo por UUID para Paper y otra Item/Concept, llamada desde los helpers que ya poseen TX. Facilita columnas agregadas de autores/venue y evita mapping externo de UUID↔rowid adicional. Alternativas external-content/triggers añaden mapping, agregación y riesgo de dobles authorities; no hay necesidad demostrada aquí. No imponer SQL ni nuevo trait ahora.

El brief original asigna projection.rs/search_repository.rs, pero reserva a Sol hooks/composición. Hace falta transferencia explícita de `application/library.rs` o wrapper/repository pertinente, futuros knowledge cases, `lib.rs`, `desktop/lifecycle.rs`, commands/mod.rs, migrations.rs y modules/settings.rs. Ubicar SQL exclusivamente en adapter; dominio no contiene rusqlite. Si se introduce SearchPersistence/puerto interno, fijar su responsabilidad siguiendo ReaderPersistence y TX prestada, sin repository genérico ni modules duplicado con SQL.

Cobertura mínima de invalidación:

- Nuevo Paper confirmado **y recovery confirmado**; edición title/authors/venue/domain reemplaza proyección completa en la TX de audit/receipt. ReuseExisting/replay no duplica ni reindexa como creación. No indexar intents STAGING/PROMOTED como Papers.
- Item creado/editado; Concept creado/renombrado/definición usa mirror/base atómicos. Alias no cambia FTS bajo columnas actuales, pero sigue siendo canon. Si posteriormente se decide buscar alias, requiere cambiar norma y cobertura.
- Lifecycle/confidence/domain/links son filtros canónicos aunque no cambien texto; no borrar FTS al archivar porque ARCHIVED/ALL deben encontrarlo. Actualizar generación si se adopta cursor invalidable. Cambios bibliográficos de lifecycle vía Workflow también importan para filtro/generación.
- T07 locator/hash modifica contexto navegable, no title/body indexado; Relation/provenance quote no entran en FTS. Evitar rebuild/reindex general en toda edición de Provenance. Si excerpt/page/context se consultan en canon, su lectura debe ser consistente con hit.

Fallo de proyección revierte canon/audit/receipt/revisiones; fallo posterior del receipt también revierte FTS. Índice inconsistente no se arregla con «evento eventual» después del commit. SQL de proyección sin nested tx, nueva conexión ni receipt propio.

### D7. Rebuild, salud y recuperación sin inventar comandos

Norma exige mantenimiento y error distinto de vacío; no existe comando rebuild/index status ni DTO de estado de índice. Registry v1 cerrado: no añadir endpoint por iniciativa del autor. El rebuild interno necesita un punto de ejecución aprobado (inicio tras recovery, reintento controlado o nueva extensión de contrato decidida por Sol).

Propuesta: reconstrucción completa desde canon, no desde copias FTS de texto posiblemente obsoletas; una TX del actor, permiso mantenimiento propietario hasta commit/rollback, verificación antes de disponible. Fallo conserva el estado previo y Search retorna IntegrityFailure/Busy acorde a salud, nunca página vacía falsa. La salud puede ser estado interno derivado, sin nuevo DTO, pero SPEC pide ofrecer reintento: Sol debe decidir si reabrir la aplicación activa el retry interno suficiente o si se requiere una acción nueva versionada. No reconstruir silenciosamente dentro de una lectura sin acordar la mutación/mantenimiento.

Un índice autocoherente con filas/textos ausentes respecto al canon puede pasar su comprobación interna. Falta definir cómo detectar esa discrepancia: comparación contra proyección canónica al iniciar/verificar, o revisiones de canon/proyección junto a una comprobación de contenido acordada. quick_check genérico y éxito de MATCH por sí solos no demuestran paridad. Sol debe fijar alcance de detección y escenarios de corrupción, sin prometer detección universal de alteraciones arbitrarias.

begin_maintenance actual retorna Busy si hay lector/mutación; para el mínimo basta reintento finito cuando biblioteca esté ociosa, sin simular que drena. Si se exige bloqueo de admisión seguido de drain, Sol debe transferir/definir ese cambio compartido antes del worker. Drop de una Promise no debe soltar permiso mientras el job actor continúa: reutilizar trabajo propietario de Reader o mover ownership equivalente hasta finalización, sin segundo scheduler. Read-only/future schema es diagnóstico: no crear tabla temp/proyección persistente ni ejecutar rebuild en esa biblioteca.

### D8. Límite de migración y despacho

0004 es diseño, no SQL publicado. Su brief incluye FTS y futuros metadatos de portabilidad, pero una migración liberada es inmutable. Antes de T08a Sol debe decidir si0004 se integra sólo cuando todo su DDL esté cerrado, o si el corte FTS usa0004 y tareas posteriores añaden nuevas migraciones. No editar0004 liberada para acomodar08b/c. Esta decisión no exige diseñar portabilidad en este preflight.

## 4. Pruebas propuestas con oráculos observables

No ejecutadas; deben escribirse en el worktree T08a después de cerrar D1–D8. Fixtures SQL etiquetadas como schema tests no acreditan servicios T05/T07 pendientes.

| Escenario | Resultado que debe fijarse/probarse |
|---|---|
| Capacidad/upgrade | FTS5 real del binario, upgrade 0003→0004/reopen; tokenizer/columnas exactas y checksums previos intactos. No tests sólo en otra SQLite instalada. |
| Corpus positivo/negativo | Matches por title/authors/venue/domain Paper y title/body Item; Concept name/definition por mirror. Texto único sólo en PDF/quote/alias/attributes/fase no aparece mientras esté fuera del corpus. |
| Unicode/literal | investigación↔investigacion, mayúsculas/diacríticos compuestos; alpha…beta en orden invertido/separado sigue AND; `alpha/beta`, comillas, OR/NOT/NEAR/asterisco/colon no activan sintaxis; query vacía/puntuación conforme D3. |
| Scopes | knowledge/concepts juntos no duplican UUID Concept; scopes vacío/duplicado; searchKnowledge con papers según D1; enums inválidos y unknown fields InvalidInput. |
| Archivo | Las seis combinaciones lifecycle/bool: tres válidas, tres InvalidInput. ACTIVE Paper devuelve NEW/ACTIVE/COMPLETED; ALL nunca TRASHED. Archive no desaparece de ARCHIVED/ALL. |
| Filtros combinados | paperId+conceptId+type+domain+confidence según matriz aprobada; varios vínculos/provenances producen un hit; domain normalizado conserva diacríticos; items activos de paper/concept archivado según D2, sin cascada. |
| Empates/cursor | Corpus idéntico da rank empatado y UUID estable; caminar con limit 1/100/500 equivale al orden completo sin pérdidas/duplicados. Cursor de otro comando/query/filtro/biblioteca, malformado y rank no finito rechazado. Mutation/rebuild entre páginas conforme D4, nunca falsa estabilidad. |
| Límites/excerpt | Query 1000 admitida y 1001 rechazada antes de normalizar; limit 0/501/non-int rechazado; title largo/HTML/script/ampersand/emoji no ejecutable y excerpt<=500 conforme unidad aprobada; neutralización no infla más allá del wire. |
| Navegación/origen | Entity ID abre objeto exacto; Concept compartido no se clona; item con dos fuentes no inventa paper/page; origen visible en el punto aprobado D5. T07 STALE/PENDING/múltiples locators se eligen/explican sin navegación falsa. |
| Rollback de writer | Fault después de canon antes de FTS, después de FTS antes de audit/receipt y en commit: reopen conserva último estado íntegro; no receipt/revisión/canon/FTS parcial. Reintento mismo request no duplica proyección. |
| Writer coverage | Import normal y recuperación, metadata autores/venue/domain, Concept rename/body, lifecycle por Library/Workflow, confidence y asociaciones por T05: resultados inmediatos reflejan commits; replay/no-op no cambia generación si no hubo cambio relevante. |
| Rebuild referencia | Alterar proyección derivada en fixture, rebuild desde canon dos tablas; resultados/IDs/textos igual a referencia independiente preparada desde fixtures canónicas. No comparar sólo contra la misma función productora. Repetir/reopen conserva resultado. |
| Corrupción/rebuild fallido | Índice corrupto, tabla/filas ausentes o mismatch detectable: error distinguible de no-match; fault a mitad de reconstrucción no declara salud disponible. Biblioteca realmente vacía devuelve vacío exitoso. |
| Mantenimiento/cierre | Búsqueda en curso mantiene permiso hasta fin incluso receptor abandonado; rebuild con operaciones devuelve Busy o drain acordado; nuevas operaciones durante rebuild Busy; cierre finito mantiene lock hasta final; saturación actor no bloquea UI. |
| Futuro/diagnóstico | Schema superior no crea FTS ni modifica DB; capability/state no anuncia Search utilizable. Rebuild requiere writable y contexto compatible. |

QUALITY propone búsqueda básica p95 ≤ 500 ms a 20 resultados con 1k papers/10k items y filtros, sobre hardware de referencia y al menos 30 repeticiones tras calentamiento, frío/caliente separados. Es presupuesto de diseño **no medido ni aceptado todavía**, no un PASS de rendimiento. Usar su corpus sintético, registrar seed/hardware/filtros y evitar scans duplicados por joins. Tests IPC/contratos no acreditan UI, instalador o equipo limpio.

## 5. Dependencias a revalidar antes de un brief implementable

1. **Tras T05a:** DDL0003 real/decoder único/authorities Concept y aliases, asociaciones e índices y checksum. Verificar mapping de columnas contra TASK05_PORTS; no SELECT sobre tablas «aceptadas» inexistentes.
2. **Tras T05b:** servicios/casos TX realmente integrados, puntos create/update/lifecycle/link y su replay/no-op. Revisar cuáles necesitan hook y qué consultas/filter normalization se pueden compartir sin duplicar authority; no importar módulos de servicios para hacer SQL.
3. **Tras T06/T04c:** mutaciones relevantes de lifecycle Paper/candidatos/gates y composición; las respuestas Workflow siguen fuera del corpus. Confirmar que la generación, si aprobada, cubre cambios de filtros sin depender de clocks P2 como generación global.
4. **Tras T07:** Provenance attach/update/check hash y contexto de navegación real; canonical details y selección de fuentes para múltiples Document; resolver expectativa de origen y REQ-005-03 con T07 consumible, sin inventar interfaz futura ahora.
5. **En BASE T08a:** árbol/actor/maintenance/ACL/read-only/composición vigentes, D1–D8 aceptadas con escenarios esperados y ownership transferido. Dos branches Search pasan a handlers reales sólo cuando integración completa revisada; Sol cambia capability verdadera y registra evidencia. Revisión Rust obligatoria y TS si cambia contratos/tests/adaptación.

## 6. Referencia primaria SQLite consultada

Consulta puntual oficial; no investigación de paquetes ni código añadido:

- [Strings y frases](https://www.sqlite.org/fts5.html#fts5_strings), [unicode61](https://www.sqlite.org/fts5.html#unicode61_tokenizer): citar texto neutraliza operadores, pero varios tokens citados constituyen frase; unicode61 define categorías/separadores y diacríticos.
- [BM25](https://www.sqlite.org/fts5.html#the_bm25_function): menor valor significa mejor coincidencia; estadísticas pertenecen a cada tabla. La recomendación de mezcla de corpus es decisión de producto de este informe.
- [Snippet](https://www.sqlite.org/fts5.html#the_snippet_function): el límite nativo es por tokens, no500 caracteres; límite contractual adicional necesario.
- [Integrity-check](https://www.sqlite.org/fts5.html#the_integrity_check_command), [Rebuild](https://www.sqlite.org/fts5.html#the_rebuild_command): verifican/reconstruyen según contenido FTS configurado; reconstruir índice de su propio contenido derivado no equivale a repoblar textos canónicos de la aplicación.

## 7. Ejecución, cambios y autorrevisión

Único archivo producido: este informe central. Sin producto, SQL nuevo, commits, configuración, memoria escrita, agentes hijos, builds/tests ni STATUS/ledger. Lectura de memoria auxiliar sólo identificó reglas generales del proyecto; evidencia técnica y estado provienen del corte Git actual, no del estado histórico de memoria. OpenViking no consultado ni persistencia afirmada.

Comandos de inspección representativos exactos, todos en `.worktrees/integration` salvo ruta absoluta del informe: `git rev-parse HEAD`; `git status --short`; `rg --files src src-tauri`; `rg -n 'Search|search|FTS|fts|0004|rebuild|index|cursor|pagin|normaliz|archived|scope' docs/architecture/CONTRACTS.md docs/architecture/DATA.md docs/architecture/SPECS.md docs/plans/TASK05_PORTS.md`; `Get-Content docs/plans/TASK05_PORTS.md -TotalCount 118`; `Get-Content docs/architecture/CONTRACTS.md | Select-Object -Skip 506 -First 60`; `Get-Content src-tauri/src/adapters/sqlite/actor.rs | Select-Object -First 275`; `Get-Content src-tauri/src/desktop/maintenance.rs`; `Get-Content src-tauri/src/adapters/sqlite/library_repository.rs | Select-Object -Skip 1269 -First 144`; `Get-Content src-tauri/tests/desktop_bootstrap.rs | Select-Object -Skip 104 -First 18`; `git diff --name-only 130b261f4bb4841a8c0ed1c990ebbedc6acde5ff..HEAD -- src src-tauri docs/architecture docs/plans/TASK05_PORTS.md docs/plans/tasks/task-08-brief.md`.

Hubo consultas de rutas TS supuestas inexistentes (`src/shared/ports`, `src/shared/ports.ts`), un patrón de brace expansion incompatible con PowerShell y un patrón de lockfile mal escapado. Se corrigieron usando inventario `rg --files`, rutas reales `src/shared/contracts/ports.ts` y argumentos PowerShell explícitos/comillas simples; esos fallos no son validación de producto. Algunos resultados amplios fueron truncados; las conclusiones concretas se confirmaron mediante lecturas de secciones acotadas. El coordinador corrigió la ausencia inicial de lectura de la regla transversal; el informe final la trata como norma cerrada.

Autorrevisión: wire existente separado de runtime; propuestas etiquetadas; archivo/normalización no confundidos con tokenización FTS; test de disponibilidad leído separado de test ejecutado; escenarios pendientes separados de evidencia. No se cambia SearchHitDto ni contrato; ninguna firma futura T05/T07 se declara implementada. El siguiente paso de Sol es resolver D1–D8 y revalidar T05/T07 antes de redactar/activar el brief T08a.
