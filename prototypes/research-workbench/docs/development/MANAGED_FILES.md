# Archivos administrados y operaciones admitidas en Windows

Resolución T02, 2026-10-02, ADR-016. Corrige los huecos encontrados en la revisión de 45c4c8c; no certifica todavía su implementación.

## Propiedad y acceso

Un hash igual no prueba propiedad. Los records deben pertenecer a la biblioteca activa y usar IDs canónicos y el namespace exacto derivado de la operación/documento. Antes de emitir un plan de cleanup, la persistencia comprueba que el recurso no está referenciado por un documento vigente ni por otra intención. Una discrepancia se conserva como incidencia.

La composición entrega a DocumentStore tanto la raíz ligada como `expected_library_id` de `actor.info().library_id`, obtenidos del mismo actor. Ambos quedan privados e inmutables. La identidad del record se compara con esa identidad activa; no se vuelve a leer el manifiesto ni se acepta un ID enviado por UI como autorización.

DocumentStore centraliza la protección Windows en un adaptador privado. Retiene los handles del archivo y de la cadena de directorios necesaria para que la ruta no pueda sustituirse mientras actúa. Verifica atributos reparse, identidad y ruta final bajo la raíz autorizada. La comprobación no termina en un PathBuf que se vuelve a abrir sin protección. El mismo handle verificado sirve para lectura/hash/probe y para marcar eliminación. Una promoción no reemplaza atómicamente un destino existente, incluso si aparece después de una comprobación previa.

Se autoriza `windows-sys = "=0.61.2"` como dependencia directa Windows, versión ya presente en Cargo.lock, con features mínimas Foundation/Storage_FileSystem. Archivo asignado: `adapters/windows/managed_files.rs`, su export y pruebas. No se añade IPC ni acceso frontend. Las llamadas FFI se encapsulan en funciones seguras con handles RAII, buffers alineados y longitudes comprobadas; cada bloque unsafe explica sus invariantes. Esto no autoriza mmap ni unsafe en el parser PDF. Si una garantía no puede establecerse, se devuelve error seguro y se conserva el recurso.

LibraryRoot::at conserva la especificación de ubicación sin I/O. El arranque del actor liga una vez la raíz existente y retiene su identidad/guardas antes de probe o apertura de SQLite; una raíz nueva se crea con padres protegidos y luego se liga. El handshake comparte esa misma raíz ligada con composición/DocumentStore, mediante accessor interno del actor; no se vuelve a resolver la variable de entorno ni se autoriza otra raíz desde Store. El Store rechaza una raíz sin ligar. La adquisición de guardas sobre una raíz existente no crea recursos ni altera la rama de diagnóstico para esquema futuro, que permanece sin lock de escritor, migración, staging o manifest nuevos.

