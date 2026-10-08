# Research Workbench
Arquitectura y especificación del producto v0.1
1 de octubre de 2026

Research Workbench es una aplicación de escritorio local para convertir la lectura guiada de literatura científica en conocimiento estructurado, trazable y organizado por conceptos. Está dirigida a un único investigador y debe permitir trabajar desde Windows sin cuentas, servidor ni conexión obligatoria.

La primera versión debe permitir importar un survey real y procesarlo manualmente desde PRE hasta P2. Las fases de validación de evidencia y reconstrucción, el motor de reglas y la asistencia con modelos de lenguaje se incorporan después, cuando el flujo manual y el almacenamiento hayan demostrado utilidad.

La arquitectura propuesta combina Tauri 2, React y TypeScript para la experiencia de escritorio, Rust para los servicios locales y SQLite como fuente de verdad. Los PDF se conservan en una biblioteca administrada. Markdown y JSONL son formatos de salida, no almacenes canónicos paralelos.

## 1 Visión y propósito
El propósito es ayudar al investigador a entender qué afirma la literatura, cómo organiza un campo, qué evidencia sostiene sus afirmaciones y qué incertidumbres permanecen. El resultado debe servir tanto para retomar una lectura como para construir una base de conocimiento que sobreviva al documento que la originó.

Paper es la unidad de trabajo. Concept es la unidad de organización del conocimiento. KnowledgeItem es la unidad semántica capturada. Evidence y Provenance permiten rastrear el origen y el apoyo de una afirmación. Relation representa la semántica entre elementos. Workflow aporta disciplina al procesamiento.

Una vista por paper conserva el contexto de lectura; una vista por concepto reúne aportaciones de varios papers. Ambas consultan las mismas entidades. El sistema no debe duplicar un concepto únicamente porque aparezca en una fuente diferente.

El horizonte es una base de investigación capaz de mostrar contradicciones, condiciones de aplicabilidad, preguntas abiertas y fronteras del conocimiento registrado. Una frontera dentro de la biblioteca personal no equivale automáticamente a una carencia demostrada de toda la literatura.

## 2 Problema y necesidades
Las notas libres pueden conservar texto sin distinguir una afirmación de su evidencia, una interpretación de una cita o una pregunta de una conclusión. Además, las notas organizadas únicamente por paper dificultan reunir lo aprendido sobre un concepto en diferentes fuentes.

La aplicación debe reducir esa ambigüedad mediante captura tipada y trazabilidad. También debe ofrecer continuidad: recuperar la página del PDF, el punto de procesamiento, las preguntas pendientes y el estado de guardado después de cerrar el programa.

El usuario necesita una interfaz de escritorio convencional, agradable y moderna. Las operaciones habituales deben ser comprensibles sin conocer bases de datos, ontologías o programación. La complejidad técnica pertenece al diseño interno; los formularios deben hablar de conceptos, afirmaciones, fuentes y preguntas.

## 3 Objetivos y resultados
- Capturar papers, metadatos, respuestas y conocimiento sin conexión.
- Guiar la lectura mediante objetivos, preguntas, acciones permitidas y límites de cada fase.
- Organizar conceptos globales y enlazarles afirmaciones, evidencias, preguntas y relaciones.
- Distinguir lo que expresa una fuente de lo que interpreta o propone el investigador.
- Conservar datos de forma persistente con autosave, integridad y recuperación verificable.
- Exportar entidades y relaciones con identificadores estables y formatos documentados.
- Preparar una integración futura con LLMs sin necesitarla para que la aplicación sea útil.

La prueba de valor de v0.1 es procesar un survey real de PRE a P2, cerrar y reabrir la aplicación, consultar sus conceptos globales y exportar sus datos conservando las relaciones y la procedencia.

## 4 Alcance de la primera versión
La entrega v0.1 incluye shell de escritorio, biblioteca, importación y copia de PDF, metadatos manuales, CRUD de papers, workspace de lectura, PRE, P1, P2, gates de completitud, captura tipada, conceptos globales, afirmaciones, preguntas, relaciones y procedencia básica. Incluye también búsqueda básica, autosave, exportación mínima completa y backup restaurable.

La evidencia puede capturarse desde v0.1 como fragmento o resultado identificado y enlazado a una afirmación. El workspace completo de evaluación crítica y confianza pertenece a v0.2. El navegador global básico de conceptos existe desde v0.1; su comparación entre fuentes y sus vistas avanzadas pueden ampliarse en v0.3.

### 4.1 Requisitos imprescindibles
- La biblioteca funciona localmente y no necesita login ni servicio remoto.
- El PDF importado se copia a la biblioteca y puede abrirse aunque se mueva el original.
- Toda entidad tiene un UUID estable; los nombres pueden cambiar sin romper enlaces.
- El backend vuelve a evaluar los gates antes de avanzar una fase.
- La captura desde literatura conserva fuente y localizador o marca explícitamente lo pendiente.
- Las hipótesis e interpretaciones propias tienen un origen identificado.
- El guardado confirmado corresponde a una transacción completada, no a un cambio visual.
- El cierre, la restauración y las migraciones protegen el conocimiento ya registrado.
- El export incluye entidades, relaciones, procedencia y versiones del esquema.

### 4.2 Límites y cosas que no construir
No se incluyen usuarios, roles, login, colaboración, sincronización cloud, API externa ni microservicios. Tampoco se incluyen LLMs, agentes, RAG, embeddings, bases vectoriales, MCP ni análisis automático de papers.

No se construye un gestor bibliográfico completo ni integración con Zotero en v0.1. La extracción automática de metadatos, OCR y un editor de ontología extenso quedan fuera. El grafo visual es opcional en una versión posterior; las fichas y listas deben bastar para trabajar.

No se fuerza una conclusión cierta para superar un gate. No se presenta como evidencia un simple DOI, una coincidencia de texto o una sugerencia del sistema. No se borra conocimiento compartido al eliminar una fuente sin revisar sus dependencias.

## 5 Principios del producto y del conocimiento
Local primero. La biblioteca debe ser útil de manera autónoma. La eventual conectividad se añade alrededor de un núcleo que ya funciona sin red.

Un solo almacén canónico. SQLite conserva el estado estructurado; los archivos mantienen los documentos y recursos binarios. Índices, proyecciones de grafo y exportaciones pueden reconstruirse y no tienen autoridad sobre la base de datos.

Conceptos globales y fuentes identificadas. La captura se realiza desde un paper, pero un concepto puede aparecer en muchos papers. Se sugiere reutilizar conceptos existentes sin fusionar automáticamente términos que podrían representar sentidos distintos.

Trazabilidad antes que volumen. Es preferible una afirmación bien delimitada y localizable a cientos de elementos sin contexto. La evidencia conserva condiciones, límites e interpretación, no únicamente una puntuación de confianza.

Completitud y certeza son dimensiones distintas. Una respuesta desconocida, acompañada de una explicación, puede estar procesada. Un campo vacío no lo está. Una fase completada no transforma todas sus afirmaciones en conocimiento validado.

El investigador valida. Las reglas futuras pueden señalar elementos que merecen revisión. La aceptación de una conclusión y cualquier cambio de confianza permanecen explícitos y auditables.

