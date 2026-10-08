# T04a — revisión de seguridad del diseño

Fecha: 2026-10-03. Revisor: `/root/task04a_security_review`.

**Resultado: PASS DE DISEÑO.** No encuentro una contradicción bloqueante en la ABI tipada propuesta frente al baseline. Esto no aprueba código, implementación, pruebas ni merge. Los controles concretos de abajo deben contrastarse con el HEAD final congelado.

## Alcance y evidencia

- Normas leídas: AGENTS.md, INTENT.md, estado vigente de docs/STATUS.md, brief central task-04a-brief.md, sección de DocumentProof de docs/plans/TASK04_PORTS.md y ADR-018.
- Skill aplicada: `C:/Users/david/.agents/skills/security-review/SKILL.md`, acotada a validación de entradas, frontera filesystem y errores seguros.
- Baseline inmutable consultado mediante `git show`: **9188ac5dc4f1cc5ad22347ef1422a2c51d795b1b**. Coincidía con HEAD de `.worktrees/integration` al verificarlo.
- Código contrastado: `src-tauri/src/application/reader_ports.rs`, `src-tauri/src/adapters/documents/store.rs`, `src-tauri/src/adapters/windows/managed_files.rs`. Consulta puntual de nombres/aserciones existentes de regresión en reader_integration.rs y document_protocol.rs.
- No se inspeccionó el WIP del autor, no se ejecutó la aplicación ni tests y no se modificó producto, runner o documentación normativa. No se consultó OpenViking ni se asumió su peer. La memoria local sólo orientó a comprobar normas y resultados por checkout; el dictamen depende de los documentos y el baseline actuales.

Las referencias de líneas siguientes corresponden al baseline, no al futuro diff.

## Razón del dictamen

1. **La pérdida de causa existe.** `store.rs:144-179` recibe AppError de managed_file y usa StorageUnavailable también para fallo de join. `managed_files.rs:581-590` y `707-716` descartan la causa de acceso denegado/sharing. Un mapper colocado en el puerto Reader no podría recuperarla de forma fiable. Conservarla en el fallo inmediato de apertura OS resuelve esa ambigüedad sin extender DTOs.
2. **El límite entre indisponibilidad e integridad está bien definido.** TASK04_PORTS:108-116 sólo permite Missing/AccessDenied/SharingViolation desde aperturas OS; las guardas y el join permanecen Err. `ManagedOpenFailure::Fatal(AppError)` conserva la procedencia incluso cuando el código público es NotFound o StorageUnavailable. No debe existir conversión general de esos códigos a Unavailable.
3. **La adquisición existente permite un núcleo común.** `ManagedDirectory::open_child` concentra secuencia de apertura, pin y guardas (`managed_files.rs:277-302`); `open_file` concentra apertura y comprobaciones del archivo (`333-349`). Ambos puertos pueden compartir ese recorrido sin otra apertura absoluta ni una comprobación previa con exists().
4. **La prueba disponible conserva propiedad.** `ManagedFile` posee el directorio padre (`372-377`), que retiene root, pin y ancestros (`243-250`); `ManagedDocumentReadHandle` conserva el ManagedFile. La extracción no necesita sustituir esos objetos por rutas ni reabrirlos al consumir la prueba.
5. **No se amplía la frontera UI/IPC.** Los tres tipos adicionales son internos; no hay nuevas rutas, permisos, serialización de errores OS ni comandos. T04a deja la composición Workflow para T04b.

## Controles concretos para la revisión del HEAD final

### 1. Apertura del pin y guardas del pin son operaciones diferentes

`open_child` abre `.rw-directory-pin` con `ReadPin/OpenOrCreate` después de abrir/verificar el directorio (`282-289`). Es una apertura OS bajo un directorio retenido: los tres fallos OS explícitos pueden conservar su clasificación según la regla literal del diseño. Las comprobaciones posteriores del pin, metadata, padre o descendencia son siempre Fatal.

Mantener el pin y su política OpenOrCreate. No suprimir su adquisición ni relajar accesos para obtener Available. Una denegación del pin sólo indica que no se adquirió la prueba completa; no demuestra que el PDF sea inexistente ni que haya pasado las guardas pendientes. Esto no requiere cambiar el contrato: conviene que autor y pruebas lo hagan explícito.

### 2. No aplicar el mapper nuevo a la creación del fallback

En no Windows, `open_relative_directory` conserva un `create_dir` cuyo error se transforma íntegramente en StorageUnavailable (`604-608`); el `File::open` posterior es una operación distinta (`610-618`). La regla AlreadyExists→Conflict corresponde a aperturas, no autoriza reclasificar errores de create_dir. Mantener errores legacy de consumidores de creación, root binding y recovery al extraer los núcleos.

