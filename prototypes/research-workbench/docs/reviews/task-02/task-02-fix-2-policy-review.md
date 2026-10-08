# T02 fix2 — preflight normativo y revisión del probe Windows

## Preflight normativo

**PASS documental acotado.** Leídos únicamente los tres párrafos nuevos de `docs/development/MANAGED_FILES.md:39`, `:41`, `:43` y resoluciones 8/9 de `docs/plans/TASK02_PORTS.md:112`. Sin contradicción concreta ni dispensa del defecto F10.

F7 separa error seguro de staging y prueba de cleanup; permite DONE exclusivamente con ausencia/eliminación comprobada, conserva PENDING sin ella y distingue rechazo del PDF de cancelación humana. El fallo de persistencia no se convierte en éxito. Esto responde al STAGING huérfano identificado, pero no aprueba el resultado futuro: habrá que comprobar transmisión de la prueba, persistencia terminal y reapertura, además del caso con cleanup fallido.

La resolución 8 mantiene preparación y reglas dentro del helper transaccional, exige validación de metadatos en el helper público y auditoría de entidad en la misma Transaction prestada. El contexto interno mínimo de requestId/auditoría y la primitiva audit no añaden wire ni un bus. La distinción con auditoría de ejecución de with_receipt es coherente; replay no puede duplicar el cambio de entidad.

Hash streaming de inspección/promoción/cleanup y snapshot exclusivo del parser bajo gate son coherentes con la separación de recursos. La exclusión global por requestId incluye las mutaciones de metadatos/archive/restore. F10 permanece explícitamente abierto: ni retirar SHARE_WRITE ni proteger rename acredita protección de FILE_WRITE_ATTRIBUTES/reparse.

Este PASS corresponde a esas resoluciones documentales, no a implementación, suites, integración ni garantías Windows nuevas. No se reabrieron hallazgos cerrados.

Precisión adicional leída por encargo: `docs/plans/TASK03_BOUNDARIES.md:15`. El namespace global de receipts exige un único registro por requestId, compartido e inyectado una vez al introducir Reader y reutilizado por consumidores futuros. La regresión cruzada Library/Reader propuesta conserva exclusión antes de efectos. Es coherente con la resolución actual y no amplía la implementación T02 a Reader.

## Probe F10

**Fuente recibida; ejecución PENDIENTE.** El arquitecto entregó `work/research/windows-handles/HandleProbe.cs` y `run-probe.ps1`, leídos completos. Todavía no se recibió ruling final ni log de ejecución. El script crea una fixture UUID sintética bajo su directorio y conserva la evidencia, sin borrado recursivo de junctions. No ejecuté el probe.

**Concern importante de evidencia, comunicado antes de ejecución:** las aserciones de creación/rename sobre raíz convertida aceptan cualquier NTSTATUS negativo como PASS, y la colisión solo exige `Rename < 0`. Sin controles positivos de ambas llamadas en raíz ordinaria usando iguales derechos/options/ABI, un error de acceso/buffer/API produciría un falso resultado favorable. Se requieren create relativo normal exitoso y rename a destino libre exitoso, verificando ruta final y bytes, y colisión con status identificable de nombre existente. Fallo genérico no demuestra no-clobber ni apertura segura utilizable.

El control FSCTL usa FILE_WRITE_ATTRIBUTES exclusivamente y exige éxito en directorio vacío sin guarda; después intenta el mismo canal con deny SHARE_WRITE. El pin exige ERROR_DIR_NOT_EMPTY=145 y verifica que su hijo retenido no se pueda borrar. Estos experimentos están bien acotados como hipótesis; no hay resultado todavía. No observé defecto concreto en layout/buffers x64 leídos; aconsejé comprobar sizeof/offset ABI en el propio probe (OBJECT_ATTRIBUTES=48, UNICODE_STRING=16, IO_STATUS_BLOCK=16, offset FileName=20) para que errores de marshalling no se confundan con protección. El análisis no certifica ABI por ejecución.

