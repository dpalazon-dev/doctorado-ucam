# T03 — preflight de interacción con duplicados

Fecha: 2026-10-02. Agente: `/root/duplicate_ui_contract`. Estado: **DONE_WITH_CONCERNS**: propuesta documental terminada; Sol debe adoptar la aclaración y asignar el delta backend antes de dispatch T03. No se implementó producto, no se ejecutaron tests y no se revisó WIP.

## Base y evidencia

Producto examinado exclusivamente mediante `git -C .worktrees/task-02-library show 922a0669d771d6778cd3d264bcc5be91f0e96ec6:<ruta>`. BASE/HEAD de esta lectura: `922a0669d771d6778cd3d264bcc5be91f0e96ec6`; ningún commit propio.

Fuentes normativas leídas: `INTENT.md`, `docs/STATUS.md`, `docs/architecture/CONTRACTS.md` (sobre/error, DTOs y LibraryApi), `docs/architecture/SPECS.md` (REQ-002-03), `docs/plans/tasks/task-03-brief.md` y `docs/plans/TASK03_BOUNDARIES.md`. Método: `docs/development/WORKFLOW.md`, Karpathy y Superpowers verification-before-completion. No se consultaron documentos personales, archivo histórico ni memoria auxiliar.

Evidencia del corte congelado:

- `src-tauri/src/adapters/sqlite/library_repository.rs:327`: `preview` consulta candidatos, pero pasa DOI `None`; el DOI manual llega después en `confirmImport`.
- Mismo archivo, `prepare_import:380`, rechazo `:475`: usa DOI normalizado/hash para detectar candidatos y devuelve `DuplicateDecisionRequired` sin `details`. El rechazo inicial sucede antes de ligar una intención nueva.
- Mismo archivo, `:401`: una intención de confirmación ya ligada compara el hash del payload; cambiar `duplicateResolution` da `Conflict`, aunque se cambie el `requestId`.
- `src-tauri/src/modules/library/service.rs:266-308`: preparación, promoción y commit son etapas distintas; otra importación puede ganar entre ellas.
- `src-tauri/src/application/library.rs:110-111`: la comprobación final detecta la carrera mediante `has_duplicate` booleano y devuelve el mismo error sin candidatos.
- `src-tauri/src/adapters/sqlite/library_repository.rs:1002`: la búsqueda textual de lista cubre título/venue/autores; no constituye búsqueda exacta por DOI/hash.

La discrepancia real es una información de interacción ausente: REQ-002-03 exige poder abrir el existente también ante DOI introducido después del selector o duplicado tardío. El error actual del corte no entrega su ID. Esto no califica el WIP ni reabre hallazgos anteriores.

## Propuesta mínima de contrato

Conservar `contractVersion=1`, firmas IPC, `IpcErrorDto.details?: JsonObject` y `DuplicateCandidateDto`. Añadir a CONTRACTS, previa decisión de Sol:

> Cuando `confirmImport` devuelve `DuplicateDecisionRequired`, `error.details.candidates` contiene una lista no vacía de `DuplicateCandidateDto` con las coincidencias por DOI normalizado y/o SHA-256 detectadas en la transacción que rechaza la confirmación. Cada `paperId` aparece una sola vez y `reasons` contiene todas sus razones de coincidencia, sin repeticiones; se usa orden determinista por `paperId` y orden de razones `doi`, `sha256`. No incluye rutas ni contenido del PDF. El error no acredita una importación confirmada y no autoriza cambiar una intención ya ligada.

Ejemplo de `details`, sin tipo wire nuevo:

```json
{
  "candidates": [
    {
      "paperId": "00000000-0000-4000-8000-000000000001",
      "title": "Paper sintético",
      "reasons": ["doi", "sha256"]
    }
  ]
}
```

`retryable=false`: requiere decisión humana, no reintento automático. Mensaje seguro sugerido: «El PDF o el DOI coincide con un paper de la biblioteca. Abre el existente o cancela la importación». UI valida la estructura de `details.candidates` contra los campos del DTO existente antes de ofrecer selección; details ausentes/inválidos muestran error recuperable y permiten cancelación, sin reconstruir candidatos desde la lista. No es necesario añadir una marca de etapa: la interacción siguiente es uniforme.

Backend obtiene candidatos desde una única consulta SQLite parametrizada por DOI normalizado nullable y hash: preview usa DOI `None`, preparación y comprobación final usan el DOI normalizado. Exponer por el puerto interno `LibraryRepository` los candidatos existentes en lugar del resultado booleano permite a `confirm_in_tx` construir el error desde su propia TX. Un helper de `AppError` puede serializar el DTO existente en `details`; no se añade DTO, IPC, SQL en aplicación/UI ni capacidad filesystem.

