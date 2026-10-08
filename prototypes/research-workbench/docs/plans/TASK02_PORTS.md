# T02 — preflight de puertos internos
Fecha: 2026-10-01. Estado: DONE_WITH_CONCERNS. Propuesta para decisión de Sol; no modifica arquitectura normativa ni CONTRACTS v1.

## Decisión propuesta
Tres fronteras con consumidores reales: selección nativa, trabajos de documentos y persistencia específica de Library. LibraryService coordina esas fronteras; DbActor, Connection, LibraryRoot y ejecución bloqueante concreta quedan en adaptadores/composición. Los helpers de aplicación *_in_tx y el repositorio de Library reciben &rusqlite::Transaction, conservando la excepción pragmática ya autorizada. No crear un executor genérico, un contenedor DI ni un trait de Workflow anticipado.

La enumeración normativa de LibraryApi contiene **ocho métodos**: selectPdf, confirmImport, cancelImport, listPapers, getPaper, updateMetadata, archivePaper y restorePaper. El número del brief es correcto. reconcile_imports es interno; no añade comando público, DTO wire ni permiso.

## Contratos y ownership
Pseudofirmas internas; Async<T> significa resultado asíncrono Send de Result<T, AppError>, no una biblioteca nueva. Los inputs/records son tipos de aplicación o dominio, sin serialización pública. Transporte convierte los DTOs v1 existentes.

| Contrato / ubicación | Productor | Consumidor |
|---|---|---|
| NativePdfSelection, application/library_ports.rs | Adaptador nativo asignado explícitamente por Sol; versión/API del picker la verifica Sol | LibraryService.select_pdf |
| DocumentStore, mismo archivo | adapters/documents/store.rs e imports.rs; posee raíz segura y ejecución background | LibraryService y reconcile_imports |
| LibraryPersistence, mismo archivo | SqliteLibraryPersistence en adapters/sqlite/library_repository.rs; posee/referencia DbActor | LibraryService y reconcile_imports |
| LibraryRepository transaccional, modules/library/repository.rs | Repositorio SQL en adapters/sqlite/library_repository.rs | Helpers de aplicación Library *_in_tx; futuras consultas declaradas para Reader/Workflow |
| lib.rs / desktop | Sol compone implementaciones; transferencias de shared files explícitas | Transporte y arranque |

Agregar application/library_ports.rs y el adaptador concreto de selector al ownership del brief. El worker no decide su ubicación ni edita shared files sin transferencia. El registro temporal de tokens y la exclusión por import pertenecen a LibraryService; no son otro puerto ni otra tabla.

```rust
trait NativePdfSelection {
    select_pdf() -> Async<Option<SelectedPdf>>;
}
trait DocumentStore {
    stage(operation_id, selected: SelectedPdf) -> Async<StagedPdf>;
    inspect_owned(intent: ImportRecord) -> Async<ResourceInspection>;
    promote(prepared: PreparedImport) -> Async<PromotionProof>;
    cleanup_owned(plan: CancelPlan) -> Async<CleanupProof>;
}
trait LibraryPersistence {
    begin_import(NewImportIntent) -> Async<ImportRecord>;
    record_staged(operation_id, StagedPdf) -> Async<PreviewFacts>;
    prepare_confirmation(ConfirmCommand) -> Async<PreparedOrReplay>;
    record_promoted(PreparedImport, PromotionProof) -> Async<()>;
    commit_confirmation(ConfirmCommand, PromotionProof) -> Async<PaperView>;
    prepare_cancel(CancelCommand) -> Async<CancelPlanOrReplay>;
    commit_cancel(CancelCommand, CleanupProof) -> Async<()>;
    recoverable_imports() -> Async<Vec<ImportRecord>>;
    // Firmas tipadas de list/get/update/archive/restore con sus
    // filtros, requestId, expectedRevision y metadata existentes.
}
fn confirm_in_tx(
    tx: &Transaction<'_>, repo: &dyn LibraryRepository,
    command: &ConfirmCommand, verified: &PromotionProof
) -> Result<PaperView, AppError>;
```

SelectedPdf es una capacidad Rust no serializable producida únicamente por el selector: contiene selección/archivo autorizado y nombre; ninguna ruta libre procede de React. El DocumentStore consume esa capacidad para copiar; no recibe paths wire. StagedPdf/PreparedImport/PromotionProof contienen IDs y referencias relativas administradas, biblioteca y hash/tamaño comprobados. El store verifica raíz, propiedad y reparse points antes de actuar; ResourceInspection distingue coincidencia inequívoca, incompleto, ausente y ambiguo, sin borrar.