Una solución relativa para DocumentStore no demuestra por sí sola seguridad de SQLite por ruta. El ensayo del pin en un padre tampoco demuestra adquisición sin carreras, protección de toda cadena, persistencia/gestión del pin ni compatibilidad con diagnóstico readonly; esa arquitectura y sus evidencias siguen pendientes. El revisor no autoriza nueva API ni modificación de datos de biblioteca a partir de este borrador.

**Cierre del turno: DONE_WITH_CONCERNS, probe pendiente.** El arquitecto informa que añadió controles positivos, ABI y colisión específica; no se volvió a leer esa versión ni se afirma verificación de las correcciones. Por instrucción del orquestador, la revisión de fuente final se transfiere a code-reviewer antes de ejecución; este revisor retomará con ruling y evidencia exactos. Las observaciones anteriores ya se comunicaron a ambos.

Única escritura asignada: este documento central. Sin producto, Git, suites, subagentes, OpenViking ni contexto externo en este preflight.

## Revisión F10 tras ejecución2 y norma definitiva

**PASS arquitectónico acotado / DONE_WITH_CONCERNS.** La propuesta es suficiente y coherente con la garantía exigida en biblioteca **local NTFS, Windows x64**, después de resolver el gap de DB ausente indicado abajo. Cero Critical/Important abiertos en la norma final revisada. **No es aprobación de producto ni cierre de S1/F10:** quedan implementación y regresiones Rust exactas.

Autoridad revisada: `docs/development/WINDOWS_DIRECTORY_GUARDS.md` completo, incluido paso 3 actualizado; MANAGED_FILES remite a esa norma. El ruling `task-02-windows-handle-ruling.md` es investigación/propuesta histórica: cuando difiere, prevalece la decisión final del coordinador. Se leyó el reporte independiente `task-02-windows-probe-code-review.md`, el ruling final y la evidencia real de ejecución2. Se comprobaron hashes de fuente/runner y log; se releyeron únicamente funciones/adiciones pertinentes del delta2 del probe. Ninguna ejecución propia.

### Evidencia efectivamente comprobada

`work/research/windows-handles/probe-2.exit.txt` contiene 0. `probe-2.log` contiene 31 líneas PASS y ALL ASSERTIONS PASSED; SHA256 `D37CD4C6AFE3AC0575373949C9831D70AA8A71C740D362C169C825A803EA65E8`. Fuente SHA256 `1AD735D72D5719B52B197F073C38425E831E9F503F3FB286EAE27F6A5EAE80F4` y runner `DAC526ADC50E5530DDDE52F86AF4730A36806112187B1CEFE71E02480973BDE7` coinciden con la revisión previa final.

| Experimento | Resultado leído |
|---|---|
| Controles positivos NTcreate/rename relativos | STATUS_SUCCESS, ruta final y bytes correctos |
| FSCTL mount-point en padre vacío, WRITE_ATTRIBUTES exclusivamente | Éxito y junction dirige al marker sintético |
| Retirar SHARE_WRITE del guarda | FSCTL todavía tiene éxito: no protege atributos |
| Padre convertido antes de NTcreate/rename relativo | Ambos C0000280; sin archivos nuevos en destino exterior sintético |
| Pin READ_DATA+ATTRS retenido, no DELETE sharing | FSCTL falla145; borrar pin falla HRESULT80070020; tras cerrar el guarda se borra correctamente |
| Pin únicamente de atributos | Borrado permitido: variante insuficiente |
| Crear directorio/pin relativo y reabrir pin readonly | STATUS_SUCCESS y ruta final correcta |
| Win32 rename absoluto bajo pin | Éxito, padre final correcto y bytes correctos |
| Colisión Win32 no-clobber | Error183, preserva origen y destino |