Portabilidad y evolución gradual. La aplicación debe tener exportación documentada, backups íntegros y migraciones versionadas. Cada ampliación se justifica por el trabajo real de lectura, evitando infraestructura prematura.

## 6 Casos de uso principales
### 6.1 Importar y preparar una lectura
El investigador elige un PDF, revisa título, autores, año, venue, DOI y tipo de revisión, y confirma la importación. El sistema calcula un hash, busca posibles duplicados y copia el archivo a la biblioteca. PRE recoge propósito, incertidumbre, baseline y resultado esperado.

### 6.2 Continuar un paper
La pantalla inicial muestra las lecturas activas y su última actividad. Al continuar, abre el paper en la fase, página y contexto anteriores. Los requisitos pendientes aparecen con una explicación concreta.

### 6.3 Capturar conocimiento desde el PDF
El investigador selecciona un fragmento o introduce manualmente una ubicación, elige el tipo de objeto y redacta su contenido. Puede reutilizar conceptos existentes, vincular el objeto a las preguntas de lectura y registrar relaciones. La aplicación conserva el texto citado separado de la interpretación.

### 6.4 Consultar un concepto entre fuentes
Al abrir un concepto se muestran definición, alias, afirmaciones, evidencia, preguntas, relaciones y fuentes. Cada afirmación mantiene su alcance. Las afirmaciones incompatibles no se sustituyen silenciosamente por una síntesis.

### 6.5 Validar y reconstruir en versiones posteriores
En P3 se trabaja sobre una cola seleccionada por relevancia, no sobre todo lo capturado. En P4 se redacta una reconstrucción con fuentes ocultas, después se compara con notas y PDF y se registran correcciones y preguntas persistentes.

## 7 Arquitectura de información e interfaz
La navegación principal contiene Inicio, Biblioteca y Conocimiento. Configuración reúne ubicación de biblioteca, exportación, backups y preferencias. Conocimiento ofrece conceptos, afirmaciones, preguntas y gaps; las fuentes pueden consultarse por paper, autor o venue.

Inicio muestra un acceso para continuar la última lectura, papers recientes y una acción clara para importar. Los contadores describen el contenido existente, sin sugerir que el número de notas mide comprensión.

Biblioteca ofrece búsqueda y filtros por estado, fase, año, tipo y dominio. Cada fila permite abrir, editar o archivar. Debe existir una vista para elementos archivados y una recuperación explícita de operaciones reversibles.

El workspace presenta identidad del paper, fase actual, progreso de requisitos y guardado. El objetivo, preguntas clave, sí hacemos, no hacemos y consideraciones permanecen accesibles mientras se trabaja.

En P2 la vista usa dos paneles ajustables: PDF y conocimiento. El panel de conocimiento alterna fichas, preguntas y relaciones. Se puede crear un objeto sin selección de texto, indicando manualmente página o sección.

Los símbolos siempre van acompañados de etiquetas. El estado no depende solamente del color. Se requieren navegación por teclado, foco visible, tamaños legibles y controles accesibles. El autosave muestra Guardando, Guardado o Error al guardar con opciones de recuperación.

Los porcentajes se calculan sobre requisitos procesados de una fase. Se muestra también el número de requisitos completos sobre el total. La aplicación no llama Completado a un paper procesado únicamente hasta P2 si P3 y P4 siguen pendientes.

## 8 Workflow y contratos de fase
El flujo general es PRE → P1 → P2 → P3 → P4 → Procesado. Las fases PRE, P1 y P2 son operativas en v0.1. P3 y P4 se conservan en el modelo como fases previstas y se habilitan al implementar sus workspaces.

Cada PhaseDefinition contiene código, nombre, objetivo, preguntas clave, sí hacemos, no hacemos, consideraciones, salidas requeridas y reglas de completitud. Su versión se fija en cada procesamiento para que una actualización de plantilla no cambie retrospectivamente lo que significó completar la fase.

Un gate verifica presencia, estado de procesamiento y consistencia mínima de las salidas. No juzga automáticamente la verdad científica. La interfaz explica cada fallo y el servicio de dominio repite la comprobación dentro de la operación de avance.

### 8.1 PRE y contexto previo
Objetivo: definir por qué se lee el paper y qué incertidumbre se quiere reducir. Se registra un baseline antes de la lectura, expresando lo que el investigador cree o conoce actualmente.

Preguntas: ¿por qué leo esta fuente?, ¿qué decisión, hipótesis o experimento podría cambiar?, ¿qué sé ahora?, ¿qué espero saber al terminar? y ¿qué profundidad necesito?

Sí hacemos: identificar el documento, formular el propósito y dejar por escrito el modelo previo. No hacemos: evaluar todavía toda la evidencia ni anticipar conclusiones a partir del título. Consideración: un DOI o metadato desconocido no impide necesariamente leer, pero debe quedar identificado como desconocido.

Gate: PDF disponible, título, tipo de revisión procesado, propósito, incertidumbre objetivo, baseline y resultado esperado. La salida es una ficha bibliográfica y un contrato de lectura que pueda revisarse después.

### 8.2 P1 y orientación
Objetivo: reconocer alcance, estructura y utilidad de la fuente para el propósito definido. La lectura inicial recorre abstract, introducción, encabezados, figuras, tablas, conclusión y referencias.

Campos obligatorios: scope, out_of_scope, review_type, literature_cutoff, core_message y relevance_decision. Se añaden organización del campo, cobertura y metodología de selección cuando resulten pertinentes. Si la fecha de corte o metodología no aparecen, se registra unknown con una explicación.

Sí hacemos: entender la taxonomía propuesta, los límites y la metodología declarada. No hacemos: interpretar una comparación resumida como superioridad general ni validar todas las referencias. Consideración: la valoración de calidad depende del propósito de lectura.

Gate: respuestas obligatorias procesadas, valoración suficiente / usar con cautela / débil para mi propósito y decisión continuar / lectura ligera / archivar. Solo continuar habilita P2. Archivar conserva metadatos y respuestas, y no equivale a procesar todas las fases.

### 8.3 P2 y modelo conceptual
Objetivo: construir una representación del campo mediante conceptos, afirmaciones, condiciones, preguntas y relaciones, conectada con las preguntas de PRE.

Sí hacemos: capturar ideas relevantes, identificar supuestos, reunir conceptos y localizar afirmaciones. No hacemos: transcribir todo el paper, perseguir indiscriminadamente todas las referencias o declarar validada una afirmación por haberla encontrado en un survey.

La barra de captura ofrece Concepto, Afirmación, Pregunta, Evidencia, Supuesto, Condición, Limitación, Ejemplo y Referencia. Se reutilizan entidades existentes y se registran conexiones tipadas entre objetos.

Gate: preguntas principales procesadas, síntesis o mapa del campo creado, conceptos relevantes revisados, relaciones significativas documentadas, contradicciones revisadas, referencias clasificadas y candidatos P3 seleccionados o ausencia justificada. No se exige inventar contradicciones ni un número arbitrario de conceptos o relaciones.

La salida es un modelo conceptual trazable y una cola de validación priorizada. La completitud de P2 no constituye validación epistemológica. En v0.1 la cola se puede guardar aunque el workspace P3 no esté habilitado.