Los métodos de LibraryPersistence no implementan negocio en SQL. Su adaptador hace actor.submit, abre la UoW pertinente y llama al helper de aplicación con el repositorio transaccional. Los helpers aplican validación/duplicados/revisions/transiciones/auditoría de dominio. El adaptador usa with_receipt para las mutaciones públicas; serializa/deserializa únicamente el resultado interno correspondiente y conserva su receipt. Ningún helper/repositorio hace commit. Las consultas también van al hilo DB, con snapshot corto cuando necesitan varias lecturas.

## Recorrido y garantías
1. select_pdf adquiere selección nativa; el servicio crea importOperationId/importToken, registra intención STAGING y autoriza token para libraryId+sesión actual. Copia/hash/PDF/tamaño se ejecutan fuera de TX y fuera del hilo DB/ventana. record_staged publica los hechos verificados; no crea Paper.
2. prepare_confirmation comprueba receipt previo, token, expiry, biblioteca/sesión y decisión de duplicado; en TX corta guarda intención confirmada, IDs reservados, destino relativo y contenido necesario para replay. PreparedOrReplay distingue resultado previo, decisión requerida y promoción autorizada. El replay se comprueba antes de promover.
3. promote verifica propiedad/hash y promueve en mismo volumen fuera de TX. record_promoted conserva la frontera durable. Un corte antes de esa actualización se resuelve cotejando archivos e intención.
4. commit_confirmation vuelve a cargar intención, comprobar payload/estado/duplicados y aplicar confirm_in_tx bajo with_receipt. Paper+autores ordenados+Document+COMMITTED+resultado durable+receipt+auditoría se confirman juntos; devuelve éxito solo tras COMMIT. Un fallo mantiene intención y archivos recuperables. Un duplicado aparecido entre preparación y commit exige decisión; no se crea otra entidad.
5. Reutilizar no promueve un segundo documento ni modifica el existente; COMMITTED y resultado de reuse quedan vinculados al token y al receipt. Cleanup de recursos propios puede quedar pendiente y debe aparecer como incidencia recuperable.
6. El permiso de operación del coordinador de mantenimiento abarca toda la operación aceptada, incluso sus fases filesystem. Una exclusión local por token evita confirm/cancel simultáneos; no mantener una TX o bloquear la ventana para conseguirla. Los jobs usan snapshots/IDs propios, no referencias a Transaction ni archivos de otra operación.

Reintentos por requestId/command/payload usan la regla congelada de T01. Además, importToken conserva un hash del contenido de confirmación sin requestId y el PaperView originalmente confirmado: el mismo contenido con otro requestId devuelve ese resultado, y el contenido distinto da Conflict. El payload original canónico se conserva separado de metadata normalizada; no asumir que dos inputs diferentes pasan a ser equivalentes por normalizar DOI.

## Representación durable propuesta para fijar antes de dispatch
Inspección puntual de 0001 confirma request_id, payload_hash, metadata_json, result_json y error_json; no hay columna session_id ni CANCELLED. No cambiar 0001.

- En metadata_json, al preparar confirmación: objeto interno versionado `{version:1, kind:"confirmation", normalizedMetadata, receipt:{command, payload}, duplicateResolution}`. command viene del registry existente; payload conserva el input exacto necesario para canonical_hash/with_receipt, según la convención T01. request_id guarda la petición de confirmación pendiente; payload_hash guarda el hash del contenido ligado al token. result_json conserva el resultado original confirmado.
- Cancelación: prepare_cancel guarda FAILED con error_json interno `{version:1, kind:"cancellation", code:"OperationCancelled", requestId, receipt:{command,payload}, cleanup:"PENDING"}`; preserva ConfirmIntent si existía. cleanup_owned solo elimina recursos con propiedad inequívoca y ninguna referencia documental vigente. commit_cancel deja cleanup DONE y confirma receipt/auditoría de cancelación en una TX. Si cleanup falla, no publica receipt exitoso; recovery incluye FAILED con cancellation PENDING. No eliminar target ambiguo.
- libraryId es durable; sesión/admisión de token son un registro en memoria. No persistir paths de origen adicionales ni exigir releer el original para terminar una confirmación autorizada.

Estas formas JSON son una **decisión pendiente de Sol**, no un contrato ya congelado. Sol debe definirlas en brief/documento interno antes de que Luna las codifique. Distinguir el hash del receipt del hash por token evita imponerle al worker una semántica accidental.

## Recuperación interna y extensión T04
desktop llama reconcile_imports antes de habilitar operaciones normales, bajo exclusividad de recuperación. Lista intenciones, ejecuta inspección/hash fuera de TX y usa los mismos helpers de aplicación y commit_confirmation/commit_cancel.

