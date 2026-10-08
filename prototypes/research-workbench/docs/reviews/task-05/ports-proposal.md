# T05 — propuesta de puertos, captura y esquema consumidor

**Estado: PROPOSED para publicación por Sol.** 2026-10-03. Las decisiones de Sol comunicadas durante este preflight están recogidas expresamente; las firmas y el DDL siguientes son propuestas nuevas que Sol revisará antes de publicar TASK05_PORTS/ADR. No sustituye todavía los documentos normativos ni activa T05a/T05b. BASE definitiva pendiente de integrar T04b y T04c.

**Fuentes:** AGENTS, INTENT, STATUS; IMPLEMENTATION y brief canónico T05; DOMAIN/DATA/CONTRACTS; SPEC-004 y WORKFLOW_GATES sólo en lo relevante; TASK04_PORTS. Runtime sólo leído con `git show 4162ac1355d214c7d5d47c4aa96695c526b2e3d3:<path>`. Ninguna lectura de WIP T04b. El informe previo `task-05-preflight.md` conserva las omisiones detectadas, no autoridad nueva.

**Objetivo:** catorce comandos existentes Knowledge/Concept, captura atómica y un schema0003 consumible por T06/T07/T08. No implementar RelationApi, ProvenanceApi, hash físico, gates/candidatos P2, formularios T06 o FTS/export futuros.

## 1. Decisiones de Sol para incorporar al ADR y contratos

| Decisión | Comportamiento fijado por Sol | Documento que precisará Sol |
|---|---|---|
| D1 clocks/candidatura | Cambio real de artefacto renueva clock P2 e invalida su aceptación conforme helper. PhaseAnswer candidata sólo cambia revisión si cambia la proyección de selección/count/justificación; editar body/confidence de candidato no inventa tal cambio | Brief T05, WORKFLOW_GATES/tests |
| D2 ausencia | `provenance=[]` produce `NONE`, sin Document/fila ficticia. Literatura NONE significa pendiente de atribuir; T06 debe mostrarlo. Documento explícito sin anclaje produce fila PENDING | CONTRACTS Knowledge/Provenance, brief T05/T06 |
| D3 Concept | preferredName/title y definition/body son el mismo canon; ConceptApi edita nombre/definición/alias/dominio. Knowledge.updateItem sobre Concept sólo admite confidence; title/body/attributes presentes se rechazan aunque sean iguales. Archive/restore genéricos delegan a lógica común | CONTRACTS Knowledge/Concept, DOMAIN/DATA |
| D6 creación | Padre item/concept y cada Provenance nueva tienen revision0. Captura crea todos juntos; no expectedParentRevision wire ni aumento por adjuntar las filas iniciales. Attach futuro de T07 sobre padre existente mantiene CAS/incremento propio | CONTRACTS/DATA/briefs |
| concept link | Nueva adición incrementa revision del item una vez; no revision del Concept destino. Adición repetida no-op, con receipt durable. Alcance anterior/posterior invalidado una vez/Paper | CONTRACTS Concept/WORKFLOW_GATES |
| listConceptItems | filter.conceptId null o igual al conceptId superior; distinto da InvalidInput antes de query. Los demás filtros se conservan | CONTRACTS Concept |
| asociación captura | Nuevas paper_items de createItem usan phase_code=P2; selected_for_p3=0, priority/rationale NULL. Es clasificación, no inicio/activación de fase | DATA/CONTRACTS/brief T05 |
| domain filter | OR existencial: Concept.domain propio o dominio de cualquier Concept enlazado mediante item_concepts; no Paper.domain implícito | CONTRACTS Knowledge/Concept |
| Insight affected | affectedConceptIds es subset explícito de item_concepts: create exige IDs también en conceptIds, update exige ya vinculados. Quitar affected no desvincula; link no añade affected. UUID/subtipo/existencia en TX y set canónico para no-op | CONTRACTS Knowledge/DOMAIN/briefs |
| catálogo | namespace core, versión1.0.0, doce tipos y trece relaciones/matriz DOMAIN exactos | DATA/DOMAIN/ADR |
| normalización | Texto: CRLF/CR→LF y trim exterior, preservando contenido interior. Nombre/alias normalizado: colapsar whitespace Unicode y Unicode lowercase, conservar tildes/puntuación; sin NFKC ni equivalencias semánticas | DATA/CONTRACTS |
| actualización locator T07 | STALE permanece STALE al editar; capturedHash conservado. documentId inmutable, distinto→InvalidInput. Otra fuente requiere nueva attach explícita, conservando anterior | CONTRACTS Provenance/ADR |
| hash T07 | checkDocumentHash no cambia Document.sha256 ni capturedHash; detecta hash físico distinto y LOCATED→STALE. No revive PENDING/STALE automáticamente | CONTRACTS Provenance/ADR |

