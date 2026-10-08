# T02 fix 2 — rerevisión independiente de seguridad

Estado: **DONE_WITH_CONCERNS**. SPEC: **NOT APPROVED**. Calidad/seguridad: **NEEDS FIXES**. Hay dos Important de seguridad: F7 y C1 (este último compartido con la revisión general). F3 añade el Important de auditoría confirmado por esa revisión. El resultado consolidado son **tres Important abiertos**, sin Critical ni Minor nuevos; este HEAD no debe integrarse.

BASE `922a0669d771d6778cd3d264bcc5be91f0e96ec6`; HEAD congelado `21fd14ed20ae391518a7dabaaece3dc551a92fa2`; commit de código `3cdeae2bfc936e5447ae9910a8f7bc0ea16a730d`. Checkout `.worktrees/task-02-library`, limpio según contexto del orquestador. Revisor `/root/task02_security_review`; única escritura: este informe. Sin cambios de producto, Git, subagentes ni nuevas ejecuciones.

## Alcance y veredictos

Se leyó el paquete completo `review-922a066..21fd14e.diff` por porciones, contexto/brief/reporte fix2, normativa pertinente del principal y evidencia final existente. El PASS documental anterior de WINDOWS_DIRECTORY_GUARDS y MANAGED_FILES C1 no se tomó como aprobación del producto. El reporte del autor tampoco sustituye evidencia. F1/F2/F4/F5/F6/F8/F12 y M1–M4 permanecen cerrados; solo se examinan correcciones focales y regresiones introducidas por este delta.

| Hallazgo | Veredicto | Motivo |
|---|---|---|
| F7 / S6 | **NOT ADDRESSED completamente** | Los errores tipados y los fallos con residuos quedan registrados, pero un error temprano de apertura del origen declara DONE sin acreditar ausencia del staging declarado (Important 1). |
| F10 / S1 | **ADDRESSED** | Adquisición nativa relativa, cadena retenida, pin READ_DATA sin DELETE sharing, revalidación del mismo handle, comprobación NTFS y creación atómica de DB después del lock. Limitado a Windows x64/local NTFS aprobado. |
| F11 / S2 | **ADDRESSED** | La condición de reserva sin destino se aplica a None; planes con destino conservan namespace, referencias, estado e identidad actuales. El defecto C1 se registra por separado. |
| C1 | **NOT ADDRESSED completamente** | El caso ordinario postpromote funciona; una cancellation de versión desconocida puede convertirse en autoridad válida antes de validar y permitir eliminación (Important 2). |
| F9 | **ADDRESSED en Library** | Mutadores adquieren el registro por requestId antes del permiso de operación y conservan los permisos durante el trabajo. El consumidor Reader compartido pertenece a T03 y no se exige aquí. |
| F13 | **ADDRESSED** | Hash y tamaño se comprueban por el mismo handle mediante lectura acotada, buffer de 64 KiB y límite MAX+1; sin nueva lectura completa no acotada. |
| F3 | **NOT ADDRESSED completamente** | Recovery omite import.cancelled de entidad. Important compartido con la revisión general; no se cuenta como hallazgo nuevo de seguridad. |
| M5 | **ADDRESSED** | El reporte rectifica que construir Store no realiza I/O y que recovery se omite para esquema futuro. |

S3/S4/S5/S7 permanecen cerrados; no se identificó una regresión focal adicional que los reabra.

## Important 1 — F7: fallo temprano del origen declara limpieza terminada sin comprobar el staging

Rutas del checkout: `src-tauri/src/adapters/documents/store.rs:215`–218; persistencia `src-tauri/src/adapters/sqlite/library_repository.rs:469`, `:482`, `:500`; exclusión de recovery `library_repository.rs:947`–950. Consumidor: `src-tauri/src/modules/library/service.rs:194`.

En stage, File::open del origen puede fallar antes de adquirir el directorio administrado o abrir el staging. Ese retorno construye StageFailure con SourceUnreadable y StageCleanup::Done. No inspecciona la ruta staging/{operationId}/source.pdf ni acredita su ausencia. Service entrega esa clasificación al repository, que la persiste como FAILED, kind=stageFailure, cleanup=DONE. recoverable_imports excluye FAILED/DONE. Por tanto, un recurso desconocido presente en el staging queda fuera de reconciliación y no se comunica mediante recoveryRequired, aunque no exista prueba de limpieza.

