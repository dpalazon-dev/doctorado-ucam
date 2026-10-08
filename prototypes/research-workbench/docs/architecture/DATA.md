# Diseño de datos, persistencia y recuperación

Estado: baseline de ejecución v0.2, adoptada el 1 de octubre de 2026. Modelo lógico v0.1; no es una migración ejecutada.

## 1. Autoridad y representación

SQLite conserva entidades, relaciones, procedencia, respuestas, preferencias e historial. Los PDF son archivos administrados identificados desde SQLite; sus rutas se guardan relativas a la raíz. Export, FTS, vistas por concepto y un eventual grafo son derivados.

UUID en texto con forma canónica, fechas UTC RFC3339 y contadores de revisión enteros desde cero. Los escritores emiten milisegundos y sufijo Z; el decoder admite también +00:00 preservando strings y receipts previos, conforme ADR-019, sin aceptar otros offsets ni fechas locales. DTOs camelCase; columnas snake_case. Un cambio de etiqueta traducida no cambia enums persistidos. Ausencias bibliográficas son NULL o colecciones vacías, sin cadenas como "desconocido" que parezcan valores reales. El cuerpo inicial es body_text, plain_text versión 1; body_json queda NULL.

No se almacenan blobs PDF en SQLite. No se incluye una URL de descarga remota como sustituto de un documento administrado. Documento y paper tienen identidades distintas para conservar versiones documentales y anclajes.

## 2. Modelo lógico y restricciones

El DDL del DOCX original es el antecedente de este modelo; las migraciones que se implementen deben incorporar los ajustes explícitos siguientes, no copiarlo sin revisar.

| Grupo | Tablas y claves | Restricciones y responsabilidades |
|---|---|---|
| Identidad bibliográfica | papers(id), authors(id), venues(id), paper_authors(paper_id, author_order) | DOI normalizado único no NULL; título no vacío; autores ordenados; mismo nombre no prueba misma persona |
| Documentos | documents(id, paper_id) | Ruta relativa única; SHA-256; media type; tamaño; documento vigente seleccionado explícitamente; versiones previas preservadas si tienen anclajes |
| Imports | import_operations(id) | Token opaco; estados STAGING/PROMOTED/COMMITTED/FAILED; metadatos e IDs reservados antes de promoción; resultado de commit conservado |
| Lectura | reading_positions(document_id), app_session(singleton) | Página física >=1; zoom finito positivo; revisión; último paper abierto nullable con FK |
| Definición de fase | phase_definitions(code, version) | Documento JSON validado e inmutable por versión; hash de contenido y reglas declarativas conocidas |
| Procesamiento | paper_phases(paper_id, phase_code), phase_answers(paper_id, phase_code, question_key) | FK a versión exacta; revisión de fase y respuesta; resolución, explicación y valor estructurado validado cuando lo exige la pregunta; snapshot/hash del último gate aceptado |
| Objetos | knowledge_items(id) | Tipo válido, origen y confianza explícitos; revisión; lifecycle; cuerpo de texto independiente de cita |
| Conceptos | concepts(item_id), concept_aliases(concept_id, normalized_alias) | Subtipo de item; nombre normalizado para búsqueda, sin uniqueness global que fuerce fusionar homónimos |
| Especialización | evidence_details(item_id), question_details(item_id) | FK a item con tipo comprobado también por dominio; estados específicos separados de lifecycle |
| Asociaciones | paper_items(paper_id,item_id), item_concepts(item_id,concept_id) | Compartir un item/concepto no es procedencia; selected_for_p3, prioridad y motivo se conservan por paper/item, protegidos por revisión del procesamiento |
| Relaciones | relations(id) | Extremos distintos y existentes; tipo/contexto/origen; revisión y archive; simetría normalizada; unicidad del vínculo semántico activo según DOMAIN |
| Procedencia | provenance(id), item_provenance(item_id,provenance_id), relation_provenance(relation_id,provenance_id) | Versión documental, hash capturado, página/sección/cita, estado PENDING/LOCATED/STALE; revisión para editar un localizador |
| Evaluación futura | validations(id) | Reservada para P3; sin comandos de validación humana completa en v0.1 |
| Catálogo | ontology_types(code), relation_types(code) | Catálogo core versionado; allowlist de extremos, dirección y reglas; ningún código ejecutable |
| Operación | schema_migrations(version), audit_events(id), app_settings(key), operation_receipts(request_id), maintenance_operations(id) | Migraciones con checksum; eventos en misma transacción; preferencias validadas; idempotencia; trabajos largos con estado/resultado/error durable |
| Índices derivados | papers_fts, items_fts | FTS5 reconstruible; alimentado desde tablas canónicas y filtros de lifecycle posteriores al match |