Estas son precisiones sin nuevo wire v1. Las restricciones nuevas de semántica deben publicarse antes de despacho, no residir sólo en instrucciones informales del autor.

## 2. Normalización y canon

### Texto, nombres, dominio e identidad

`canonical_text(s)` reemplaza CRLF/CR por LF y recorta whitespace exterior; no colapsa espacios interiores, no modifica Unicode/case. Se aplica a campos de contenido tipados y a strings de presentación nombre/alias/dominio. Null sigue null; un nullable textual canónico vacío se conserva o rechaza según el campo del contrato, no se reemplaza silenciosamente por una explicación. Un campo requerido no puede quedar vacío donde su contrato exige contenido. Validar límites sobre texto recibido, antes de aceptar normalización; nunca truncar.

`normalized_name(s)` toma texto canónico, separa por whitespace Unicode, une con un espacio ASCII y convierte con Unicode lowercase. Ejemplos: `"  ÁGUA\t Residual  "`→`"água residual"`; `"AI/ML"`→`"ai/ml"`; `"Agua"` y `"Água"` permanecen diferentes. No normalizar UUID/enums ni corregirlos. Case lowering no prueba equivalencia ni permite fusionar sentidos.

Nombre/alias normalizado sirve para sugerencias y uniqueness **dentro de un Concept**. Normalización del filtro domain usa el mismo algoritmo; corresponde a igualdad normalizada OR existencial, no substring/inferencia bibliográfica. El domain de presentación se conserva; NULL no coincide con un domain solicitado. Un match por varios conceptos no duplica item en paginación. Suggestions consume la misma normalización de nombre/alias/query/domain.

affectedConceptIds es set canónico ordenado lexicográficamente y deduplicado para igualdad/no-op, según decisión de Sol; validar identidad antes de canonicalizar, sin fusionar Concepts. Para paperIds/conceptIds, propuesta de set canónico equivalente (no convertir distinto UUID en otro); Sol puede conservar rechazo de duplicados si requiere esa política. Alias normalizados duplicados dentro del mismo Concept se rechazan. Alias iguales entre Concepts y preferredName iguales entre sentidos distintos están permitidos. No ordenar arrays cuyo orden tenga semántica propia. La regla de duplicados de respuestas Workflow no se extiende silenciosamente a un input de Knowledge distinto.

### Persistencia Concept

Propuesta mínima: `concepts.preferred_name` es nombre canónico y `knowledge_items.title` su mirror atómico; definición canónica reside en `knowledge_items.body_text`, sin otra columna definition duplicada. `concepts.normalized_name/domain` y concept_aliases completan attrs. Tanto KnowledgeItemDto.attributes como ConceptDto.attributes/superiores se reconstruyen desde esa autoridad; no guardar attributes_json concept duplicado.

Create/update Concept actualiza nombre y mirror/title/normalizedName juntos. Una lectura que detecta preferred_name≠title o subtype incoherente devuelve IntegrityFailure; no repara silenciosamente. Definition/bodyText se devuelve idéntico en todas las vistas. Knowledge confidence-only modifica base/confidence, con CAS común de revision; no toca nombre/definición/domain/aliases. Patch mixto con confidence y campo prohibido rechaza entero, no aplica parcialmente.

Coste: mirror de nombre para el DTO genérico/índice futuro; se mantiene dentro de una TX y se comprueba al decodificar. Alternativa sería construir title desde concepts en cada query sin persistir mirror, pero knowledge_items requiere title y todas las lecturas genéricas deben seguir resolviendo doce tipos. No justificar con eso dos fuentes divergentes.

## 3. D4: localización mínima sin I/O

### Locator cerrado admitido

```ts
// Dentro del JsonObject wire ya existente; no nuevo comando ni nuevo Args.
type PageRegionLocator = {
  kind: 'page_region';
  x: number; y: number; width: number; height: number;
};
```

Sólo null o ese objeto exacto: sin keys adicionales, coordenadas finitas, x/y>=0, width/height>0, cada valor<=1, x+width<=1 e y+height<=1. Requiere pageIndex explícito entero>=1. No tolerancias silenciosas, clamping ni coerción de strings. InvalidInput si región inválida o sin página, aunque el servicio pudiera guardar una fila pendiente. La ausencia de region no requiere crear un rectangle ficticio.

Página física>=1 sola es un anclaje registrado navegable. pageLabel es etiqueta libre, no sustituto de página. Section/quote/pageLabel solos dejan PENDING. Quote nunca se convierte en body interpretativo y nunca prueba ubicación/verdad por su contenido.

### Estado al crear y al editar