Escenario determinista para regresión, no ejecutado por este revisor: utilizar el wrapper GatedDocumentStore ya existente para pausar tras begin_import y antes de LocalDocumentStore.stage; seleccionar un origen con nombre válido que ya no pueda abrirse; consultar el operationId y staging_path creados y escribir bytes sintéticos desconocidos en esa ruta declarada; liberar la pausa. File::open del origen falla y el código actual persiste FAILED/DONE sin consultar esos bytes. Al reabrir, la operación queda excluida de recoverable_imports y el archivo permanece. Es el ámbito local sintético de interferencia filesystem ya contemplado por F10, sin acceso frontend nuevo ni datos personales.

Impacto: falsa constancia durable de limpieza y pérdida de la incidencia de recuperación de un recurso desconocido. No se afirma que este caso borre el origen o cree un Paper/Document; su defecto concreto es afirmar ausencia y ocultar el residuo. F7 exige DONE solamente tras prueba de ausencia o eliminación acreditada. La prueba positiva source_unreadable usa un namespace vacío y no cubre este caso.

Remedio mínimo: para retornos anteriores a la apertura del staging, no derivar Done solamente de SourceUnreadable/InvalidInput. Acreditar ausencia mediante inspección nativa del namespace autorizado y su binding retenido, sin crear ni eliminar contenido desconocido; cuando no pueda acreditarse, persistir Pending y dejar que recovery verifique la ausencia o comunique ambigüedad. Mantener el error original. Si el identificador no autoriza un namespace, fallar conservadoramente sin construir rutas libres. El Done procedente de una eliminación exitosa por el handle propio sigue siendo válido.

Regresión necesaria: el escenario pausado anterior conserva los bytes, registra FAILED/PENDING y recovery comunica la incidencia. El caso de origen ausente con staging realmente ausente debe poder reconciliar a DONE sin una incidencia permanente. No requiere otra suite en esta revisión; corresponde al fix3 del mismo autor.

## Important 2 — C1: retry convierte JSON desconocido en autoridad para borrar el destino promovido

Rutas: `src-tauri/src/application/library.rs:90`–94; `src-tauri/src/adapters/sqlite/library_repository.rs:737`–742; consumidor de autoridad `src-tauri/src/adapters/documents/store.rs:486`. Requisito: MANAGED_FILES C1 conserva la ambigüedad de JSON desconocido.

has_durable_promoted_confirmation acepta FAILED cuando error_json contiene kind=cancellation y promotionConfirmed=true, sin validar la versión ni cleanup de esa cancelación previa. prepare_cancel usa ese resultado y sobrescribe el JSON anterior por una cancellation version1/PENDING antes de validate_cleanup_plan. La validación posterior y destination_cleanup_authorized reciben el wrapper nuevo válido, por lo que ya no pueden rechazar la versión original desconocida.

Escenario concreto, coincidente con la revisión general: operación propia promovida, ConfirmIntent v1 íntegra y ligada a IDs/payload/hash, cancelación durable FAILED/PENDING, staging ausente, destino reservado con hash y tamaño correctos, sin referencias de Document/otras intenciones ni receipt exitoso de confirmación. Cambiar únicamente la version de cancellation a 99 conservando promotionConfirmed=true y reintentar con token admitido y requestId sin receipt exitoso. El helper conserva la bandera, prepare_cancel normaliza a version1, el plan posterior pasa y cleanup puede eliminar el destino y publicar DONE/receipt. La decisión vigente exige conservar ese archivo ante autoridad desconocida. El control de hash, namespace y referencias no sustituye la validación de la intención durable previa.

Impacto: eliminación de un archivo declarado cuando la autoridad de cancelación anterior debía tratarse como desconocida. Es un camino destructivo concreto introducido por la conservación de autoridad C1; no se atribuye al hash una propiedad de autorización que no tiene.

Remedio mínimo: validar el wrapper previo dentro de la TX y antes de reescribirlo. En FAILED, preservar promotionConfirmed únicamente desde una cancellation v1 íntegra, PENDING y ligada al request/token/receipt correspondiente. JSON desconocido o inconsistente debe producir ImportRecoveryRequired y conservar wrapper, bytes y ausencia de receipt exitoso. Añadir el negativo version99 sobre el fixture postpromote real. No se duplicó la ejecución de pruebas de la revisión general.

## Important compartido — F3: recovery omite auditoría de entidad

`src-tauri/src/adapters/sqlite/library_repository.rs:1004`/`:1011`, mark_recovered_cancelled, actualiza DONE y receipt dentro de with_receipt pero no llama a audit_change de entidad; el camino normal sí registra import.cancelled en `:934`. El evento de ejecución con entity_id NULL no sustituye ese evento. Reapertura antes del cleanup, por tanto, produce una cancelación exitosa con distinto historial de entidad. Corrección: finalización común o audit_change dentro de la misma TX, exactamente una vez junto a DONE/receipt y con rollback conjunto.