No fijar un máximo de dos candidatos como restricción de esquema: DOI es único, pero el índice de hash del corte no lo es. Devolver coincidencias completas y distintas desde el backend evita escanear/paginar la biblioteca en React.

## Decisión de interacción recomendada

Usar el mismo flujo para candidatos del preview, rechazo inicial por DOI y duplicado aparecido tras prepare/promote:

1. Mostrar cada candidato con título y razones en español; seleccionar explícitamente si son varios. Ofrecer «Abrir existente» y «Cancelar importación». No elegir automáticamente cuando DOI y hash remiten a papers diferentes.
2. «Abrir existente» ejecuta `cancelImport({requestId: nuevo, importToken})` y espera éxito durable de cancelación/cleanup. Después invoca `onOpenPaper(candidate.paperId)`, que conduce a `ReaderApi.openPaper` con otro `requestId`. No llama `confirmImport` con un payload cambiado.
3. «Cancelar importación» usa la misma cancelación durable, sin abrir paper. No cerrar ni anunciar cancelación antes de la respuesta confirmada.
4. Fallo de cancelación conserva diálogo, selección y borrador visible; no inicia apertura ni afirma éxito. Reintento técnico reutiliza el mismo requestId/payload de cancelación. `ImportRecoveryRequired` muestra el aviso previsto por T03, sin prometer reanudación del draft.
5. Tras cancelación confirmada, el token ya no se reutiliza. Si falla abrir el candidato, mostrar el error del lector y permitir reintentar apertura por ID; no volver a importar automáticamente. Abrir un archivado conserva archivo y lifecycle, sin restauración implícita.

Esta ruta satisface la alternativa normativa «se cancela con cancelImport» y el resultado UX «abrir existente», conserva el payload ligado y no sobrescribe metadatos/documento del candidato. Además permite abrir un candidato del preview sin inventar título/metadatos para construir una confirmación.

`reuseExisting` sigue siendo una resolución backend válida para un token aún no ligado, con payload compatible y metadatos válidos. No se elimina ni se cambia su semántica; T03 no necesita inferir cuándo puede emplearla. Elegir `reuseExisting` solamente ante el preview y cancel+open ante errores sería otra opción, pero añade dos caminos y no ofrece ventaja para la UX requerida.

## Ownership e integración

- **Sol:** aprobar la aclaración aditiva de CONTRACTS; reflejarla en el anexo del brief y TASK03_BOUNDARIES antes de T03. Registrar en ledger el delta y su revisión. Este informe no cambia el contrato por sí mismo.
- **Autor T02, una vez cerrado su trabajo activo:** delta pequeño en `adapters/sqlite/library_repository.rs`, `application/library_ports.rs`, `application/library.rs`, `transport/error.rs` y regresiones Library. Reusar consulta y DTO existentes; confirmar que cancelación limpia solamente recursos reservados de la intención, conservando el documento del candidato. El endurecimiento de cancelación ya asignado mantiene su owner; aquí se pide evidencia del recorrido UX, no una revisión nueva del WIP.
- **Autor T03:** `ImportPaperDialog.tsx`, `useLibrary.ts`, tests UI y coordinación con la apertura de Reader. Validar details y estados de espera/error; no tocar SQLite ni inferir candidatos con `listPapers`. Si necesita un decoder común en archivos shared, Sol lo asigna expresamente; no modificar generated/contracts.ts a mano.
- **Revisión:** Rust del delta backend, TypeScript del consumidor y revisión de conformidad REQ-002-03; solo después gate/merge habitual. T02 integrada y este contrato concretado son dependencias de T03.

## Regresiones exigidas antes de declarar la interacción disponible

| Responsable | Caso y evidencia requerida |
|---|---|
| T02 | Preview sin coincidencia de hash; DOI con prefijo/case/espacios ya existente al confirmar: error contiene paperId/título/reason doi y no liga una confirmación nueva ni crea otro Paper. |
| T02 | Mismo paper coincide DOI+hash: una fila con ambas razones; coincidencias en papers diferentes: dos candidatos distintos. |
| T02 | Barrera determinista entre prepare y commit: otra importación confirma el mismo DOI o hash; el perdedor devuelve candidatos desde TX final, no crea Paper/Document y conserva intención/reservas recuperables. |
| T02 | Tras ese error tardío, cambiar a reuseExisting con el mismo token sigue rechazando el payload incompatible; cancelación con requestId nuevo y su replay limpia los recursos propios, no el paper/documento ganador ni su posición. |
| T03 UI | Candidatos de preview y de error DOI/tardío usan cancel+open; apertura ocurre solamente después del éxito de cancelación y usa ID elegido. No calls listPapers para descubrir candidatos ni confirmImport con resolución alterada. |
| T03 UI | Cancelación pendiente/fallida no abre ni cierra como éxito; conserva borrador. Cancelación confirmada seguida de fallo de apertura permite reintentar solo Reader. Details inválidos/ausentes no ofrecen IDs inventados. |
| T03 integrada | Recorrido con backend real de DOI duplicado y carrera determinista; IDs, metadatos, documento, posición y archivo del existente conservados; ningún segundo Paper. Separar evidencia Rust, mocks UI y apertura real. |