1. T05 carga Document registrado en la misma TX; valida identidad/sha256 y copia su hash a capturedDocumentHash. No acepta hash de UI. El estado inicial con igualdad hash registrada es LOCATED si pageIndex>=1 (con region válida si existe), PENDING sin página.
2. LOCATED significa **localización registrada**, no cita verificada, PDF físico inspeccionado ni prueba de que la página existe en ese PDF. El número no se valida frente a page count en T05. T07 navega por Reader y comprueba que la página exista, mostrando la limitación/error si no; no inventa éxito de navegación ni altera procedencia para aparentarlo.
3. Si ya existe una Provenance STALE, updateLocator conserva STALE incluso si nueva página/región y hash registrado son válidos. Edición no obtiene hash físico ni revalida fuente. capturedHash y documentId no cambian. Distinto documentId recibido da InvalidInput; fuente nueva se adjunta de forma explícita.
4. Para filas no STALE, input inválido rechaza la actualización; una fila LOCATED a la que se quita página pasa a PENDING, una PENDING a la que se añade página válida puede pasar a LOCATED **como localización registrada**, siempre con igualdad captured/registrado. No cambia confianza ni valida cita.
5. Si capturedHash no coincide con Document.sha256 registrado en una fila existente, hay discordancia documental: update no puede producir LOCATED. Propuesta de precisión a Sol: tratarla STALE al modificar ese locator, conservando hash y Document; en v0.1 normalmente no nace esta discordancia porque Document.hash es inmutable. No reescribirlo como reparación. Decoder debe aceptar STALE legítimo aunque registrado y capturado coincidan, porque el stale físico detectado no modifica esos hashes.
6. checkDocumentHash de T07 compara hash físico con capturado/registrado según su contrato, marca LOCATED→STALE ante cambio y renueva sólo Provenance/clocks correspondientes en su UoW; no modifica canon documental. Sin rebind/revalidate en v0.1: corregir una fuente requiere nueva Document/version y nueva Provenance conservando historia, fuera de T05. No prometer que una biblioteca externa se repara automáticamente.

**Coherencia con contrato:** CONTRACTS Provenance §351 ya exige conservar capturedHash y que cambiar localizador no valide cita. STALE sticky cierra el riesgo de resucitar un locator comparando dos hashes registrados que continúan iguales tras un cambio físico. documentId inmutable precisa una libertad antes ambigua del input. El coste es que v0.1 no ofrece un botón de «quitar STALE» ni reasignar fuente; se preserva historia. Sol publicará estas precisiones y los tests T07; T05 sólo implementa creación.

### Primitiva pura compartida propuesta

Transferir `src-tauri/src/domain/provenance.rs` inicialmente a T05a para **una regla común de anclaje**; T07 lo amplía después para sus servicios. No segunda validación en capture ni UI.

```rust
// Nuevos tipos puros propuestos; sin imports de SQLite/Tauri/transport DTO.
pub struct PageRegion {
    pub x: f64, pub y: f64, pub width: f64, pub height: f64,
}
pub enum RegisteredAnchor { Pending, Located }
pub enum LocatorInputError { InvalidPage, InvalidRegion }

pub fn registered_anchor(
    page_index: Option<i64>, region: Option<&PageRegion>,
) -> Result<RegisteredAnchor, LocatorInputError>;
```

La aplicación decodifica JsonObject cerrado a PageRegion y mapea error→InvalidInput. La función valida page y bounds, sin leer Document/hash/I/O. Captura garantiza hash registrado=capturado al crear; T07 conserva identity/hash/STALENESS en su caso de uso alrededor de la misma regla. Así no se oculta una consulta de DB/FS en un helper aparentemente puro ni se añade un framework de estados.

## 4. D5: schema0003 exacto propuesto para consumidor

SQL-only; MIGRATIONS añade versión3, SCHEMA_VERSION3; conservar0001/0002/checksums y runner con snapshot previo. Catálogo seed1.0.0 inmutable, paridad de doce tipos/trece relaciones/matriz DOMAIN en prueba. FTS permanece0004. No crear tablas ni handlers de aceptación P2/candidatos aparte de fields paper_items ya contratados.