El escenario y regresión se detallan en [task-02-fix-2-spec-rereview.md](task-02-fix-2-spec-rereview.md). Este informe confirma el camino estático y adopta ese Important para el cierre consolidado, sin duplicarlo ni ejecutar otra prueba.

## Garantías F10/F11 acreditadas en el delta

La adquisición Windows privada usa NtCreateFile relativo de un componente con OBJ_DONT_REPARSE y FILE_OPEN_REPARSE_POINT; valida el tipo por el mismo handle. La cadena de directorios retenida impide reemplazo/rename de sus objetos mediante ausencia de DELETE sharing. Cada padre conserva un hijo retenido; los directorios terminales conservan un archivo pin con READ_DATA, READ_ATTRIBUTES y SYNCHRONIZE sin DELETE sharing. Tras adquirir hijo/pin se revalida el padre por el mismo handle antes de publicar el binding. Esto aborda el ataque FSCTL_SET_REPARSE_POINT con FILE_WRITE_ATTRIBUTES sobre padre vacío, para el que quitar FILE_SHARE_WRITE por sí solo era insuficiente.

El producto comprueba volumen NTFS por handle y rechaza rutas UNC/red y otros sistemas de archivos. No se afirma garantía universal para filesystems, drivers o administradores fuera del alcance local NTFS/Windows x64. El helper nativo queda privado, sin NtRename nuevo de producto, permisos frontend ni IPC adicionales. El rename Win32 existente conserva ReplaceIfExists=false; eliminación y comprobaciones del archivo usan su handle retenido, con verificación hash/tamaño y sin comodines sobre directorios/pins.

Para DB existente, se adquiere FILE_OPEN y pin de lectura antes del probe de versión; no se crea sentinela en el diagnóstico de esquema futuro. Para DB ausente, FILE_CREATE se ejecuta después de LibraryLock y conserva el nuevo pin; una colisión devuelve Busy sin abrir SQLite y conserva la DB aparecida. Los pins reservados no se convierten en autoridad de documento ni se eliminan como PDFs. No se exige implementación anticipada de backup/restore T08.

La evidencia de producto incluye el control FSCTL exitoso sobre directorio vacío, rechazo sobre directorio protegido, rechazo de delete del pin y control exitoso tras cierre; también no-clobber, adquisición relativa frente a reparse, colisión atómica de DB, futuro esquema sin creación y SQLite/WAL/reapertura. Estas pruebas de producto se distinguen del probe C# arquitectónico anterior. No se observó un gap concreto abierto de F10 en el delta revisado.

F11 conserva controles independientes de namespace, IDs, referencias y snapshot actual del plan. La condición que impedía cancelar preparados ordinarios con destino Some está corregida. C1 desconocido sigue necesitando la corrección anterior y no queda dispensado por esas garantías de filesystem.

## Evidencia y límites

Se inspeccionaron `work/evidence/fix2-final-check2.log` y exitcode 0: typecheck, Vitest 14/14, build Vite, Clippy y suites Rust finales con 97 entradas, sin filtros que vacíen las suites finales. Se inspeccionaron `fix2-native-debug-build.log` y exitcode 0, que acreditan cargo build nativo debug. La revisión Rust independiente ejecutó sus mínimos, según comunicación del orquestador. No se volvió a ejecutar ninguna suite ni probe.

El verde actual no demuestra el staging desconocido ante fallo temprano de origen, la conservación de cancellation version99 ni el evento import.cancelled en recovery. Los escenarios descritos son deducciones de los caminos revisados, no resultados de una ejecución nueva. Deben corregirse y acreditarse en el nuevo HEAD antes de integrar.

Cannot Verify: selector GUI, Reader/T03, bundle/NSIS, instalación limpia, resistencia a administradores/drivers arbitrarios, otros filesystems/red, durabilidad ante pérdida de alimentación y restauración T08 todavía futura. No se inspeccionaron ni modificaron bibliotecas personales. Estos límites no añaden alcance al fix.

Veredicto final: **DONE_WITH_CONCERNS / NOT APPROVED / NEEDS FIXES**. F10 y F11 corregidos; F7, C1 y F3 pendientes para fix3, con reproducciones y remedios acotados.

Nota editorial del coordinador: corregidas las rutas Store a adapters/documents y la fase backup/restore a T08; hallazgos y veredictos intactos.