La solución concreta usa los controles de compartición y flags de apertura de [OpenOptionsExt](https://doc.rust-lang.org/std/os/windows/fs/trait.OpenOptionsExt.html), comprueba el objeto abierto mediante [GetFinalPathNameByHandleW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew) y realiza operaciones sobre ese handle con [SetFileInformationByHandle](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-setfileinformationbyhandle). Para promoción, [FILE_RENAME_INFO](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_rename_info) permite rechazar un destino existente con ReplaceIfExists=false. Retener los padres estables es parte de la decisión del proyecto; comprobar solo el handle final no protege una apertura previa por una ruta sustituible.

El payload se sincroniza antes de promoción y la intención durable permite reconciliar el estado tras reinicio. Los fallos de sync no se ocultan como éxito. Si Windows no ofrece una barrera de directorio aplicable, se documenta esa limitación en lugar de ignorar su resultado o afirmar durabilidad ante corte de alimentación. Las pruebas de reinicio no certifican hardware ni pérdida de energía.

## Vida de una operación

La promoción puede usar `FileRenameInfo` con `RootDirectory=null` y destino absoluto derivado del padre cuya ruta final se ha verificado. La cadena de padres permanece retenida y protegida hasta terminar; `ReplaceIfExists=false` conserva la exclusión atómica. La garantía depende de esas guardas, no de que el campo `RootDirectory` sea no nulo. La variante relativa ensayada devolvió error Windows 87; no se generaliza ese resultado a todas las invocaciones posibles de la API. Deben verificarse sustitución de padres y aparición de destino sin pérdida de bytes.

El accessor `pub fn DbActor::library_root(&self) -> &LibraryRoot` es una interfaz Rust de solo lectura, también utilizable por pruebas de integración externas al crate. Comparte la raíz ya ligada y sus guardas; no permite reautorizarla ni constituye un comando IPC.

Una mutación Library admitida pasa a ser un trabajo de aplicación propietario de toda la saga, de sus permisos de mantenimiento y de sus exclusiones de requestId/token. El caller espera el resultado; descartarlo no aborta ni libera el trabajo admitido. No se expone un AbortHandle ni se introduce un scheduler o executor genérico. Puede usarse un task Tokio propietario y el canal de resultado existente del JoinHandle. El cierre espera al propietario hasta estado terminal/recuperable y antes de cerrar actor o liberar lock.

Antes de picker, copia, promoción o limpieza se adquiere exclusión por requestId y se comprueba command/payload/receipt. Una petición compatible espera y reproduce; una incompatible falla antes de efectos externos. Orden de adquisición consistente: requestId antes de token. Los registros de exclusión se liberan/prunan sin permitir dos propietarios para la misma clave. No se modifica 0001 ni se mantiene una transacción abierta durante filesystem.

## Recuperación y diagnóstico

Antes de cleanup, cancelación y recovery usan una validación transaccional breve del plan: fila/estado/identidad/namespace actuales y ausencia de referencias documentales o de otras intenciones. Se admite el método interno `LibraryPersistence.validate_cleanup_plan(&CancelPlan)` para compartir esta comprobación. El trabajo propietario retiene sus exclusiones hasta terminar filesystem; no mantiene la transacción abierta durante I/O. Recovery de arranque termina antes de admitir mutaciones. Un plan obsoleto se rechaza conservando el recurso.

El resultado seguro de recovery, incluidos los fallos globales, queda retenido en el estado de aplicación consultado por Settings. `recoveryRequired` combina ese estado con pendientes durables, incluidos COMMITTED con recurso ausente/ambiguo. Una incidencia aislada no impide trabajar con papers ajenos. Si falla globalmente la enumeración/reconciliación y no puede establecerse un estado seguro, Library no admite nuevas mutaciones hasta un arranque/reconciliación satisfactorios; Settings permanece disponible con diagnóstico. No se añaden campos ni comandos IPC.

Cleanup DONE requiere prueba de ausencia o eliminación inequívoca. Fallos o recursos de propiedad incierta quedan PENDING/con incidencia, también si stage falló antes de producir hash. Recovery reuseExisting aplica la misma limpieza verificada que la ruta normal. Nunca se rellena una prueba de limpieza a partir de un error genérico.

Corrección 2: el resultado interno de staging conserva error y prueba de cleanup como datos distintos. En `error_json` se admite la variante interna `{version:1,kind:"stageFailure",code:<ErrorCode seguro>,cleanup:"PENDING"|"DONE"}` sin migración ni cambio IPC. Solo ausencia/eliminación comprobada permite FAILED/DONE; una cancelación humana no se inventa a partir de un rechazo de PDF. Recovery puede reconciliar pendientes mediante las mismas comprobaciones de propiedad y ausencia. Un fallo al persistir conserva el diagnóstico recuperable y nunca se presenta como importación exitosa.

Los hashes de inspección, promoción y cleanup usan streaming con buffer pequeño fijo sobre el mismo handle; el snapshot completo se reserva exclusivamente para el parser bajo su gate. Todas las mutaciones Library, incluidas metadatos y archive/restore, participan en la exclusión global por requestId antes de efectos o receipts.

Resolución F10: la compartición Windows no restringe FILE_WRITE_ATTRIBUTES. [WINDOWS_DIRECTORY_GUARDS](WINDOWS_DIRECTORY_GUARDS.md) fija adquisición relativa, directorio mantenido no vacío mediante hijo retenido, revalidación por handle, bootstrap sin escrituras para esquema futuro y pins internos. Sus reglas prevalecen sobre la descripción inicial de guardas basada solo en sharing. El destino absoluto se admite únicamente tras adquirir esta protección completa; implementación y revisión siguen pendientes.

## Regresiones exigidas

Pruebas sintéticas deterministas: sustitución de padre/archivo y aparición de destino entre validación y operación; referencias vigentes con hash coincidente; caller descartado mientras filesystem continúa; dos peticiones con igual requestId y payload compatible/incompatible; fallo de limpieza y de recovery global; COMMITTED sin recurso. Ninguna prueba usa bibliotecas personales ni requiere modificar configuración global.

## C1 — cancelación de destino propio promovido

Una cancelación humana puede retirar el destino reservado de una importación rechazada al detectar un duplicado tardío. La autorización requiere estado anterior durable PROMOTED y ConfirmIntent version1 íntegro, no reuseExisting, con IDs, payload y hash ligados. prepare_cancel conserva en error_json de cancellation `promotionConfirmed:true` al pasar a FAILED; en otros casos es false/ausente. Los reintentos de la misma cancelación preservan ese dato y recovery lo revalida: nunca lo deduce de la presencia del archivo, hash o ruta.

La validación transaccional exige biblioteca, namespace, IDs y record actuales, sin resultado ni receipt exitoso de confirmación y cero referencias Document u otra intención a los recursos. Store solo admite ese destino autorizado, comprueba hash y tamaño sobre el mismo handle con padres/pins retenidos y exige staging ausente cuando existe destino. STAGING con destino, ambos archivos presentes, JSON desconocido, recurso ajeno/referenciado o discrepancia se conservan como ambiguos. No basta hash igual para conceder propiedad.

Cleanup solo elimina original.pdf declarado; conserva pins y directorios. DONE y receipt de cancelación se confirman únicamente al comprobar ausencia de ambas rutas declaradas. El caso de ausencia ya comprobada es idempotente; fallo de eliminación conserva PENDING y recurso. Probar cancelación/replay tras carrera postpromote real, reapertura de cancellation PENDING y controles negativos sin promoción durable o con referencia vigente. Precisión interna de C1/ADR-016, sin migración, DTO ni IPC nuevo.