| Tabla | Columnas propuestas | Claves/checks/autoridad |
|---|---|---|
| ontology_types | code TEXT, ontology_version TEXT | PK code; doce códigos seed; versión core1.0.0 |
| relation_types | code TEXT, ontology_version TEXT, symmetric INTEGER, endpoint_matrix_json TEXT | PK code; trece seed; symmetric0/1; matriz JSON declarativa cerrada, parejas de códigos FK/validación semántica; sin código ejecutable |
| knowledge_items | id TEXT, type_code TEXT, title TEXT, body_text TEXT, body_format TEXT, body_json TEXT NULL, attributes_json TEXT NULL, origin TEXT, confidence TEXT, lifecycle TEXT, revision INTEGER, created_at TEXT, updated_at TEXT | PK id; FK type catálogo; plain_text/body_json NULL; enums cerrados; revision0..2^53−1; authority de attrs descrita abajo |
| concepts | item_id TEXT, preferred_name TEXT, normalized_name TEXT, domain TEXT NULL, merged_into_id TEXT NULL | PK/FK item_id; tipo concept; merged_into_id NULL v0.1; sin uniqueness global; preferred_name/title mirror atómico; definition sólo base.body_text |
| concept_aliases | concept_id TEXT, alias TEXT, normalized_alias TEXT | PK(concept_id,normalized_alias); FK concepts.item_id; alias no vacío; sin uniqueness global |
| evidence_details | item_id TEXT, evidence_kind TEXT, observed_result TEXT NULL, conditions_text TEXT NULL, limitations_text TEXT NULL | PK/FK item_id; item.type=evidence; única authority de esos attrs |
| question_details | item_id TEXT, state TEXT, answer_text TEXT NULL, next_action TEXT NULL | PK/FK item_id; item.type=question; estados OPEN/ANSWERED/DEFERRED/DISMISSED; única authority attrs |
| paper_items | paper_id TEXT, item_id TEXT, phase_code TEXT, selected_for_p3 INTEGER, priority INTEGER NULL, rationale TEXT NULL | PK par; FKs papers/items; phase_code enum fases; initial P2; selected0/1; priority NULL o1..5, rationale máximo5000; creación sin selección/priority/rationale. T06 verifica seleccionables claim/question/justificación |
| item_concepts | item_id TEXT, concept_id TEXT | PK par; FK knowledge_items y concepts.item_id; enlaces archivados conservados |
| relations | id TEXT, source_item_id TEXT, target_item_id TEXT, type_code TEXT, context_text TEXT, justification_text TEXT, origin TEXT, confidence TEXT, lifecycle TEXT, revision INTEGER, created_at TEXT, updated_at TEXT | PK id; ambos FK items distintos; FK relation_types; enums/revisión; order normalizado simétricas; matrix validada por caso de uso T07/decoder |
| provenance | id TEXT, document_id TEXT, page_index INTEGER NULL, page_label TEXT NULL, section TEXT NULL, quote_text TEXT NULL, locator_json TEXT NULL, captured_document_hash TEXT, locator_state TEXT, revision INTEGER, created_at TEXT, updated_at TEXT | PK id; FK documents; page>=1 si existe; estado cerrado; hash backend; initial revision0; locator JSON cerrado D4 |
| item_provenance | item_id TEXT, provenance_id TEXT | PK par/FKs, initial capture sólo item |
| relation_provenance | relation_id TEXT, provenance_id TEXT | PK par/FKs; tabla creada por T05; servicio T07 |
| validations | id TEXT | PK id; **reserva vacía**, sin comandos/filas/mock. P3 posterior amplía por nueva migración, no edición0003 |

Cada columna no marcada NULL es NOT NULL. PK/FKs explícitas; no ON DELETE CASCADE ni triggers destructivos. No estado previo extra para items/relations: su único estado restaurable es ACTIVE, a diferencia de Paper. UUID/UTC/hashes validados por writer y decoder; CHECK básicos complementan, no sustituyen invariantes con joins. future/core desconocido no se trata como tipo permitido.

### Única authority de atributos

- `attributes_json` **NULL** para concept/evidence/question. Concept reconstruye attrs desde concepts/base.body/aliases; evidence/question desde su tabla detail. Cada subtype exige exactamente su fila; fila ausente/incompatible produce IntegrityFailure, no defaults.
- Para claim/gap/assumption/condition/limitation/method/example/insight/reference, attributes_json es objeto discriminado completo con typeCode persistido y exactos attrs CONTRACTS; no JSON libre de usuario. Campo requerido nullable sigue presente con null. JSON desconocido/mismatch/over-limit rechaza; lectura/import futura también valida.
- KnowledgeItemDto.attributes siempre reconstruido como union completo de doce tipos. Ningún DTO contiene atributos nulos porque no haya attributes_json.
- `body_format='plain_text'`, `body_json IS NULL`, revision>=0 y<=2^53−1. Lifecycle sólo ACTIVE/ARCHIVED. La revisión única de Concept vive en knowledge_items, no otra revision en concepts/aliases/details.

### Índices y unicidad de Relations

Índices: items(type_code,lifecycle,id); aliases(normalized_alias,concept_id); concepts(normalized_name,item_id), concepts(domain,item_id); paper_items(item_id,paper_id) además de PK; item_concepts(concept_id,item_id) además de PK; provenance(document_id,id); enlaces provenance por provenance_id; relations(source_item_id,id) y(target_item_id,id).