Una intención ya confirmada por el humano y durable, con IDs/metadata/payload conservados y archivo inequívoco, puede terminar automáticamente bajo el requestId original; no depende de autorizar el token viejo para una nueva llamada UI. Recovery no inventa metadata ni decisión reuseExisting. Un STAGING sin intención confirmada no se autoimporta: conserva datos y comunica recovery_required seguro; un incompleto inequívoco puede cancelarse siguiendo DATA. Ambiguos y COMMITTED sin archivo legible quedan como issues y nunca ocasionan borrado de ficha/archivo ajeno.

CONTRACTS permite renovar tokens, pero LibraryApi no ofrece comando de reautorización/renovación ni consulta de imports pendientes. **No se implementa esa UX/API por deducción.** El mínimo T02 termina automáticamente solo intenciones ya confirmadas; informa los restantes pendientes al arranque mediante el estado/reporte interno existente. Sol debe explicitar esta limitación y cómo la UI representa recovery_required antes de exigir reanudación interactiva. El informe usa operationId/código/estado seguro, sin paths ni contenido.

T02 mantiene processingInitialized=false y activePhaseCode=null. T04 amplía confirm_in_tx para llamar al caso de uso transaccional initialize_processing(tx, ...) del módulo Workflow antes de COMMITTED/resultado/receipt, usando su puerto declarado y la misma Transaction. Sol asigna explícitamente ese cambio y su composición entonces; no añadir un placeholder Workflow ahora. Un fallo Workflow revierte también Paper/Document/receipt; los replays previos no inicializan ni modifican Paper existente. Reader/Workflow consumen interfaces de aplicación Library, nunca el repositorio SQL privado.

## Corrección propuesta del brief
Sustituir “Service/API async usa DbActor; repositorio SQL recibe Transaction” por:

> LibraryService es asíncrono y depende únicamente de dominio, NativePdfSelection, DocumentStore y LibraryPersistence. SqliteLibraryPersistence implementa el puerto, usa DbActor y abre UoW/with_receipt; llama helpers transaccionales de aplicación con LibraryRepository. Repositorios y helpers reciben &rusqlite::Transaction como excepción pragmática autorizada, sin commits propios. Selección/copia/hash/promoción/inspección se ejecutan en adaptadores background fuera de TX. lib.rs/desktop compone concretos bajo ownership Sol. T04 incorporará initialize_processing en el helper de confirmación bajo esa misma TX.

Mantener la regla application→dominio/puertos y registrar la excepción Transaction de forma explícita. Añadir los ownership/formatos/limitación de recuperación indicados. No cambiar los ocho métodos, envelopes ni DTOs v1.

## Tradeoff y evidencia
El puerto específico requiere wrappers por operación y helpers transaccionales; a cambio deja un único dueño de reglas y un único commit verificable sin exponer Connection/DbActor. Un executor genérico sería más pequeño en líneas iniciales, pero introduciría política/closures de infraestructura en los servicios y otro framework; mover toda la aplicación al adaptador escondería el acoplamiento. Se elige el puerto específico.

Lecturas: AGENTS, INTENT, STATUS, WORKFLOW, brief T02/LibraryApi, ARCHITECTURE dependencia/puertos, DATA import/UoW/receipts y firmas del reporte T01. Comprobación adicional exclusivamente del DDL import_operations/operation_receipts; no revisión del diff T01. MEMORY.md no tuvo coincidencias relevantes. Aplicado Karpathy; using-superpowers declara exclusión para subagentes despachados.

Cambios: únicamente este informe. Commits/merges/tests/instalaciones: ninguno, por alcance. No se demuestra aquí compilación, seguridad del picker ni comportamiento de recuperación. Autorrevisión: ocho métodos contados, cada puerto tiene productor/consumidor, Transaction conservada, shared files asignables, decisiones JSON y ausencia de UX de reautorización explícitas.


## Resolución del coordinador — vinculante para T02

Se adopta la estructura mínima de tres puertos y los helpers transaccionales de este informe. Las secciones anteriores son la propuesta conservada para trazabilidad; las decisiones siguientes resuelven sus pendientes y prevalecen sobre su estado inicial.