Tampoco confundir el exists() histórico de la rama de creación con una autorización para añadir exists() como prueba de disponibilidad. La rama de proof no crea directorios ni usa ese atajo.

### 3. Mantener acceso solicitado y share flags exactos

La apertura Managed pide lectura, escritura y DELETE, compartiendo sólo lectura (`675-683`). Los directorios y pins tienen sus propias máscaras. Available significa adquisición con esta política existente; no equivale a una simple lectura permisiva del PDF. Las denegaciones derivadas de esas máscaras conservan la clasificación OS. Cambiar a File::open/read-only o permitir write/delete sharing alteraría las garantías aunque desaparecieran fallos de acceso.

Mantener también RootDirectory, nombre de un componente, OBJ_DONT_REPARSE y FILE_OPEN_REPARSE_POINT. Los rechazos de reparse explícitos o NTSTATUS genéricos no se incorporan a la lista de Unavailable.

### 4. No degradar fallos de guardas ni panic

Aplicar Fatal en root.managed_root, ManagedRoot::directory, validación de RegisteredDocument, validate_component, metadata/final_path, regularidad, padre, descendencia y tamaño. Una conversión `From<AppError>` sólo sería segura si siempre produce Fatal, sin inspeccionar error.code. No convertir join/panic al enum de indisponibilidad.

El baseline devuelve PathNotAllowed para tamaño registrado fuera de rango (`store.rs:162-164`) e IntegrityFailure para tamaño real distinto (`167-168`). Conservar esta diferencia legacy. Las pruebas no deben exigir IntegrityFailure indiscriminadamente para cualquier entrada relacionada con tamaño.

### 5. Retención y cancelación

La apertura bloqueante debe poseer root, registro y cualquier handle adquirido, sin referencias prestadas al future IPC. El éxito debe devolver el mismo tipo de handle retenido. La extracción no puede trasladar las guardas al consumidor ni devolver sólo el registro.

La conservación de permisos de operación/request durante cancelación corresponde al trabajo admitido de Reader y, posteriormente, de Workflow. El store por sí solo no demuestra esa propiedad. Mantener las regresiones Reader de cancelación/retención; el gate transaccional de Workflow y su lifetime hasta commit pertenecen a T04b, no son un resultado de T04a.

## Evidencia requerida al revisar implementación

| Caso | Resultado de proof esperado | Compatibilidad Reader |
|---|---|---|
| Archivo o directorio intermedio ausente en apertura OS | Unavailable(Missing) | NotFound |
| AccessDenied en apertura OS inyectada | Unavailable(AccessDenied) | StorageUnavailable |
| SharingViolation, mapper y archivo NTFS bloqueado | Unavailable(SharingViolation) | StorageUnavailable |
| NTSTATUS genérico, fallo de join/panic | Err(StorageUnavailable) | Mismo error |
| Guardas inyectadas con NotFound/StorageUnavailable | Err original | Mismo error |
| Biblioteca/ID/ruta inválida, reparse rechazado | Err de la guarda existente | Mismo error |
| Tamaño real distinto del registrado | Err(IntegrityFailure) | IntegrityFailure |
| Apertura correcta y cancelación del llamador Reader | Handle/pins retenidos según propietario del trabajo | Regresión sin cambio |

La denegación inyectada no acredita ACL real. Las pruebas Windows no acreditan ejecución del fallback. Ni este informe ni un gate backend acreditan funcionamiento de la aplicación instalada.

## Comandos de inspección y entrega

Se usaron Get-Content/rg sobre los documentos indicados, `git -C .worktrees/integration rev-parse HEAD`, `git ... ls-tree -r --name-only 9188ac5 -- <tres archivos>`, `git ... show 9188ac5:<archivo>` y `git ... grep -n -E 'cancel|abort|retained|sharing|reparse|metadata|pins' 9188ac5 -- src-tauri/tests/reader_integration.rs src-tauri/tests/document_protocol.rs`. Comandos de inspección completados con exit 0. Ningún test/build/audit de dependencias fue ejecutado: no corresponde a este dictamen de diseño acotado y no hubo cambios de dependencias.

Único archivo escrito: este informe central. Sin commits, merges ni cambios globales. Autorrevisión: conclusiones limitadas al diseño y al código inmutable; los puntos de atención no se presentan como vulnerabilidades demostradas en una implementación todavía no revisada.

**Entrega: DONE — PASS DE DISEÑO, pendiente revisión independiente del código final congelado.**