Propuesta de índice parcial activo Relations: UNIQUE(source_item_id,target_item_id,type_code,context_text) WHERE lifecycle='ACTIVE'. El caso de uso T07 canoniza contexto y extremos simétricos antes de insertar; contextos distintos no se deduplican, archived preservado no bloquea vínculo nuevo. Restore que colisiona con activo da Conflict, no merge/pérdida de historia. Justification/origin/confidence son contenido del vínculo, no parte de su identidad semántica; esta elección necesita aceptación Sol antes de congelar0003. No servicios T07 en T05 para «probar» ese índice: fixtures SQL/constraints explícitamente etiquetados como schema tests.

Export contract mapping: attributes_json/detalles se proyectan a KnowledgeItem.attributes; Concept preferredName/domain/mergedIntoId y alias records; asociaciones paperItem/itemConcept/itemProvenance/relationProvenance preservan todos sus campos; provenance locator_json→locator. validations no se exporta con filas fabricadas. T08 recibe mapping/rows fixture, no implementa export con raw SELECT*.

## 5. ABI mínima de aplicación/persistence

### Patrones existentes comprobados

En4162ac1 `application/ports.rs` **sí** define:

```rust
pub trait Clock: Send + Sync { fn now_utc(&self) -> String; }
pub trait IdGenerator: Send + Sync { fn next_uuid(&self) -> String; }
// SystemClock / RandomIds también existen allí.
```

Reader usa Repository con TX prestada y ReaderPersistence async boxed Send; ReaderService hace admisión/owned spawn/oneshot y mueve RequestPermit/OperationPermit hasta resultado. T05 reutiliza DbActor64, MaintenanceCoordinator, RequestRegistry y RecoveryStatus. El servicio no es dueño de SQL/TX; repositorio no hace actor/receipt/commit. No crear traits Clock/IDs nuevos ni suponerlos por nombre.

### Propuesta de servicios/persistence públicos internos

Nuevo `application/knowledge_ports.rs`; nuevas firmas, pendientes de aceptar Sol. Reusar Args/DTO transport existentes en **aplicación**, no importarlos en dominio puro. Una implementación `SqliteKnowledgePersistence` puede implementar ambos traits y compartir actor/repos; no segundo actor.

```rust
pub type KnowledgeFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;

pub trait KnowledgePersistence: Send + Sync {
    fn writable(&self) -> bool;
    fn create_item(&self, args: KnowledgeCreateItemArgs) -> KnowledgeFuture<'static, KnowledgeItemDto>;
    fn update_item(&self, args: KnowledgeUpdateItemArgs) -> KnowledgeFuture<'static, KnowledgeItemDto>;
    fn archive_item(&self, args: KnowledgeArchiveItemArgs) -> KnowledgeFuture<'static, KnowledgeItemDto>;
    fn restore_item(&self, args: KnowledgeRestoreItemArgs) -> KnowledgeFuture<'static, KnowledgeItemDto>;
    fn get_item(&self, args: KnowledgeGetItemArgs) -> KnowledgeFuture<'static, KnowledgeItemDto>;
    fn list_items(&self, args: KnowledgeListItemsArgs) -> KnowledgeFuture<'static, PageDto<KnowledgeItemDto>>;
}
pub trait ConceptPersistence: Send + Sync {
    fn writable(&self) -> bool;
    fn suggest(&self, args: ConceptSuggestArgs) -> KnowledgeFuture<'static, Vec<ConceptDto>>;
    fn get(&self, args: ConceptGetArgs) -> KnowledgeFuture<'static, ConceptDto>;
    fn list_items(&self, args: ConceptListItemsArgs) -> KnowledgeFuture<'static, PageDto<KnowledgeItemDto>>;
    fn create(&self, args: ConceptCreateArgs) -> KnowledgeFuture<'static, ConceptDto>;
    fn update(&self, args: ConceptUpdateArgs) -> KnowledgeFuture<'static, ConceptDto>;
    fn link(&self, args: ConceptLinkArgs) -> KnowledgeFuture<'static, ()>;
    fn archive(&self, args: ConceptArchiveArgs) -> KnowledgeFuture<'static, ConceptDto>;
    fn restore(&self, args: ConceptRestoreArgs) -> KnowledgeFuture<'static, ConceptDto>;
}
```

Request/maintenance permits se adquieren y retienen en trabajo propietario por mutación hasta que el future persistence termina tras commit/rollback. Cancelar IPC abandona receptor, no rollback garantizado. Writable/recovery antes de nueva operación; lookup durable/replay antes de CAS y guards de contenido/lifecycle en wrapper, sin proof físico. Lecturas sin receipt; no iniciar/activar fase por consultar.

### Casos TX principales y responsabilidades repository