papers incorpora revision y archived_from_lifecycle. Items y relaciones sólo alternan ACTIVE/ARCHIVED: restore siempre devuelve ACTIVE y no requieren columna de estado previo. Los campos de progreso derivados no se actualizan desde la UI.

papers incluye current_phase y last_opened_at; app_session referencia el último paper abierto. paper_phases conserva state, definition_version, revision y completed_at; phase_answers incluye revision, answer_text, resolution, explanation y structured_value_json validado por tipo de pregunta. Los valores estructurados, como la decisión P1, no se deducen mediante búsqueda de palabras en prosa. Provenance incorpora revision y updated_at si su localizador puede editarse. Todos los campos que un contrato somete a expectedRevision tienen una revisión persistida.

paper_phases guarda accepted_gate_snapshot_json/hash al completar y conserva snapshot/completed_at históricos al invalidar; state decide vigencia. Snapshot canónico y proyecciones exactas en [WORKFLOW_GATES.md](WORKFLOW_GATES.md), ADR-017: definitionHash, respuestas normalizadas/revisiones, dependencias aceptadas y Document vivo en PRE/P1/P2, artefactos/edges ordenados. No incluir phaseRevision/contexto/lifecycle/timestamps en el hash. No reconstruir historia desde estado actual.

Una modificación efectiva de una respuesta invalida conservadoramente la completitud de esa fase y marca NEEDS_REVIEW las posteriores ya iniciadas, sin tocar NOT_STARTED ni borrar datos. Cambios efectivos en artefactos usados por P2 —items, relaciones, procedencia, enlaces y selección P3— invalidan P2 si estaba completada. Una petición idempotente o que conserva los mismos valores no invalida. La regla y las revisiones afectadas se aplican en el mismo Unit of Work; no dependen de inferir la relevancia del cambio mediante texto libre.

Los índices mínimos incluyen DOI, documentos por paper, papers por lifecycle/updated_at, items por type/lifecycle, relaciones por ambos extremos, provenance por documento y asociaciones por ambos lados. Las constraints de catálogo que exigen joins se comprueban dentro del caso de uso, además de FKs y CHECK básicos.

Todos los enlaces persistentes usan FKs verificables. No hay cascadas que borren conocimiento global al archivar una fuente. Una consulta excluye archivados por defecto y puede solicitarlos explícitamente; el filtro de biblioteca ACTIVE significa no archivado, incluyendo NEW.