## Comandos y límites

Lecturas realizadas con `Get-Content`, `rg`/`rg --files`, `git ... ls-tree -r --name-only <HEAD>` y `git ... show <HEAD>:<ruta>`; selección de fragmentos/lineas mediante PowerShell. Rutas de producto adicionales leídas desde el mismo HEAD: `transport/error.rs`, `application/library_ports.rs`, `modules/library/service.rs`, `adapters/sqlite/migrations/0001_library.sql`. `git status --short` mostró `AGENTS.md` modificado previamente; se preservó.

Única escritura: este informe. Sin ramas/worktrees nuevos, merges, configuración global, consulta OpenViking, navegación web, ejecución del producto ni tests. La propuesta no demuestra la disponibilidad de esta interacción, la cancelación endurecida de T02, el lector ni instalación nativa. No hay conflicto normativo entre cancel+open y LibraryApi; queda la decisión de Sol y la implementación/revisión del delta requerido.

## Addendum — comprobación de promoción normativa C1

2026-10-02, **DONE** para la comprobación documental solicitada por Sol. C1 adoptada por el orquestador; delta backend asignado al autor T02 después de F10 y antes de freeze, sin interrupción de su trabajo activo.

Se leyó únicamente el párrafo nuevo `DuplicateDecisionRequired` de CONTRACTS, sus copias en los briefs T02/T03 y la sección «Interacción con duplicados» de TASK03_BOUNDARIES. Comparación case-sensitive por PowerShell: un párrafo canónico, una copia por brief y `ExactCopy=True` en ambos. `git diff -- docs/architecture/CONTRACTS.md docs/plans/tasks/task-02-brief.md docs/plans/tasks/task-03-brief.md docs/plans/TASK03_BOUNDARIES.md` confirma adiciones en prosa exclusivamente; ningún bloque TypeScript, DTO, firma IPC o contractVersion cambió.

Resultado: coherencia con C1 y REQ-002-03, sin hallazgos abiertos. Candidatos desde la TX también tras promoción, elección explícita, cancelación durable antes de apertura, payload ligado conservado, reintentos y errores separados, sin escaneo de listPapers ni restauración implícita. Los boundaries concretan validación de details y las regresiones backend/UI. No se leyó producto/WIP, no se ejecutaron tests y no se modificaron los documentos normativos; la única escritura adicional es este addendum. Esta comprobación no acredita implementación.

## Addendum — autorización de cleanup para el perdedor promovido

2026-10-02, **DONE_WITH_CONCERNS**: precisión propuesta a Sol; no aplicada a norma ni producto. Esta lectura adicional se limita al corte `922a0669d771d6778cd3d264bcc5be91f0e96ec6` y a MANAGED_FILES/TASK02_PORTS actuales, con las reglas de pins de WINDOWS_DIRECTORY_GUARDS y el apartado de imports de DATA. El addendum anterior aprobó la coherencia documental C1; no verificó que el backend congelado pudiera cancelar efectivamente el perdedor postpromoción.

Evidencia concreta:

- `adapters/documents/store.rs:363-401`, frozen: cleanup devuelve ambiguous cuando el destino existe, antes de verificarlo o eliminarlo. El destino reservado inequívoco requiere una rama explícitamente autorizada.
- `adapters/sqlite/library_repository.rs:654-764`, frozen: prepare_cancel cambia el estado a FAILED y conserva ConfirmIntent/reservas, pero no conserva como dato específico que el estado previo fuera PROMOTED. validate_cleanup_plan verifica identidad/namespace y referencias; su condición no_destination_is_consistent también limita una reserva nueva con destino. Esta lectura identifica el caso normativo necesario, sin revisar las correcciones WIP.
- En el flujo congelado, record_promoted persiste PROMOTED después de la promoción comprobada y antes de commit_confirmation. El duplicado final aparece después de esa frontera durable. La reserva de IDs/ruta/hash y ConfirmIntent están en la fila existente; la única evidencia que se pierde al pasar a FAILED es el estado previo de promoción.
- MANAGED_FILES admite recursos inequívocamente propios y no referenciados, pero establece que hash igual no prueba propiedad. TASK02_PORTS resolución2 exige cleanup durable y conservación de ambiguos/referenciados. Los pins no conceden autoridad de borrado y se conservan.