```rust
// application/knowledge.rs, propuestas; no dominio con rusqlite.
pub fn capture_item_in_tx(
    tx: &Transaction<'_>, knowledge: &dyn KnowledgeRepository,
    concepts: &dyn ConceptRepository, workflow: &dyn WorkflowRepository,
    args: &KnowledgeCreateItemArgs, now: &str, ids: &dyn IdGenerator,
) -> Result<KnowledgeItemDto, AppError>;
pub fn create_concept_in_tx(
    tx: &Transaction<'_>, knowledge: &dyn KnowledgeRepository,
    concepts: &dyn ConceptRepository, workflow: &dyn WorkflowRepository,
    args: &ConceptCreateArgs, now: &str, ids: &dyn IdGenerator,
) -> Result<ConceptDto, AppError>;
pub fn set_item_lifecycle_in_tx(
    tx: &Transaction<'_>, knowledge: &dyn KnowledgeRepository,
    workflow: &dyn WorkflowRepository, item_id: &str,
    expected_revision: i64, next: KnowledgeItemDtoLifecycle, now: &str,
) -> Result<KnowledgeItemDto, AppError>;
```

Persistence obtiene `now=clock.now_utc()` sólo para ejecutar acción nueva; replay no regenera IDs ni fechas. Generador existente entrega UUID canónicos; inicial revision0. Entrar en esas funciones exige wrapper receipt/TX ya abierto; nunca nested receipts.

Puertos repository mínimos propuestos; SQL sólo en SQLite. Estos métodos son datos/persistencia, no nuevos commands:

```rust
pub trait KnowledgeRepository: Send + Sync {
    fn item(&self, tx: &Transaction<'_>, item_id: &str) -> Result<KnowledgeItemDto, AppError>;
    fn document(&self, tx: &Transaction<'_>, document_id: &str) -> Result<DocumentDto, AppError>;
    fn paper(&self, tx: &Transaction<'_>, paper_id: &str) -> Result<PaperDto, AppError>;
    fn insert_item(&self, tx: &Transaction<'_>, item: &KnowledgeItemDto) -> Result<(), AppError>;
    fn write_item(&self, tx: &Transaction<'_>, expected_revision: i64,
        next: &KnowledgeItemDto) -> Result<usize, AppError>;
    fn add_paper_link(&self, tx: &Transaction<'_>, paper_id: &str,
        item_id: &str, phase: PhaseCode) -> Result<(), AppError>;
    fn add_concept_link(&self, tx: &Transaction<'_>, item_id: &str,
        concept_id: &str) -> Result<bool, AppError>;
    fn insert_initial_provenance(&self, tx: &Transaction<'_>, item_id: &str,
        provenance: &ProvenanceDto) -> Result<(), AppError>;
    fn affected_papers(&self, tx: &Transaction<'_>, item_id: &str) -> Result<Vec<UUID>, AppError>;
    fn audit_change(&self, tx: &Transaction<'_>, request_id: &str,
        item_id: &str, action: &str, now: &str) -> Result<(), AppError>;
}
pub trait ConceptRepository: Send + Sync {
    fn concept(&self, tx: &Transaction<'_>, concept_id: &str) -> Result<ConceptDto, AppError>;
    fn insert_details(&self, tx: &Transaction<'_>, concept: &ConceptDto) -> Result<(), AppError>;
    fn write_details(&self, tx: &Transaction<'_>, concept: &ConceptDto) -> Result<(), AppError>;
}
```

`insert_item/write_item` escriben base+detalle evidence/question/JSON general, **no** edges ni detalles concept; add-links y primitive provenance escriben sólo sus datos. Concept write usa Knowledge.write_item con CAS compartido seguido de Concept.write_details dentro de la misma TX; rollback total si falla. Repository.item decodifica doce tipos con un decoder SQLite único reutilizado por ConceptRepository.concept, no dos authorities de atributos. `add_concept_link` devuelve true sólo si insertó; conflicto subtype/registro corrupto es error, no false. No INSERT OR REPLACE.

`affected_papers(item_id)` incluye todos los Papers cuyo closure P2 alcanza el item/concept, también archivados/NOT_STARTED para que el helper decida estado. Para Concept alcanza sus paper_items directos y items que lo enlazan; el closure incorpora conceptos de relaciones y los referenciados por Insight mediante el subset ratificado en §6. No consulta lifecycle para ocultar historia. Caller recoge conjunto antes/después del cambio, une/deduplica y llama una sola vez/Paper:

```rust
invalidate_effective_change(tx, workflow, &paper_id.0, &[PhaseCode::P2])?;
```

Reusar signature publicada TASK04_PORTS; el helper mantiene NOT_STARTED/clock/state según norma. Item mutation incrementa item.revision una vez si cambia contenido/lifecycle/enlace; link no incrementa Concept target. Audit_change sólo modificación real, con acción cerrada/segura definida por aplicación; el wrapper puede mantener su evento de receipt sin presentarlo como cambio de contenido. Stale expectedRevision da Conflict incluso no-op; receipt exacto replay primero.