El concern de falsos PASS del borrador está resuelto por controles positivos, colisiones específicas y control de borrado tras cerrar el guarda. El Minor de fail-fast no ocultó escenarios en esta ejecución: el log llegó a todos. No se convierte este hecho en prueba de carreras concurrentes completas.

### Adquisición, estabilidad y vida

La norma exige adquirir cada componente mediante NtCreateFile relativo, con nombre simple validado, OBJ_DONT_REPARSE y OPEN_REPARSE_POINT; no permite fallback por ruta absoluta. Cada padre mantiene el siguiente hijo/directorio retenido con LIST_DIRECTORY y sin DELETE sharing. El leaf mantiene un hijo con READ_DATA, no un handle de atributos solamente. La revalidación de **ese mismo directorio después de retener el hijo** es esencial: comprobarlo antes y publicar un PathBuf no serviría.

La adquisición relativa cubre el intervalo anterior al pin; el pin/no-vacío y las retenciones cubren el posterior dentro del alcance adoptado. El binding completo se publica únicamente al terminar ambas fases. La retención acompaña clones y jobs admitidos, y el pin raíz no termina antes de cerrar SQLite. Esto permite conservar el rename absoluto ya existente con no-clobber, mientras read/hash/delete siguen en el archivo verificado. La investigación de NtRename se queda fuera de imports de producto.

Comprobación del volumen por handle y rechazo de otros filesystems/red deben fallar conservadoramente ante error, sin probar capacidades escribiendo en bibliotecas. [GetVolumeInformationByHandleW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getvolumeinformationbyhandlew) consulta el filesystem del objeto y documenta falta de soporte SMB; no basta un string de ruta ni FILE_SUPPORTS_REPARSE_POINTS como prueba de la propiedad no-vacío. La norma exige además ancla local y rechazo de UNC. No se extiende la evidencia a drivers/filtros arbitrarios.

### Gap encontrado y resuelto: DB que aparece después de comprobar ausencia

**Important arquitectónico P1 — RESOLVED en norma final `WINDOWS_DIRECTORY_GUARDS.md:17`.** El borrador pedía comprobar otra vez ausencia tras LibraryLock y luego abrir SQLite por ruta. Eso todavía permitía que otra DB apareciera entre comprobación y Connection::open, que podía abrirla como nueva antes de clasificar versión.

La decisión final exige **LibraryLock → NtCreateFile FILE_CREATE atómico → retener el nuevo DB pin READ_DATA/sin DELETE sharing → abrir SQLite**. Solo el éxito de FILE_CREATE autoriza la rama nueva. Colisión devuelve Busy, conserva la DB aparecida y no abre SQLite; el siguiente arranque la abre como existente y clasifica su versión antes de writable setup. La creación nunca precede LibraryLock. Esta precisión elimina la ventana check/use de presencia/identidad; no pretende impedir escrituras externas al contenido de DB fuera del modelo del producto.

Para DB existente, FILE_OPEN relativo readonly adquiere el pin antes del probe; error distinto de ausencia no crea biblioteca. La rama futura no crea sentinel/lock de escritor/staging/manifest ni migra. El documento distingue esa rama del caso de inicialización que encontró ausencia y luego sufrió creación externa simultánea, donde no promete inventario universalmente intacto de artefactos de inicialización. Pruebas exactas de ambas ramas siguen obligatorias.

### Pin reservado, backup/restore y FFI

`.rw-directory-pin` vacío es un recurso interno sin autoridad ni contenido interpretado. Abrirlo existente no trunca y verifica tipo/no-reparse. Se crea solamente en init autorizado sin DB o directorios administrados writable; nunca en ancestros personales ni rama de esquema futuro. Cleanup se limita al recurso PDF declarado y conserva pins/directorios; no usa comodines. Que staging termine y conserve su pin está declarado, sin purga automática implícita.