**Resolución mínima recomendada para MANAGED_FILES/TASK02_PORTS:**

> Para una cancelación humana de una intención nueva ya promovida, prepare_cancel conserva en error_json version1/kind=cancellation el marcador interno promotionConfirmed=true únicamente si la fila actual tenía estado PROMOTED durable, ConfirmIntent válido de creación/no-reuse y las reservas ligadas eran coherentes. Al pasar a FAILED no destruye ConfirmIntent ni las reservas. Una cancelación pendiente reintentada conserva el marcador original validado; nunca lo vuelve a inferir del estado FAILED, del hash o de la presencia del destino. En los demás casos el marcador queda falso/ausente y no autoriza eliminar un destino presente. Recovery reusa y revalida esta misma evidencia. No cambiar esquema0001, DTO wire, comandos o permisos.

El marcador es una afirmación emitida por persistencia desde el estado anterior, no un booleano que pueda enviar React ni una decisión libre de DocumentStore. Puede emplearse previousState=PROMOTED en lugar de promotionConfirmed; no almacenar ambos. Se recomienda promotionConfirmed como único hecho requerido para esta rama. Un legacy FAILED sin prueba de promoción se conserva como pendiente; no se retroautoriza por semejanza de bytes.

**Autorización y ejecución, sin TX durante filesystem:**

1. Bajo exclusiones de requestId/token y permiso de saga ya establecidos, preparar cancelación durable y validar en TX corta el record actual: biblioteca activa, IDs canónicos, staging exacto, destino `documents/<reserved_document_id>/original.pdf`, ConfirmIntent version1 completo/no-reuse y payload/hash ligados, marcador de promoción válido, estado FAILED de esta cancelación, resultado de confirmación ausente. No aceptar COMMITTED ni convertir reuseExisting en propiedad del documento existente.
2. Comprobar cero referencias documentales a las rutas/ID reservados y cero otras intenciones que reclamen esos recursos. Emitir la autorización interna únicamente para ese PDF reservado y para ese plan actual. La ABI interna concreta queda al autor/Sol; no autorizar todos los destinos presentes ni permitir un plan fabricado desde un record no validado. validate_cleanup_plan sigue siendo la frontera de persistencia compartida con recovery.
3. DocumentStore revalida biblioteca/namespace/plan y abre el recurso bajo la cadena/pins de F10. Si target está presente, exigir autorización anterior, staging ausente y tamaño/SHA-256 esperados, comprobados sobre el mismo handle protegido que se marca para eliminación. Borrar exclusivamente original.pdf reservado. Staging+target presentes, hash/tamaño distintos, record ajeno, referencias/claims, JSON desconocido o prueba incompleta conservan el recurso como ambiguo. Un target ausente permite la ruta de ausencia/cleanup de staging ya establecida.
4. Mantener guardas, pins y exclusiones durante el trabajo filesystem. No borrar `.rw-directory-pin`, no eliminar directorios ni usar wildcard/cleanup recursivo. Solo prueba de ausencia/eliminación de ambos recursos declarados permite cleanup DONE y receipt/auditoría en commit_cancel. Fallo parcial conserva PENDING; el replay/recovery verifica lo restante y no vuelve a borrar un recurso reemplazado o referenciado.

Esta autorización concreta usa propiedad lógica durable de una promoción exitosa a una reserva exclusiva, además de hash/tamaño y acceso protegido; hash+ruta reservada solos siguen siendo insuficientes. Estado STAGING con destino presente, incluso con hash igual, no satisface esta rama. No se promete resolver automáticamente el corte anterior a record_promoted ni ampliar la autoridad a un archivo ajeno.

**Regresiones mínimas del autor:** perdedor PROMOTED recibe DuplicateDecisionRequired, cancelación limpia únicamente su PDF y conserva ganador/metadatos/posición/pins; fallo después de guardar FAILED y antes/después de borrar conserva promotionConfirmed y permite mismo-request replay/recovery; destino referenciado/otra intención, staging+target, target mismatch y STAGING+target igual se conservan; legacy/unknown JSON no concede autorización. Son requisitos propuestos, no tests ejecutados.

Conclusión de diseño: C1 cancel→open necesita esta precisión antes del delta de cleanup. No requiere alterar el flujo UI ni permitir reuse con payload incompatible. Sol prepara/adopta la resolución; el mismo autor T02 implementa después de F10, con revisión y gate. Única escritura de este turno: este addendum; no código, WIP, tests ni cambios normativos.

Actualización de coordinación: Sol comunicó la adopción íntegra en la sección final de MANAGED_FILES y en el brief/context de corrección2, y su envío al autor T02. **DONE** para esta aclaración; no se volvió a revisar esa promoción documental ni se acredita su implementación. La revisión normativa independiente queda a cargo del orquestador.