Query/suggest/paginación son métodos de persistence y SQL adapter, sin TX mutante ni receipt; pueden usar lectura consistente DEFERRED cuando ensamblan varias filas. No crear un query repository genérico. Cursor opaco con orden estable por UUID y filtros canónicos; cambios de filtro/cursor incompatible InvalidInput. Limits1..500. Fixtures/assertions consumidoras fijan orden, no rowid público.

## 6. Insight.affectedConceptIds: tensión resuelta por Sol

CONTRACTS define esos IDs como atributo editable de Insight, y conceptIds/item_concepts como asociación separada. No dice que sean alias ni cómo se preservan FKs/alcance P2; Knowledge.update puede cambiar affectedConceptIds, linkConcept sólo añade asociación. Copiar equivalencia estricta haría que enlazar un concepto modificase «afectado» sin intención, y quitar un afectado podría intentar eliminar asociación sin API unlink.

**Decisión ratificada por Sol para publicación:** conceptos afectados son un subconjunto semántico explícito de conceptos vinculados. Create exige affectedConceptIds⊆conceptIds recibidos y existentes; update exige afectados ya en item_concepts (linkConcept previo permite agregar uno). Atributo JSON mantiene la clasificación «afectado», asociación mantiene contexto general y FK. Retirar affected no elimina asociación temática; link no agrega affected. No autoagregar vínculos implícitos ni inventar API/junction. affectedConceptIds se valida como set ordenado/deduplicado para no-op, sin equivalencia semántica entre UUIDs.

Así todo concepto afectado aparece en closure P2 vía item_concepts, y `affected_papers` conserva el alcance incluso cuando no hay paper_items directo del Concept. Validar UUID/tipo/existencia/subset bajo la misma TX; texto/JSON no elude FK. Test insight compartido A/B con Concept sin paperIds directos: editar ese Concept invalida ambos una vez y conserva C no alcanzable.

Pruebas de rechazo atómico: create con ID válido no incluido expresamente en conceptIds, update con ID existente no vinculado, ID inexistente/subtipo noConcept o UUID inválido; no item/attrs/link/receipt/clocks parciales. Orden distinto/duplicados del mismo affected UUID canonizado no cambian revision/P2clock. Quitar affected conserva enlace y alcance; link adicional no modifica la clasificación afectada.

**Alternativa/coste:** IDs afectados independientes de asociaciones requieren otro enlace físico con FK o defensa redundante sobre JSON y una segunda ruta de alcance; amplían schema0003/query sin capacidad nueva. La equivalencia total reduce semántica distinta de los dos campos y entra en conflicto con linkConcept. Sol ratificó el subset; revisión independiente puede cuestionarlo antes de publicación normativa.

## 7. Secuencia T05a/T05b y ownership

T05a/T05b secuenciales, autor nuevo por corte y una revisión independiente de cada uno. Sol confirma dependencias/rama/worktree/BASE al activar, registra transferencias y revisa diff completo. No dos editores de0003 ni subagentes del autor.

**T05a — esquema/dominio/captura de aplicación:** posee domain/{knowledge,concepts,ontology,provenance}.rs; application/{knowledge,knowledge_ports}.rs; adapters/sqlite/{knowledge_repository,concept_repository}.rs y migrations/0003_knowledge.sql; modules/knowledge/capture.rs como delegación, sin segunda regla; tests/{knowledge_capture,concepts,ontology}.rs. Shared transferidos: mod.rs pertinentes y migrations.rs sólo registro/version3. Puede probar las funciones TX con SQLite real/receipt existente sin handlers de comandos activos. Ninguna capability Knowledge/Concept verdadera por integración de sólo esquema.

**T05b — lifecycle/query/IPC/composición:** continúa archivos T05 donde hagan falta; módulos knowledge/service/concept_service; commands/{knowledge,concept}.rs; shared lib.rs, desktop/lifecycle.rs para registro de servicios, commands/mod.rs para conectar catorce branches existentes y modules/settings.rs para capabilities completas. Tests shared sólo fixtures afectados por campos/version (`scaffold_contracts,desktop_bootstrap,schema_diagnostic,db_actor`). TypeScript sólo contract tests si necesarios, sin producto/UI; revisión TS si delta. No tocar actor/receipts/Windows/Reader/workflow helper wire/manifests/locks/config/deps/scripts/release.

DTO/generated/ports/client/commands/permissions actuales ya declaran wire; no cambio de significado unilateral. Si integración necesita un archivo no transferido, Sol asigna tras evidencia. T07 obtiene domain/provenance compartido sólo después de este corte; su attach/update/hash y servicios siguen pendientes. T06 consume catorce IPC/DTO/normalización, no considera completado gate P2. T08 recibe esquema+rows fixture y mapping export, sin runtime de export simulado.