FTS5 inicial: papers_fts contiene entity_id UNINDEXED, title, authors_text, venue y domain; items_fts contiene entity_id UNINDEXED, type_code UNINDEXED, title y body_text. Se utiliza tokenizer unicode61 con remove_diacritics=2. Los UUID son la identidad pública, nunca el rowid interno de FTS. El caso de uso actualiza la proyección en la misma transacción de la mutación canónica; una reconstrucción completa se hace bajo mantenimiento y se confirma antes de declarar el índice utilizable. La búsqueda interpreta entrada como texto literal tokenizado, con todos los términos requeridos; no ofrece sintaxis SQL ni operadores FTS avanzados en v0.1. Un empate de relevancia se resuelve por UUID para paginación estable. La [documentación FTS5](https://www.sqlite.org/fts5.html) es la referencia del adaptador; pesos y orden son decisiones del producto, no medida de calidad científica.

Respuestas: expectedRevision=0 sólo para fila ausente, primera revision=1; CAS precede no-op y replay receipt precede CAS. Clock de fases fresco por Paper: max(revision)+1 bajo UoW, valores distintos por fase afectada y renovación destino por contexto; sin tabla nueva. No-op conserva revisiones/timestamps/snapshot; rollback no consume clock. Proyección de candidatos cambiada incrementa PhaseAnswer.revision además de clock P2. Única justificación cero candidatos: structured_value_json de PhaseAnswer(P2,p3_candidates_or_justification); selección/priority/rationale conserva autoridad paper_items. Detalle WORKFLOW_GATES.

## 3. Configuración SQLite y unidad de trabajo

Cada conexión configura foreign_keys=ON. Para la biblioteca local se seleccionan journal_mode=WAL, synchronous=FULL y busy_timeout inicial de 5000 ms. Son decisiones de durabilidad que se medirán en las pruebas; no significan garantía absoluta frente a hardware o disco defectuoso.

El mismo hilo posee la conexión y ejecuta transacciones cortas. Mutaciones usan transacción IMMEDIATE cuando cargan y modifican invariantes compartidas; los repositorios reciben la transacción y no hacen commits propios. Una actualización con expectedRevision comprueba y aumenta revision en la misma transacción; cero filas actualizadas produce conflicto, no éxito.

La documentación de [SQLite WAL](https://www.sqlite.org/wal.html) describe sus archivos asociados y límites de funcionamiento. Este diseño restringe la biblioteca activa a disco local. No se comparte una conexión concurrentemente entre tareas.

| Operación | Escrituras que se confirman juntas |
|---|---|
| confirmImport | Paper, autores ordenados, documento, intención COMMITTED, recibo y auditoría |
| Captura tipada | Item y detalle especializado, pertenencia al paper, conceptos, localizador y enlace inicial, auditoría y revisiones afectadas |
| Guardar respuesta | Respuesta, revisión de procesamiento, reevaluación de invalidaciones relevantes y evento |
| Avanzar fase | Snapshot de gate actual, estado de fase, fase activa, historial y revisión; decisión P1 archive incluye archive de Paper en ese mismo commit |
| Relación | Extremos y tipos comprobados, relación/procedencia inicial y auditoría |
| Archive/restore | Estado previo y nuevo, revisión, invalidaciones dependientes y auditoría |

No existe una transacción común real entre filesystem y SQLite. Las operaciones que afectan ambos conservan una intención verificable y recuperación explícita.

Prueba de acceso documental/handle y guardas fuera de la TX; dentro se revalida referencia Paper/Document y concordancia con prueba. No leer/hashear PDFs de hasta 500 MiB bajo DbActor/TX. Inaccesibilidad externa cambia snapshot disponible pero no fabrica revisión; cambios DB Document sí invalidan/renuevan en UoW. En habilitación/completitud revalidar cadena COMPLETED+snapshot aceptado+gate vigente. ABI tras T03, ver WORKFLOW_GATES.

## 4. Idempotencia, historial y límites de mutación

confirmImport vuelve a entregar su PaperDto si el mismo token ya se confirmó, sin crear otra entidad; una petición con contenido incompatible devuelve conflicto. operation_receipts guarda request_id UUID, comando, hash canónico de petición, referencias de resultado y fecha para los comandos idempotentes definidos en CONTRACTS. La recepción repetida de mismo ID y hash recupera el resultado; mismo ID con otro contenido se rechaza.

Los recibos no se purgan automáticamente en v0.1. Las mutaciones de edición siguen necesitando expectedRevision; requestId no sustituye la protección contra textos obsoletos. Antes de comunicar un éxito se confirma la transacción, incluido el recibo.

audit_events registra acción, identidad afectada y cambios necesarios para explicar el historial. El historial de contenido sensible permanece dentro de la biblioteca y sus backups; no se copia a logs de diagnóstico. No se denomina event sourcing ni se promete undo general. Merge y purga definitiva no se exponen en v0.1.

maintenance_operations conserva operationId, tipo, estado, progreso si es conocido, resultado/error y manifiesto de recursos propios. Para activar una raíz restaurada o cambiar biblioteca, un journal durable de coordinación fuera de la raíz que se está cerrando permite resolver el cambio incluso si la DB anterior ya no está abierta. No se serializan secretos ni contenidos de papers en ese journal.

## 5. Raíces y archivos administrados

```text
%LOCALAPPDATA%/ResearchWorkbench/
  library/
    library.json             # libraryId, formato y compatibilidad
    research.sqlite
    documents/<documentId>/original.pdf
    staging/<importOperationId>/source.pdf
    recovery/                # manifiestos de operación cuando correspondan
  backups/<backupId>/         # protección interna y copias elegidas locales
  logs/                      # diagnóstico rotado sin contenido de investigación
```

La raíz real se resuelve con APIs OS y preferencias; ninguna operación se basa en cwd. Los metadatos SQLite y library.json deben coincidir en libraryId; el manifiesto no es una segunda autoridad para conocimiento. Una biblioteca abierta conserva su identidad al moverla.

DocumentStore comprueba ruta relativa, raíz canónica y reparse points/junctions antes de abrir. El protocolo del lector consulta documentId; no recibe una ruta. Un archivo cambiado externamente marca sus anclajes STALE y provoca diagnóstico de integridad; no reescribe automáticamente el hash capturado ni finge que la procedencia sigue verificada.

El primer piloto gestiona un documento vigente por paper. La estructura admite versiones sin que eso prometa una interfaz de reemplazo de PDF. Los recursos binarios se retienen mientras existan anclajes o una operación recuperable que los use.

## 6. Importación y recuperación

1. Selector nativo concede una operación para un archivo elegido. Se registra intención STAGING y se copia a staging; cancelación retira solo archivos propiedad de ese intento.
2. Se valida formato mínimo, tamaño y legibilidad, se calcula hash y se ofrece decisión ante DOI/hash duplicado. No se infiere autor o año si faltan.
3. Antes de promover se persisten documento/paper UUID reservados, metadatos confirmados, destino relativo y hash. Las rutas staging y destino están en el mismo volumen.
4. Se promueve el archivo y se marca el intento PROMOTED. Después la transacción inserta las entidades y deja COMMITTED.
5. Si se interrumpe entre pasos, el arranque coteja intención, destino y hash. Coincidencia inequívoca permite terminar; un caso ambiguo queda pendiente, nunca se elimina un archivo de propietario desconocido.

Un COMMITTED sin archivo legible es una incidencia de integridad, no un motivo para eliminar la ficha. Un staging incompleto sin destino puede cancelarse con registro de resultado. Las pruebas inyectan interrupción antes y después de cada frontera durable.

Los archivos internos `.rw-directory-pin` pueden permanecer en raíz/directorios administrados conforme a [WINDOWS_DIRECTORY_GUARDS](../development/WINDOWS_DIRECTORY_GUARDS.md), ADR-016. No son documentos ni conocimiento; cleanup no los elimina y export/backup los omiten de sus manifiestos. Restore de formato soportado los regenera al adquirir la raíz/directorios, sin confiar en pins aportados como autorización. La biblioteca v0.1 usa volumen local NTFS.

## 7. Migraciones y compatibility

| Secuencia prevista | Contenido |
|---|---|
| 0001 piloto | Fuentes, documentos, imports, lectura, revisión, settings, recibos, auditoría y ledger |
| 0002 workflow | Definiciones PRE/P1/P2, respuestas, procesamiento y gates versionados |
| 0003 conocimiento | Catálogo core, items, conceptos, asociaciones, relaciones y procedencia |
| 0004 portabilidad | Proyecciones FTS y metadatos necesarios para export/backup/restore de v0.1 |

Esta es una secuencia de diseño, no SQL ya publicado. Una migración liberada es inmutable y conserva checksum. appVersion, dbSchemaVersion, ipcVersion, ontologyVersion, phaseDefinitionVersion y exportSchemaVersion son versiones independientes.

Antes de migrar: bloquear escritores, terminar o fijar operaciones de archivos, verificar esquema y checksum y crear snapshot consistente con los archivos necesarios. El backup incluye estado recuperable de imports. Se valida foreign_key_check y quick_check; un fallo impide continuar. La migración se ejecuta en transacción; el ledger se escribe en ese mismo commit. Un fallo revierte DB y mantiene el backup, sin restaurarlo automáticamente ni iniciar una UI editable sobre esquema parcial.

Si dbSchemaVersion es mayor que la soportada, se abre solo una pantalla de diagnóstico; no se modifica la biblioteca ni se ejecuta una migración descendente. El instalador limita downgrades, pero el backend conserva esta defensa por separado.

## 8. Snapshot, exportación y backup

SnapshotService detiene nuevas escrituras y obtiene un estado consistente de DB y referencias documentales; copia los recursos inmutables necesarios mientras conserva el permiso de mantenimiento. Para DB se utiliza la [API de backup de SQLite](https://www.sqlite.org/backup.html), no una copia aislada del archivo con WAL activo. El diseño garantiza consistencia del conjunto imponiendo además la exclusión de mutaciones de la aplicación.

ExportLibrary crea JSONL documentado y Markdown derivado con IDs, relaciones, procedencia y versiones. ExportPaper calcula cierre de referencias: incluye entidades compartidas y extremos requeridos para no emitir IDs sin definición. La omisión de PDFs en una exportación de texto se declara en el manifiesto; no se confunde con un backup restaurable.

Un backup restaurable contiene DB de snapshot, PDFs requeridos, library.json y manifest.json con backupId, libraryId, versiones, fecha, lista de archivos relativos, tamaños y hashes. El manifiesto es el contrato de verificación; los logs no son necesarios para recuperar investigación.

Política inicial propuesta: protección obligatoria antes de migración y backup manual desde v0.1; recordatorio tras siete días de uso sin backup exitoso, sin tarea residente ni sincronización. Conservar todas las copias por defecto; cualquier eliminación futura debe ser explícita. Un backup junto a la biblioteca no protege frente a pérdida del disco: la UI permite elegir otro destino local.

## 9. Restauración y cambio de biblioteca

Restore verifica manifiesto, formato soportado, hashes, referencias, esquema y espacio **antes** de activar datos. Extrae o copia a una raíz nueva junto a la biblioteca, rechaza rutas absolutas, traversal y enlaces que escapen. El formato inicial de backup es directorio; empaquetarlo como archivo comprimido requerirá especificar y probar la extracción segura.

Se prepara la raíz y se verifica con SQLite; restore devuelve un token de biblioteca preparada, sin cambiar la activa. Tras una elección explícita, switchLibrary registra intención durable de cambio, cierra la conexión, cambia la raíz activa mediante configuración recuperable y vuelve a abrir. La anterior permanece disponible para rollback explícito; nunca se sobrescribe una DB abierta. Un crash en cada paso se resuelve en el siguiente inicio con el manifiesto, identificando cuál de las raíces está completa.

SwitchLibrary utiliza el mismo coordinador: bloquea operaciones, guarda/cancela pendientes y verifica libraryId/compatibilidad/bloqueo de destino. No fusiona bibliotecas, no cambia IDs y no mueve datos por accidente. Elegir otra raíz requiere selector nativo, no una ruta arbitraria enviada por UI.

## 10. Verificación de persistencia previa a entrega

- Restricciones y transacciones: FKs, revisiones, duplicados, rollback e historial coherente.
- Reapertura real: misma identidad, contenido, última página y procedencia después de reiniciar.
- Filesystem: Unicode, espacios, escapes, disco lleno, PDF corrupto y hash cambiado.
- Fault injection: cortes en promoción/import, commit, snapshot, migración y activación de raíz restaurada.
- Portabilidad: export sin referencias rotas; backup verificable; restore completo con PDFs en biblioteca temporal.
- Ciclo de instalación: upgrade, rechazo incompatible, uninstall y reinstall conservando datos.

Estas son pruebas previstas; los resultados se registrarán cuando exista implementación. La aceptación se rige por QUALITY y las SPECs, no por la presencia de estas tablas en un documento.


## 0003: manifiesto consumidor aceptado (ADR-022)

[TASK05_PORTS](../plans/TASK05_PORTS.md) fija columnas, autoridad de atributos, claves/índices, catálogo core1.0.0 y mapping consumidor antes de implementar. Concept almacena definición sólo en body_text y preferred_name/title como mirror atómico; attrs de Concept/Evidence/Question se reconstruyen de tablas específicas, attributes_json=NULL. Otros nueve tipos usan objeto discriminado completo validado. No autoridades duplicadas ni defaults ante corrupción.

Padre/Provenance inicial revision0. paper_items nuevos phase_code=P2, selected_for_p3=0, priority/rationale NULL. Concept comparte revision con knowledge_items. UNIQUE activo de Relations por extremos/tipo/contexto canónico; colisión create/update/restore Conflict sin efectos. validations(id) es reserva vacía, sin filas ni servicios simulados. No ON DELETE CASCADE ni triggers destructivos. 0001/0002/checksums/runner intactos; DDL0003 y backup/upgrade/reopen se revisarán en T05a, FTS0004 posterior.

Auditoría efectiva lleva requestId, entidad, acción cerrada y before/after pertinente, dentro de la TX de datos/receipt/clocks. El evento genérico legacy de receipt no sustituye historial de contenido. Alcance P2 e inversa siguen exactamente el cierre de WORKFLOW_GATES; no-op no inventa modificaciones.