### 8.4 P3 y validación de evidencias
Objetivo: evaluar las afirmaciones y preguntas seleccionadas por su efecto sobre decisiones, hipótesis o comprensión. Flujo: afirmación, evidencia, calidad, comparabilidad, condiciones, contradicciones, aplicabilidad y confianza.

Para comparabilidad se revisan dataset, métrica, split, preprocesado y baselines. Cada dimensión permite comparable, parcial, desconocido o no comparable, con explicación. No se sustituye esta evaluación por un score arbitrario.

Sí hacemos: buscar fuentes primarias pertinentes, registrar pasajes y resultados, examinar condiciones y expresar incertidumbre. No hacemos: asumir consenso a partir de varias citas que repiten una misma fuente ni generalizar un resultado fuera de sus condiciones.

Gate: cada candidato tiene decisión, fuentes inspeccionadas o ausencia justificada, comparabilidad procesada, condiciones, límites y conclusión de confianza. Desconocido o validación pendiente pueden cerrar una evaluación si se documenta por qué y qué queda abierto.

### 8.5 P4 y reconstrucción
Objetivo: comprobar qué modelo se ha integrado y qué cambió respecto al baseline. La etapa inicial oculta PDF y notas fuente, y conserva una reconstrucción independiente.

Se responde cómo se organiza ahora el campo, cuáles son sus familias, relaciones, trade-offs y supuestos, qué cambió y qué incertidumbres quedan. Después se habilita Compare with source para revisar omisiones, relaciones erróneas y preguntas sin resolver.

Sí hacemos: recordar, comparar, detectar gaps y reparar. No hacemos: copiar una síntesis del paper y llamarla reconstrucción. Gate: reconstrucción guardada antes de comparar, diferencias procesadas, reparaciones registradas y preguntas persistentes identificadas.

### 8.6 Revisiones y retrocesos
Una fase completada sigue siendo editable. Los cambios registran historial y revisión. Si se elimina una salida obligatoria, la fase deja de satisfacer su gate; las fases posteriores se marcan para revisión y sus datos se conservan. Volver atrás no borra trabajo.

El estado editorial NEW, ACTIVE, COMPLETED, ARCHIVED o TRASHED se separa de la fase actual. Las etiquetas visibles PRE_IN_PROGRESS, P1_IN_PROGRESS y equivalentes se derivan del estado y de los registros de fase para evitar dos fuentes contradictorias de progreso.

## 9 Objetos de conocimiento y simbología
KnowledgeItem tiene identidad, tipo, título, cuerpo, origen, estado, confianza y fechas. El contenido puede ser breve y atómico. El cuerpo normalizado en texto permite buscar y exportar; el contenido enriquecido, si existe, se conserva con formato y versión conocidos.

| Objeto | Función | Datos específicos |
|---|---|---|
| Concept | Organizar significado compartido | Nombre preferido, definición, alias, dominio |
| Claim | Expresar una afirmación delimitada | Enunciado, alcance, condiciones, confianza |
| Evidence | Conservar apoyo o resultado inspeccionado | Fragmento o resultado, fuente, localizador, límites |
| Question | Registrar una incertidumbre | Pregunta, estado, respuesta o siguiente acción |
| Gap | Identificar una carencia delimitada | Alcance, justificación, búsqueda pendiente |
| Assumption | Explicitar un supuesto | Enunciado y contexto |
| Condition | Delimitar aplicabilidad | Condición y ámbito |
| Limitation | Registrar una restricción | Restricción y efecto |
| Method | Describir un procedimiento | Nombre, familia y contexto |
| Example | Ilustrar un concepto | Descripción y fuente cuando corresponda |
| Insight | Registrar un cambio del modelo propio | Baseline, interpretación y conceptos afectados |
| Reference | Marcar una fuente a perseguir | Identificador o referencia, motivo, estado |

Dataset, Metric, Task y System se incorporan cuando la captura real lo requiera. Los tipos del catálogo representan una capacidad del modelo, no una obligación de habilitar todos los formularios desde el primer día.

| Símbolo | Acción o significado | Persistencia |
|---|---|---|
| Δ | Cambio en mi modelo | Insight con origen del investigador |
| ? | Pregunta | Question |
| ≠ | Contradicción por revisar | Relación contradicts con contexto |
| ⚠ | Cautela | Limitation o estado requires_validation |
| Ex | Ejemplo | Example |
| Ref | Referencia a perseguir | Reference |
| → | Relacionar elementos | Relation |
| C y E | Afirmación y evidencia | Claim y Evidence |

Los símbolos son accesos de interfaz; no reemplazan tipos y relaciones persistentes. Una cautela debe indicar su objeto. Una contradicción no se limita a un icono: conecta afirmaciones y conserva el contexto que las hace incompatibles.

## 10 Ontología y relaciones
La ontología core define tipos de objeto y tipos de relación. Los domain packs pueden añadir vocabulario especializado, por ejemplo sensor, régimen de operación, drift, fallo, ataque, latencia de detección o variable SCADA. Estos términos no deben confundirse automáticamente entre sí.

| Relación | Significado | Ejemplo de uso |
|---|---|---|
| supports | Apoya dentro de un contexto | Evidence → Claim |
| contradicts | Registra incompatibilidad | Claim → Claim |
| extends | Amplía un objeto o propuesta | Method → Method |
| causes | Afirmación causal explícita | Claim → Claim o Concept |
| requires | Exige una condición | Method → Condition |
| depends_on | Depende de un componente | Method → Concept |
| works_when | Delimita condiciones favorables | Method → Condition |
| fails_when | Delimita condiciones de fallo | Method → Condition |
| compares_with | Registra una comparación | Method → Method |
| part_of | Expresa pertenencia | Concept → Concept |
| similar_to | Registra semejanza delimitada | Concept → Concept |
| limits | Restringe alcance o uso | Limitation → Claim o Method |
| improves | Afirma mejora condicionada | Method → Method |

Una relación causal requiere un origen y una justificación adecuados; seleccionar causes no demuestra causalidad. Supports y contradicts se interpretan junto con condiciones y procedencia. Una evidencia no es un concepto, y compartir concepto no implica apoyo entre afirmaciones.

Los packs se distribuyen en YAML o JSON versionado. Al activar un pack se valida su estructura y se materializa el catálogo correspondiente en SQLite. No se ejecuta código incluido en un pack. El editor genérico de ontología se pospone; inicialmente se cargan catálogos conocidos.

Las relaciones simétricas normalizan sus extremos para detectar duplicados; las dirigidas conservan su orden. La validación comprueba existencia de extremos, combinaciones permitidas, autoenlaces y contexto. Cualquier regla específica del dominio debe ser visible y versionada.

## 11 Procedencia y confianza
Provenance identifica paper, documento concreto, hash del documento, página física del PDF, etiqueta de página si difiere, sección, cita o snippet y ubicación opcional. El número de página físico se cuenta desde uno; no se confunde con la numeración impresa.

La cita literal se mantiene separada del cuerpo interpretativo. El hash permite advertir que el PDF cambió y que un anclaje podría estar desactualizado. Se conserva el documento anterior cuando una versión nueva lo sustituye si aún contiene anclajes utilizados.