## 8. Pruebas de contrato y criterios de aceptación

- T05a: upgrade0002→0003/reopen/catálogo exacto/matriz/checksum/FKs y fallo rollback con backup; doce attrs roundtrip con authority única; Concept mirrors consistentes; padre/Provenance revision0; pending sin source ficticio; límites/unknown JSON/region inválida; captura multiobjeto rollback sin clocks/receipts/audit huérfanos.
- Locator: page1+null region→LOCATED; todo null→PENDING; quote/section/pageLabel solos→PENDING; region válida boundaries→LOCATED; finite/bounds/width/height/page faltante InvalidInput. Nada de ello acredita page count/PDF/cita. T07 futura prueba STALE sticky/documentId inmutable/capturedHash sin rewrite/checkHash no revival; no escribir esos servicios en T05.
- Canon: updateItem de Concept rechaza title/body/attributes incluso idénticos o junto con confidence; confidence-only CAS/no-op correcto; updateConcept domain omitido preserva, domain:null limpia (Args real Option<Option<String>>). Rename mantiene UUID y ningún homónimo merge.
- P2: captura P2 association no inicia fase/contexto ni cambia NOT_STARTED; cambio compartido Concept alcanza A/B; link nuevo cambia itemrevision/P2clock una vez, destino conceptrevision intacta; duplicado no-op. Insight rechazo atómico/subset/no-op y alcance de todos los Papers según §6 ratificada. No completar gate fixture como supuesto runtime futuro.
- Candidatos: cambio body de item seleccionado modifica artifact/P2clock, no PhaseAnswer candidata si proyección igual; cambio de selección/proyección es responsabilidad T06. Eliminar test excesivo del brief y separar los dos casos.
- T05b: catorce handlers reales/SQL fixtures; filters/domain OR existencial sin duplicate rows; listConceptItems null/igual aceptado, distinto InvalidInput; archived references resuelven; receipts/replay antes de CAS/guardas; receiver drop retiene permisos hasta commit/rollback; read-only esquema futuro sin mutation.
- Capability knowledge/concepts true sólo con todos sus métodos conectados y probados; provenance/relations/gates-candidatos P2 siguen pendientes. Debug build/gate no acredita instalador/UI/página real ni equipo limpio.

Futuros comandos en worktree activado, no ejecutados por esta preparación:

```powershell
. .\scripts\development-env.ps1
cargo test --manifest-path src-tauri/Cargo.toml --test knowledge_capture
cargo test --manifest-path src-tauri/Cargo.toml --test concepts
cargo test --manifest-path src-tauri/Cargo.toml --test ontology
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
npm.cmd run tauri:build -- --debug --no-bundle
```

## 9. Precisiones que Sol debe cerrar antes de aceptación final

1. Política de duplicados de paperIds/conceptIds: se propone set canónico ordenado/deduplicado; decisión ya fijada para affectedConceptIds, no extrapolar automáticamente el rechazo de duplicates de PhaseAnswer. Cualquier opción debe mantener igualdad/no-op, UUID explícitos e integridad de captura.
2. Aceptar representación física propuesta y UNIQUE activo Relations por extremos/tipo/contexto, incluido restore collision=Conflict; T07 no podrá reescribir0003 publicada. Confirmar handling discordancia capturedHash/registrado en update §3.5 sin implementar hash futuro en T05.
3. Lifecycle de escritura sobre padres archivados: docs exigen get/enlaces archivados navegables y archive/restore reversible, pero no fijan claramente si updateItem/updateConcept/linkConcept admiten un padre ARCHIVED. Propuesta para Sol: edición de padre archivado rechaza con InvalidInput y pide restore explícito; referencias a Concept destino archivado pueden seguir resolviendo, y nuevo link a destino archivado necesita política explícita (admitir selección consciente o rechazar hasta restore). No inferir prohibición de leer porque un extremo esté archivado. Esta política afecta guards, no un wire nuevo.

Sol publicará ADR/TASK05_PORTS y sincronizará briefs/CONTRACTS/DATA afectados antes de despachar. No se solicitan decisiones al usuario ni se presentan estas propuestas nuevas como aceptadas.

**Evidencia de preparación:** único archivo escrito `task-05-ports-proposal.md`; sin normativos/producto/subagentes, sin instalación/build/tests/commits/merges/BASE definitiva. Runtime traits Clock/IdGenerator/Reader patrón comprobados en commit estable; las APIs Knowledge/Concept existentes se conservan. Las firmas nuevas del documento se someten a Sol para aceptación, no se afirman implementadas.