Backup/export omite pins; restore soportado los regenera al adquirir la nueva raíz. El manifiesto no los convierte en evidencia de propiedad. Switch/restore drena jobs y conexión, libera todas las guardas antiguas antes de renombrar esa raíz y conserva recuperación de la anterior. Esta política es coherente; no afirma implementación ni prueba actual de T04. El nuevo binding debe estar completo antes de admitir I/O en la raíz restaurada.

Bindings autorizados se limitan a NtCreateFile privado en windows-sys0.61.2 y Wdk_Foundation/Wdk_Storage_FileSystem/Win32_Security/Win32_System_IO necesarios para estructuras. Componentes/derechos/disposiciones cerrados, longitudes UTF-16 comprobadas y memoria/handles vivos hasta terminar la llamada síncrona. Win32_Security no autoriza ACL; no NtSetInformationFile de producto, VFS nuevo ni unsafe en parser. La propuesta no necesita ampliar el wire.

### Condiciones de cierre de producto y límites

El autor debe demostrar en Rust adquisición atacada antes del pin sin escritura exterior, FSCTL WRITE_ATTRIBUTES con control positivo y rechazo145 protegido, rechazo de DELETE y control tras cierre, cadena completa/vida de clones, promoción no-clobber, cleanup que preserve pin, FILE_CREATE postlock con colisión real que deje intacta DB aparecida, rusqlite writable/WAL/checkpoint/reopen y futuro esquema con inventario/hashes iguales. La revisión independiente del producto debe comprobar que ningún binding incompleto permita I/O por ruta. Son obligaciones de implementación de la norma existente, no resultados del probe.

**CannotVerify:** integración Rust, SQLite/WAL real con pin, carreras concurrentes completas del bootstrap, restauración real, instalación y pérdida de energía no se han ejecutado/revisado aquí. Tampoco se acredita comportamiento de otros tags, filesystem o filtros. La evidencia C# es válida para sus llamadas/máscaras y el entorno sintético probado. No se usaron datos personales, suites repetidas, subagentes, Git ni cambios de producto. Única escritura: este informe; consulta web adicional acotada a la API de comprobación de volumen.

## Addendum C1 — destino propio promovido, revisión documental

**PASS documental acotado.** Leída únicamente la sección C1 de `docs/development/MANAGED_FILES.md:49`. Cero defectos Critical/Important concretos nuevos en esta regla. Sin lectura de WIP, producto, suites ni nueva evidencia de comportamiento.

La autorización para borrar destino se deriva de una transición desde **PROMOTED durable** con ConfirmIntent version1 íntegro y ligado, no reuseExisting; `promotionConfirmed` conserva esa prueba al pasar a FAILED y durante retries. La norma prohíbe reconstruirla por presencia, ruta o hash del archivo. El flag aislado no sustituye revalidación de intención, identidad/namespace/payload actuales, ausencia de resultado/receipt de confirmación ni exclusión de referencias documentales/de otras intenciones. Esto es coherente con la propiedad exigida anteriormente y distingue una promoción rechazada por duplicado tardío de un documento confirmado.

El Store vuelve a comprobar tamaño/hash del destino sobre el mismo handle y exige staging ausente. Ambos archivos presentes, STAGING con destino o evidencia desconocida/discrepante conservan ambigüedad. La limpieza se limita al original declarado, mantiene pins/directorios y solo confirma DONE/receipt de cancelación con ausencia comprobada de ambas rutas. La idempotencia de ausencia no dispensa las comprobaciones de autoridad del plan. Fallo de eliminación conserva PENDING; no cambia wire ni convierte un rechazo en confirmación exitosa.

**Pendiente para revisión de HEAD congelado:** comprobar que la transición que fija el flag usa el estado previo durable y la intención íntegra bajo TX, que retries/recovery preservan y revalidan el dato, y que no llega al borrado un plan sin las referencias/condiciones exigidas. Regresiones ya requeridas por la norma: carrera postpromote real, replay, reapertura PENDING y controles negativos sin promoción durable/con referencia vigente. Este PASS no aprueba la implementación ni afirma resultados de esas pruebas.