Origin distingue literature, researcher_interpretation y researcher_hypothesis. Una definición aportada por el investigador no necesita inventar una cita, pero debe quedar señalada. Una captura desde literatura puede guardarse como borrador con localizador pendiente; no se presenta como trazada hasta completarlo.

| Estado de confianza | Etiqueta visible | Uso |
|---|---|---|
| sufficiently_supported | Suficientemente apoyado | Apoyo evaluado dentro de condiciones explícitas |
| context_dependent | Dependiente del contexto | Resultado condicionado o heterogéneo |
| uncertain | Incierto | No permite una conclusión suficiente |
| requires_validation | Requiere validación | Capturado y todavía pendiente de evaluación |

El estado de evidencia y el de la afirmación se mantienen por separado. La confianza de una afirmación no se calcula por contar citas. Una evaluación registra qué se revisó, condiciones, comparabilidad, límites, conclusión y motivo de la decisión humana.

## 12 Modelo relacional
El diseño usa un supertipo knowledge_items y extensiones para conceptos, evidencias y preguntas. Permite añadir formularios específicos sin introducir desde el principio una tabla completamente independiente para cada tipo. Los atributos comunes no se esconden en JSON.

Paper se relaciona con autores mediante una tabla ordenada y opcionalmente con un venue. Puede tener varios documentos, aunque v0.1 muestre principalmente uno. Los items se vinculan a uno o varios papers mediante asociaciones; esa pertenencia no reemplaza la procedencia.

Los conceptos son knowledge_items globales con atributos especializados. item_concepts relaciona cualquier item con conceptos. relations conecta dos knowledge_items; procedencia de items y relaciones usa tablas de enlace diferentes para conservar claves foráneas verificables.

| Grupo | Tablas | Responsabilidad |
|---|---|---|
| Fuentes | papers, authors, paper_authors, venues, documents | Identidad y archivos |
| Workflow | phase_definitions, paper_phases, phase_answers | Versiones, respuestas y progreso |
| Conocimiento | knowledge_items, concepts, concept_aliases, evidence_details, question_details | Objetos tipados |
| Asociación | paper_items, item_concepts, relations | Contexto y conexiones |
| Trazabilidad | provenance, item_provenance, relation_provenance | Localizadores y citas |
| Evaluación | validations | Revisión humana |
| Operación | ontology_types, relation_types, audit_events, schema_migrations | Catálogos, historial y evolución |

Todas las fechas se almacenan en UTC en ISO 8601 y se muestran en la zona local. Se usan UUID en texto, claves foráneas activas y transacciones para operaciones que modifican varias entidades.