1. ConfirmIntent usa exactamente la envoltura versionada propuesta en metadata_json: version=1, kind=confirmation, normalizedMetadata, receipt={command,payload}, duplicateResolution. El payload conserva entrada canónica original, separado de metadata normalizada. IDs/rutas/hash reservados utilizan columnas ya existentes. No modificar 0001. payload_hash de import_operations representa contenido de confirmación sin requestId; el hash del receipt se calcula desde su payload completo canónico según la convención T01. result_json conserva PaperDto originalmente confirmado para replay de token. Validar el JSON interno y rechazar versiones desconocidas con incidencia de recuperación, sin adivinar metadatos.
2. Cancelación usa FAILED y error_json versionado kind=cancellation/code=OperationCancelled con intención/receipt y cleanup PENDING/DONE. Solo se registra éxito después de cleanup inequívoco y commit de receipt/auditoría. Fallos quedan recuperables; no borrar archivos ambiguos ni referenciados. Cancelar no vuelve a borrar ni modifica Paper ya confirmado: esa petición devuelve Conflict salvo replay válido de cancelación anterior.
3. Recovery interno puede terminar intenciones confirmadas durables bajo requestId original; no necesita conceder a una UI nueva el token de una sesión anterior. STAGING sin confirmación no se autoimporta. Incompletos inequívocos sin destino pueden cancelarse con resultado registrado conforme a DATA; una copia completa pendiente de decisión se conserva, con recoveryRequired. No inventar metadatos, decisión de duplicados ni un comando de renovación. No se promete reanudación interactiva de drafts anteriores en v0.1. La UI T03 debe mostrar el aviso de pendientes; no presentar recuperación total mientras persista una incidencia. Un pendiente aislado no vuelve readonly todos los papers ajenos: se bloquea su operación afectada y se mantiene diagnóstico explícito. La raíz se conserva para recuperación manual cuando el caso es ambiguo.
4. Todos los comandos públicos que persisten estado usan receipts conforme a CONTRACTS, incluido selectPdf cuando publica su staging/preview. Repetir requestId devuelve resultado original, aunque un token de una sesión anterior ya no autorice nuevas mutaciones. Un nuevo selectPdf usa requestId nuevo y selector nativo; nunca renueva silenciosamente la autoridad de un token viejo. El usuario ve cancelación o invalidez de token segura y puede iniciar una selección nueva. Esta regla no evita completar una intención confirmada mediante recovery interno.
5. El selector nativo utiliza tauri-plugin-dialog=2.8.1 exclusivamente desde Rust, según docs/development/NATIVE_PICKER.md; sin paquete JS ni permisos genéricos dialog/fs para React. Composición y lockfile se transfieren al worker T02. Verificar denegación de plugin:dialog|open desde frontend y comportamiento permitido del comando propio.
6. Las pseudofirmas fijan responsabilidades y estados, no obligan a construir un framework Async. Reutilizar la convención asíncrona aprobada en T01; concretar records internos desde las columnas/DTOs existentes. El servicio coordina puertos; el adaptador SQLite posee DbActor; helper transaccional aplica reglas bajo la UoW explícita. T04 añade initialize_processing dentro de ese mismo commit, sin placeholder anticipado.

7. Precisión de ubicación tras revisión T02 (2026-10-02): declarar LibraryRepository en application/library_ports.rs y los helpers *_in_tx en application/library.rs; SQL en adapters/sqlite/library_repository.rs. Así aplicación no depende de módulos concretos. modules/library/repository.rs puede reexportar por compatibilidad si existe consumidor real, sin wrappers duplicados ni capa vacía. Se mantiene la excepción explícita Transaction prestada: helpers sin actor/adaptador/commit propio; el wrapper de persistencia posee UoW/receipt y el consumidor cruzado presta su propia transacción. Esta precisión sustituye la ubicación inicial del trait en la tabla, sin cambiar responsabilidades ni IPC.

Tradeoffs aceptados: wrappers de persistencia por caso de uso y JSON interno validado a cambio de una frontera clara y recuperación durable sin migración adicional; drafts no confirmados se conservan como pendientes cuando no pueden cancelarse inequívocamente. Esta limitación es explícita, no un resultado de recuperación demostrado. Contratos IPC v1 y los ocho métodos LibraryApi permanecen iguales.

8. Corrección 2: prepare_import pertenece también al helper de aplicación: estado, expiración, payload, duplicados/reuse y reserva de IDs se deciden allí sobre primitivas de repositorio. Los helpers públicos validan metadatos por sí mismos y emiten auditoría de entidad dentro de la Transaction prestada, incluida su reutilización por Workflow. Se autoriza contexto interno mínimo de auditoría/requestId y primitiva de repositorio audit; no un bus de eventos. La auditoría de ejecución de with_receipt permanece distinta y el replay no duplica el cambio de entidad.
9. Fallos de staging usan la variante interna stageFailure de MANAGED_FILES, con prueba de limpieza separada del error original. No se cambia 0001 ni wire; el helper registra FAILED/DONE solo con prueba suficiente, preservando PENDING cuando no la tiene.