## 13 Esquema SQLite propuesto
El siguiente DDL define un núcleo implementable para v0.1 y deja espacio para las fases posteriores. Las comprobaciones semánticas entre tipos, los gates y los localizadores obligatorios se implementan también en el servicio de dominio. Las tablas de catálogo se inicializan antes de crear objetos.

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL,
  checksum TEXT NOT NULL
);
CREATE TABLE ontology_types (
  code TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  pack_id TEXT NOT NULL,
  pack_version TEXT NOT NULL,
  attributes_schema_json TEXT NOT NULL DEFAULT '{}'
);
CREATE TABLE relation_types (
  code TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  symmetric INTEGER NOT NULL DEFAULT 0
    CHECK (symmetric IN (0,1)),
  endpoint_rules_json TEXT NOT NULL DEFAULT '{}'
);
CREATE TABLE venues (
  id TEXT PRIMARY KEY, name TEXT NOT NULL,
  kind TEXT, identifier TEXT
);
CREATE TABLE authors (
  id TEXT PRIMARY KEY, display_name TEXT NOT NULL,
  orcid TEXT UNIQUE
);
CREATE TABLE papers (
  id TEXT PRIMARY KEY, title TEXT NOT NULL,
  doi TEXT, year INTEGER, review_type TEXT NOT NULL,
  domain TEXT, url TEXT,
  venue_id TEXT REFERENCES venues(id),
  lifecycle TEXT NOT NULL DEFAULT 'NEW'
    CHECK (lifecycle IN
      ('NEW','ACTIVE','COMPLETED','ARCHIVED','TRASHED')),
  last_activity_at TEXT,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE UNIQUE INDEX papers_doi_unique
  ON papers(doi) WHERE doi IS NOT NULL;
CREATE TABLE paper_authors (
  paper_id TEXT NOT NULL REFERENCES papers(id),
  author_id TEXT NOT NULL REFERENCES authors(id),
  position INTEGER NOT NULL CHECK (position >= 1),
  PRIMARY KEY (paper_id,author_id),
  UNIQUE (paper_id,position)
);
CREATE TABLE documents (
  id TEXT PRIMARY KEY,
  paper_id TEXT NOT NULL REFERENCES papers(id),
  relative_path TEXT NOT NULL UNIQUE,
  original_filename TEXT NOT NULL,
  original_path TEXT, sha256 TEXT NOT NULL,
  imported_at TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'ACTIVE'
    CHECK (status IN ('ACTIVE','SUPERSEDED','MISSING'))
);
CREATE INDEX documents_hash ON documents(sha256);

CREATE TABLE phase_definitions (
  code TEXT NOT NULL,
  version INTEGER NOT NULL,
  definition_json TEXT NOT NULL,
  PRIMARY KEY (code,version)
);
CREATE TABLE paper_phases (
  paper_id TEXT NOT NULL REFERENCES papers(id),
  phase_code TEXT NOT NULL,
  definition_version INTEGER NOT NULL,
  state TEXT NOT NULL DEFAULT 'NOT_STARTED'
    CHECK (state IN ('NOT_STARTED','IN_PROGRESS',
      'COMPLETED','NEEDS_REVIEW')),
  revision INTEGER NOT NULL DEFAULT 0,
  completed_at TEXT,
  PRIMARY KEY (paper_id,phase_code),
  FOREIGN KEY (phase_code,definition_version)
    REFERENCES phase_definitions(code,version)
);
CREATE TABLE phase_answers (
  paper_id TEXT NOT NULL,
  phase_code TEXT NOT NULL,
  question_key TEXT NOT NULL,
  answer_text TEXT NOT NULL DEFAULT '',
  resolution TEXT NOT NULL DEFAULT 'PENDING'
    CHECK (resolution IN
      ('PENDING','ANSWERED','UNKNOWN','NOT_APPLICABLE')),
  explanation TEXT,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (paper_id,phase_code,question_key),
  FOREIGN KEY (paper_id,phase_code)
    REFERENCES paper_phases(paper_id,phase_code)
);

CREATE TABLE knowledge_items (
  id TEXT PRIMARY KEY,
  type_code TEXT NOT NULL REFERENCES ontology_types(code),
  title TEXT NOT NULL, body_text TEXT NOT NULL DEFAULT '',
  body_json TEXT,
  origin TEXT NOT NULL CHECK (origin IN ('literature',
    'researcher_interpretation','researcher_hypothesis')),
  lifecycle TEXT NOT NULL DEFAULT 'ACTIVE'
    CHECK (lifecycle IN ('ACTIVE','ARCHIVED','TRASHED')),
  confidence TEXT NOT NULL DEFAULT 'requires_validation'
    CHECK (confidence IN ('sufficiently_supported',
      'context_dependent','uncertain','requires_validation')),
  attributes_json TEXT NOT NULL DEFAULT '{}',
  revision INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE INDEX items_type ON knowledge_items(type_code);
CREATE TABLE concepts (
  item_id TEXT PRIMARY KEY REFERENCES knowledge_items(id),
  preferred_name TEXT NOT NULL,
  normalized_name TEXT NOT NULL,
  domain TEXT,
  merged_into_id TEXT REFERENCES concepts(item_id),
  CHECK (merged_into_id IS NULL OR merged_into_id <> item_id)
);
CREATE INDEX concepts_name ON concepts(normalized_name);
CREATE TABLE concept_aliases (
  concept_id TEXT NOT NULL REFERENCES concepts(item_id),
  alias TEXT NOT NULL, normalized_alias TEXT NOT NULL,
  PRIMARY KEY (concept_id,normalized_alias)
);
CREATE TABLE paper_items (
  paper_id TEXT NOT NULL REFERENCES papers(id),
  item_id TEXT NOT NULL REFERENCES knowledge_items(id),
  phase_code TEXT,
  selected_for_p3 INTEGER NOT NULL DEFAULT 0
    CHECK (selected_for_p3 IN (0,1)),
  PRIMARY KEY (paper_id,item_id)
);
CREATE TABLE item_concepts (
  item_id TEXT NOT NULL REFERENCES knowledge_items(id),
  concept_id TEXT NOT NULL REFERENCES concepts(item_id),
  PRIMARY KEY (item_id,concept_id)
);
CREATE TABLE evidence_details (
  item_id TEXT PRIMARY KEY REFERENCES knowledge_items(id),
  evidence_kind TEXT NOT NULL,
  observed_result TEXT,
  conditions_text TEXT,
  limitations_text TEXT
);
CREATE TABLE question_details (
  item_id TEXT PRIMARY KEY REFERENCES knowledge_items(id),
  state TEXT NOT NULL CHECK (state IN
    ('OPEN','ANSWERED','DEFERRED','DISMISSED')),
  answer_text TEXT,
  next_action TEXT
);
CREATE TABLE relations (
  id TEXT PRIMARY KEY,
  source_item_id TEXT NOT NULL REFERENCES knowledge_items(id),
  target_item_id TEXT NOT NULL REFERENCES knowledge_items(id),
  type_code TEXT NOT NULL REFERENCES relation_types(code),
  context_text TEXT NOT NULL DEFAULT '',
  origin TEXT NOT NULL CHECK (origin IN ('literature',
    'researcher_interpretation','researcher_hypothesis')),
  confidence TEXT NOT NULL DEFAULT 'requires_validation'
    CHECK (confidence IN ('sufficiently_supported',
      'context_dependent','uncertain','requires_validation')),
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
  CHECK (source_item_id <> target_item_id),
  UNIQUE (source_item_id,type_code,target_item_id,context_text)
);
CREATE INDEX relations_target ON relations(target_item_id);

CREATE TABLE provenance (
  id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL REFERENCES documents(id),
  page_index INTEGER CHECK (page_index >= 1),
  page_label TEXT, section TEXT, quote_text TEXT,
  locator_json TEXT,
  captured_document_hash TEXT NOT NULL,
  locator_state TEXT NOT NULL DEFAULT 'PENDING'
    CHECK (locator_state IN ('PENDING','LOCATED','STALE')),
  created_at TEXT NOT NULL
);
CREATE TABLE item_provenance (
  item_id TEXT NOT NULL REFERENCES knowledge_items(id),
  provenance_id TEXT NOT NULL REFERENCES provenance(id),
  PRIMARY KEY (item_id,provenance_id)
);
CREATE TABLE relation_provenance (
  relation_id TEXT NOT NULL REFERENCES relations(id),
  provenance_id TEXT NOT NULL REFERENCES provenance(id),
  PRIMARY KEY (relation_id,provenance_id)
);
CREATE TABLE validations (
  id TEXT PRIMARY KEY,
  item_id TEXT REFERENCES knowledge_items(id),
  relation_id TEXT REFERENCES relations(id),
  decision TEXT NOT NULL,
  comparability_json TEXT NOT NULL DEFAULT '{}',
  conditions_text TEXT, limitations_text TEXT,
  rationale TEXT NOT NULL,
  reviewed_at TEXT NOT NULL,
  CHECK ((item_id IS NOT NULL AND relation_id IS NULL)
    OR (item_id IS NULL AND relation_id IS NOT NULL))
);
CREATE TABLE audit_events (
  id TEXT PRIMARY KEY,
  entity_type TEXT NOT NULL, entity_id TEXT NOT NULL,
  action TEXT NOT NULL,
  before_json TEXT, after_json TEXT,
  occurred_at TEXT NOT NULL
);
```

Los nombres de DOI se normalizan al importar y los valores desconocidos se almacenan como NULL. Un posible duplicado por DOI o hash requiere una decisión visible de reutilización o nueva versión; no se fusionan contenidos automáticamente.

El esquema evita borrados en cascada sobre conocimiento compartido. El servicio de eliminación debe resolver asociaciones explícitamente. Los JSON contienen atributos extensibles, pero se validan contra esquemas versionados; no sustituyen columnas, índices y restricciones del núcleo.

## 14 Persistencia de documentos y biblioteca
La biblioteca reside por defecto en una carpeta local del usuario, por ejemplo %LOCALAPPDATA%/ResearchWorkbench/library, o en una ubicación elegida. Las rutas de documentos se guardan relativas a la raíz para facilitar mover la biblioteca.

```text
ResearchWorkbench/
  research.db
  papers/
    <paper_uuid>/
      <document_uuid>/source.pdf
      assets/
  ontology/
    core.yaml
    domains/
  exports/
  backups/
  config/
  staging/
```

La subcarpeta de documento permite conservar distintas versiones de una fuente. La interfaz v0.1 puede mostrar una sola versión activa, pero sus localizadores apuntan siempre a un documento concreto.

La importación usa staging: copiar el PDF a un archivo temporal dentro de la biblioteca, comprobar lectura y hash, registrar intención, mover al destino y confirmar el registro. SQLite y filesystem no comparten una transacción atómica; una recuperación al iniciar reconcilia imports interrumpidos y archivos sin registro.

Se conservan original_filename, original_path, SHA256 e imported_at. La ruta original ayuda al diagnóstico, pero no es necesaria para abrir el paper. Las exportaciones compartibles omiten rutas privadas salvo que el usuario elija incluirlas.

Solo una instancia puede escribir una biblioteca al mismo tiempo en v0.1. La biblioteca activa se mantiene en almacenamiento local; sincronizar archivos de una base abierta o situarla en una unidad de red no forma parte del soporte inicial. El traslado se hace con la aplicación cerrada y verificación posterior.

## 15 Arquitectura técnica y módulos
La aplicación es un monolito local organizado por módulos. React gestiona presentación e interacción. Los comandos internos de Tauri conectan la interfaz con los servicios Rust. Estos servicios aplican reglas del dominio y acceden a SQLite y a la biblioteca de archivos.

```text
Interfaz React y TypeScript
        |
Comandos internos de Tauri
        |
Servicios de aplicación Rust
        |
Dominio y validación
        |
Repositorios SQLite y biblioteca de archivos
```

La comunicación interna no requiere desplegar una API HTTP ni un backend remoto. La interfaz no ejecuta SQL arbitrario ni recibe acceso general al disco. Los comandos aceptan parámetros definidos y devuelven resultados o errores tipados.

| Módulo | Responsabilidad | Operaciones representativas |
|---|---|---|
| LibraryService | Fuentes y archivos | importPaper, updateMetadata, archivePaper |
| WorkflowService | Fases y gates | saveAnswer, evaluateGate, advancePhase |
| KnowledgeService | Items y conceptos | createItem, linkConcept, mergeConcepts |
| RelationService | Semántica entre items | createRelation, validateEndpoints |
| ProvenanceService | Anclajes y citas | attachLocator, checkDocumentHash |
| ValidationService | Evaluaciones posteriores | selectCandidate, recordReview |
| SearchService | Consulta e índices | searchLibrary, searchKnowledge |
| ExportService | Datos portables | exportLibrary, exportPaper |
| BackupService | Snapshot y restauración | createBackup, verifyBackup, restore |

Las reglas del dominio son independientes de los componentes de interfaz. Repositorios y servicios separan transacciones, consultas y reglas. Las operaciones de fusionar, avanzar fase y restaurar no se implementan como una secuencia informal de cambios de frontend.

## 16 Stack propuesto
| Componente | Selección | Uso y límite |
|---|---|---|
| Shell | Tauri 2 | Ejecutable de escritorio e IPC local |
| UI | React y TypeScript | Pantallas, formularios y navegación |
| Componentes | shadcn/ui | Controles accesibles con diseño consistente |
| Formularios | React Hook Form y Zod | Interacción y validación de entrada |
| Núcleo local | Rust | Dominio, archivos, transacciones y servicios |
| Base de datos | SQLite | Estado canónico estructurado |
| Búsqueda | SQLite FTS5 | Índice derivado de texto |
| PDF | PDF.js | Lectura y selección de fragmentos |
| Editor enriquecido | TipTap cuando se necesite | Contenido con salida textual estable |
| Consulta UI | TanStack Query cuando aporte valor | Caché e invalidación del estado persistido |
| Estado transitorio | Estado React o Zustand | Paneles y selección; no segunda base de datos |
| Grafo posterior | Cytoscape.js | Vista derivada opcional |

Las versiones concretas y licencias se fijan durante la implementación y se revisan antes de distribuir el ejecutable. El documento define elecciones de arquitectura; no certifica que el stack haya sido instalado o probado.

## 17 Autosave y consistencia
Los formularios guardan cambios mediante un debounce breve o al confirmar una acción estructurada. El backend devuelve la revisión persistida. El indicador Guardado aparece únicamente tras confirmación de commit.

Los cambios de un mismo objeto se serializan o usan control de revisión para impedir que una respuesta antigua sobrescriba una nueva. Crear un item junto con conceptos y procedencia requiere una transacción. Una relación incompleta no se guarda como definitiva.

Si falla el guardado, el contenido pendiente se mantiene visible y se ofrece reintentar. El cierre espera los cambios pendientes o explica el fallo. El lector conserva página, selección relevante y contexto mediante preferencias separadas del contenido científico.

Se evalúa WAL para lecturas concurrentes, con una política de durabilidad explícita y una conexión de escritura coordinada. El objetivo es no perder un cambio confirmado ante cierre inesperado. Ninguna decisión de rendimiento debe hacer que la interfaz prometa más durabilidad que la configuración real.

## 18 Conceptos y operaciones de mantenimiento
Al escribir un nombre se sugieren conceptos por nombre normalizado y alias. El investigador puede reutilizar uno o crear otro sentido, con definición o dominio que explique la diferencia. Normalizar mayúsculas o acentos ayuda a buscar; no demuestra equivalencia semántica.

Renombrar un concepto conserva UUID, enlaces y procedencia. Archivar lo oculta de las listas activas, mantiene referencias y permite restaurarlo. Eliminar se inicia con una revisión de dependencias y una operación reversible de papelera cuando sea posible.

Fusionar conceptos exige elegir un destino y ver los enlaces afectados. La transacción mueve asociaciones, consolida alias y deduplica relaciones equivalentes sin eliminar contextos diferentes. El origen permanece como entidad fusionada con redirección al destino y evento de auditoría.

Las fusiones deben comprobar ciclos de redirección, autorrelaciones y colisiones. Si la operación no puede preservar significado y trazabilidad, se bloquea con una explicación. Una vista por concepto nunca elimina una afirmación por tener confianza baja; la muestra con su estado.

Al archivar o retirar un paper se conserva el conocimiento global. La eliminación permanente informa qué archivos, localizadores y asociaciones dependen de él. No borra automáticamente conceptos o afirmaciones utilizados por otras fuentes.

## 19 Búsqueda y vistas derivadas
FTS5 indexa título y cuerpo de los knowledge_items y una proyección textual de los metadatos de papers. El índice se actualiza en la misma transacción que modifica el contenido o mediante un mecanismo recuperable claramente definido.

La consulta combina texto con filtros por tipo, concepto, paper, dominio, confianza y estado. Los resultados enlazan al objeto y, cuando corresponde, a su ubicación en el PDF. Los objetos archivados y de papelera quedan fuera por defecto con opción explícita de incluirlos.

En v0.1 basta la búsqueda de metadatos y conocimiento capturado. Buscar en todo el texto de los PDF exige extracción e indexación adicional y se pospone. Los PDF escaneados no se anuncian como buscables sin OCR.

El grafo, las listas de contradicciones y los indicadores de cobertura son proyecciones de SQLite. Un índice puede reconstruirse sin perder contenido canónico. La aplicación no necesita una segunda base de grafos para representar relaciones.

## 20 Exportación y contrato de portabilidad
El export completo incluye manifest, catálogo de ontología, papers, autores, venues, documentos, fases, respuestas, items, asociaciones, relaciones, procedencia y validaciones. JSONL conserva una entidad por línea; Markdown produce vistas legibles por paper y concepto.

```text
research-export/
  manifest.json
  papers.jsonl
  authors.jsonl
  venues.jsonl
  documents.jsonl
  phases.jsonl
  phase_answers.jsonl
  knowledge_items.jsonl
  concepts.jsonl
  relations.jsonl
  provenance.jsonl
  validations.jsonl
  associations.jsonl
  ontology/
  markdown/papers/
  markdown/concepts/
  files/                     opcional
```

Manifest contiene export_version, schema_version, ontology_versions, timestamp UTC, conteos y hashes de archivos. Todas las referencias emplean UUID; nombres y etiquetas se incluyen como ayudas de lectura, sin convertirse en claves.

Exportar metadatos no implica incluir los PDF. El usuario elige incluir documentos según el uso previsto; el manifest indica si el paquete contiene los binarios. Las rutas originales y datos locales de diagnóstico se excluyen de los exports compartibles por defecto.

Markdown es una representación derivada. Editarlo fuera de la aplicación no cambia automáticamente SQLite. Una futura importación requiere validación de esquema, conflictos y trazabilidad; no se promete round trip de Markdown en v0.1.

Ejemplo ilustrativo de un item exportado, sin afirmar un resultado científico real:

```json
{
  "export_version": "1.0",
  "id": "b89b75e3-26aa-49d0-8d10-0bce9e579321",
  "type": "claim",
  "title": "El método depende del contexto de entrenamiento",
  "origin": "researcher_interpretation",
  "confidence": "requires_validation",
  "concept_ids": [
    "2a3752dc-7a77-437d-90ee-c9f7fc813304"
  ],
  "provenance_ids": [
    "71f3fba5-4bf7-488d-b02f-7278c0cbe329"
  ],
  "conditions": "Condiciones aún por revisar",
  "created_at": "2026-10-01T10:00:00Z"
}
```

## 21 Backups migraciones y recuperación
Un backup útil incluye una copia consistente de SQLite, todos los documentos y recursos referenciados, la ontología activa y un manifest con versiones y hashes. Copiar únicamente research.db mientras existe un WAL activo puede producir un snapshot incompleto.

Se usa el mecanismo de backup de SQLite o una operación consistente equivalente. Los archivos se copian bajo una coordinación que impida que una importación o eliminación cambie el conjunto a mitad del snapshot. El backup se verifica antes de marcarlo correcto.

La política inicial propuesta es un snapshot diario cuando haya cambios, un snapshot antes de cada migración y backups manuales. La retención es configurable; nunca se elimina la última copia verificada antes de comprobar una nueva. Una copia en el mismo disco protege frente a errores lógicos, pero no frente a pérdida del disco.

Restaurar se realiza primero en una carpeta nueva. Se comprueban hashes, integridad de SQLite, claves foráneas y disponibilidad de documentos; se abre la biblioteca restaurada antes de sustituir la activa. La biblioteca previa permanece recuperable hasta confirmar el resultado.

Las migraciones tienen versión y checksum. Se ejecutan con exclusión de escritura y backup previo, y no destruyen datos para adaptar una interfaz. Si falla una migración, se mantiene el estado recuperable y se muestra un error concreto.

## 22 Privacidad y seguridad local
La aplicación no transmite papers, notas o metadatos en el núcleo v0.1. No incorpora telemetría de contenido. Los logs describen errores e identificadores técnicos sin copiar textos completos, rutas privadas innecesarias o documentos científicos.

El sistema usa los permisos del usuario del sistema operativo; no existe administración de usuarios interna. SQLite no se presenta como cifrado por defecto. La confidencialidad del disco y de backups depende de la protección del equipo y del destino elegido.

Los comandos de Tauri limitan su acceso a la biblioteca y a archivos seleccionados por el usuario. Se valida que las rutas resueltas permanezcan dentro de los destinos autorizados. Los exports no permiten que títulos o UUID manipulen rutas de salida.

Las consultas son parametrizadas. El contenido enriquecido se sanea, los PDFs y ontologías se tratan como datos no confiables y el visor no recibe privilegios de ejecución del backend. Los enlaces externos requieren una acción explícita del investigador.

El instalador y las actualizaciones se diseñan para conservar la biblioteca. La distribución debe comprobar firma, dependencias y permisos del runtime seleccionado, sin introducir acceso general al sistema por comodidad.

## 23 Calidad y pruebas
La validación combina pruebas de dominio, integración con SQLite y filesystem, y recorridos completos de la aplicación. Los ejemplos se ejecutan sobre bibliotecas temporales, nunca sobre la biblioteca personal para comprobar fallos destructivos.

Las pruebas de dominio cubren respuestas desconocidas justificadas, gates incompletos, avance bloqueado, edición posterior, reglas de relación, confianza, origen y fusiones. La misma regla debe funcionar aunque la operación no venga de la pantalla habitual.

Las pruebas de integración cubren transacciones, restricciones, normalización de DOI, importación duplicada, interrupción entre copia y registro, autosave fuera de orden, índices FTS y migraciones fallidas. También verifican que renombrar no cambia UUID ni rompe enlaces.

Los recorridos de interfaz cubren importación, PRE, P1, P2, creación desde selección y manual, consulta por concepto, cierre y reapertura, export y restauración. Se revisan teclado, foco, etiquetas de símbolos, errores de guardado y legibilidad del PDF.

El export se valida por conteos, referencias y versiones, y el backup mediante restauración real. Una copia creada sin una restauración comprobada no se considera evidencia suficiente de recuperación.

Los objetivos de rendimiento se fijan con el corpus del piloto. Deben medirse tiempos de apertura, guardado y búsqueda; la aplicación no muestra Guardado anticipadamente para cumplir un objetivo visual de velocidad.

## 24 Roadmap y secuencia de implementación
| Versión | Resultado | Condición para pasar a la siguiente |
|---|---|---|
| v0.1 | Captura local de PRE a P2 | Survey real, reapertura, conceptos, export y restore verificados |
| v0.2 | P3 y evaluación explícita | Cola de validación, confianza y procedencia operativas |
| v0.3 | P4 y comparación | Reconstrucción independiente y vistas globales ampliadas |
| v0.4 | Motor de reglas | Alertas explicables de incertidumbre y contradicciones |
| v0.5 | Asistencia IA opcional | Propuestas revisables sobre un corpus ya curado |

Dentro de v0.1 se construye primero la biblioteca, esquema y servicios de persistencia. Después se conectan Home y Library; se implementan PRE y P1 con gates; se añade el visor y la captura P2; y finalmente se cierran las vistas de conceptos, exportación y recuperación.

Cada corte debe ser utilizable y conservar lo anterior. No se empieza por un grafo complejo o motor lógico. Tras procesar entre cinco y diez surveys se revisa qué estructura resulta útil, qué campos sobran y dónde el usuario abandona el flujo. La incorporación de IA depende de calidad y utilidad del corpus, no de una fecha automática.

## 25 Criterios de aceptación de v0.1
1. El usuario instala y abre el programa en Windows sin crear una cuenta y puede trabajar sin conexión.
2. Importa un PDF legible, registra metadatos y vuelve a abrirlo tras mover el original.
3. PRE conserva propósito y baseline; el avance está bloqueado si faltan salidas requeridas.
4. P1 acepta un dato desconocido justificado y distingue continuar de archivar.
5. P2 crea conceptos, afirmaciones, preguntas y relaciones con origen y procedencia.
6. Dos papers pueden reutilizar un concepto con el mismo UUID sin duplicar su ficha.
7. Se puede crear un localizador manual y abrir el PDF en la página correspondiente.
8. Un cambio confirmado sigue presente tras cerrar y reabrir; un fallo de guardado se muestra.
9. Editar una fase anterior conserva historial y señala revisiones pendientes sin borrar P2.
10. Archivar un paper no elimina conocimiento compartido; un borrado muestra dependencias.
11. La búsqueda encuentra metadatos y contenido capturado con filtros coherentes.
12. El export contiene todas las entidades y asociaciones del caso de prueba, con UUID estables.
13. Un backup restaura una biblioteca que abre correctamente y conserva fuentes y relaciones.
14. P2 completo se muestra como tal, sin afirmar que P3 o P4 hayan sido realizadas.

## 26 Motor de reglas y futura integración con LLMs
El motor de v0.4 consulta datos explícitos y produce avisos explicables: contradicciones registradas sin revisión, afirmaciones sin evidencia enlazada, preguntas abiertas relevantes y relaciones con confianza baja. La ausencia de un enlace significa una carencia del registro, no que la afirmación sea falsa.

Una vista de frontera reúne preguntas, límites y contextos aún no evaluados por concepto. La cobertura depende de las fuentes importadas y del alcance declarado. Las reglas no demuestran exhaustividad bibliográfica ni resuelven automáticamente contradicciones.

Un LLM futuro recibe unidades con ID, tipo, contexto, procedencia y límites. Sus propuestas se guardan aparte del conocimiento aceptado, identificando modelo, fecha y fuentes utilizadas. Incorporar una propuesta exige revisión humana y no sobrescribe afirmaciones existentes.

Las primeras capacidades posibles son sugerir conceptos, duplicados, relaciones, candidatos P3 y preguntas. La exportación ya habilita consumo externo sin introducir agentes, embeddings o servicios remotos en v0.1. Cualquier envío externo de contenido se hace optativo y explícito.

## 27 Riesgos y decisiones de equilibrio
| Riesgo | Efecto | Mitigación |
|---|---|---|
| Exceso de campos | Captura abandonada | Formularios mínimos y detalle progresivo |
| Gates demasiado rígidos | Certeza inventada o flujo bloqueado | Unknown justificado y evaluación de completitud |
| Conceptos duplicados | Conocimiento fragmentado | Reutilización, alias y fusión revisada |
| Pérdida de contexto | Generalizaciones injustificadas | Condiciones, origen y fuente por objeto |
| Anclajes rotos | Pasajes difíciles de verificar | Documento versionado, hash y estado stale |
| Desajuste archivo y DB | Imports incompletos | Staging y reconciliación recuperable |
| Backup incompleto | Recuperación imposible | Manifest, hashes y restauración probada |
| JSON sin disciplina | Esquema opaco | Núcleo relacional y validadores versionados |
| Grafo visual saturado | Uso complejo | Fichas como flujo principal y filtros |
| IA prematura | Volumen sin calidad | Captura manual antes de propuestas automáticas |

SQLite simplifica distribución y operación de una aplicación personal, a cambio de limitar concurrencia y ciertas consultas analíticas complejas. Un supertipo tipado facilita evolución, pero exige validar atributos. Copiar PDFs consume espacio, pero evita depender de ubicaciones externas frágiles.

## 28 Registros de decisión arquitectónica
### ADR 001 Aplicación local de un solo usuario
Decisión: ejecutar un programa de escritorio sin autenticación ni backend remoto. Motivo: el trabajo pertenece a un investigador y debe ser operativo sin red. Consecuencia: la colaboración y sincronización requieren una decisión posterior de arquitectura.

### ADR 002 Tauri React Rust y SQLite
Decisión: separar UI web embebida, servicios Rust y persistencia SQLite. Motivo: interfaz moderna con servicios locales y una única biblioteca portable. Consecuencia: el IPC y las capacidades de archivos deben diseñarse como contratos limitados.

### ADR 003 SQLite como autoridad
Decisión: conservar conocimiento canónico en tablas relacionadas. Motivo: UUID, restricciones y transacciones son centrales para el modelo. Consecuencia: Markdown y grafos son salidas derivadas y sus ediciones no se sincronizan implícitamente.

### ADR 004 Conceptos globales con procedencia localizable
Decisión: organizar por conceptos compartidos y conservar anclajes específicos de cada fuente. Motivo: permitir síntesis entre papers sin borrar diferencias. Consecuencia: borrar una fuente exige revisar dependencias; los conceptos no se subordinan a un paper.

### ADR 005 Gates de dominio
Decisión: evaluar avance en el backend y aceptar incertidumbre procesada. Motivo: mantener disciplina sin confundir completitud y certeza. Consecuencia: se versionan plantillas, respuestas y revisiones.

### ADR 006 Documentos administrados y versionados
Decisión: copiar archivos y referenciar versiones concretas con hash. Motivo: asegurar disponibilidad y estabilidad de localizadores. Consecuencia: la importación debe reconciliar operaciones de archivos y base de datos.

### ADR 007 Captura manual antes de IA
Decisión: excluir automatización semántica del núcleo inicial. Motivo: validar primero qué información merece conservarse y cómo se procesa. Consecuencia: la asistencia futura propone; la aceptación continúa siendo humana.

### ADR 008 Ontología pequeña y extensible
Decisión: catálogo core con packs versionados y relaciones tipadas. Motivo: preservar significado sin una megaontología prematura. Consecuencia: las extensiones se validan y no se convierten en código ejecutable.

## 29 Enumeraciones y contratos mínimos
| Campo | Valores iniciales |
|---|---|
| ReviewType | survey, topical_review, slr, mapping_study, tutorial, other, unknown |
| PhaseCode | PRE, P1, P2, P3, P4 |
| PhaseState | NOT_STARTED, IN_PROGRESS, COMPLETED, NEEDS_REVIEW |
| PaperLifecycle | NEW, ACTIVE, COMPLETED, ARCHIVED, TRASHED |
| AnswerResolution | PENDING, ANSWERED, UNKNOWN, NOT_APPLICABLE |
| QuestionState | OPEN, ANSWERED, DEFERRED, DISMISSED |
| Origin | literature, researcher_interpretation, researcher_hypothesis |
| Confidence | sufficiently_supported, context_dependent, uncertain, requires_validation |
| LocatorState | PENDING, LOCATED, STALE |

Los enums persistidos son claves estables; las etiquetas visibles se traducen sin cambiar su significado. Añadir un estado exige migración o una estrategia de compatibilidad definida. Unknown y not_applicable necesitan una explicación cuando cierran una pregunta obligatoria.

El contrato mínimo de PhaseDefinition incluye code, version, objective, key_questions, do_items, dont_items, considerations, required_outputs y completion_rules. El resultado de evaluateGate devuelve complete y una lista de requisitos pendientes con identificador y explicación.

## 30 Preguntas abiertas antes de implementar
Se debe concretar el número de preguntas obligatorias por fase durante un piloto con un survey real; no añadir mínimos numéricos de conceptos para aparentar progreso. También debe decidirse qué campos bibliográficos son obligatorios y cuáles admiten unknown.

Se debe elegir el formato de cuerpo enriquecido y su conversión estable a texto, la biblioteca Rust de acceso a SQLite, las políticas de actualización e instalador y el tratamiento de PDFs protegidos o escaneados. OCR y metadatos remotos siguen fuera de v0.1.

Debe fijarse la política de retención y ubicación de backups, el nivel de historial requerido para deshacer fusiones y los tipos especializados que el piloto justifica. La primera entrega puede limitar las fusiones a casos seguros o posponer su interfaz avanzada, manteniendo UUID y trazabilidad desde el inicio.

Las pantallas iniciales a detallar son Home, Library, Paper Workspace y P2 Workspace. El siguiente paso de ingeniería es convertir esta especificación en contratos de servicios, migraciones y wireframes, y comprobar una secuencia vertical completa con datos reales antes de ampliar capacidades.
