# T02 fix1 — revisión independiente de seguridad

**DONE_WITH_CONCERNS / Needs fixes.** Especificación: cumplimiento parcial. Calidad: tres Important abiertos, cero Critical; no aprobar integración todavía. Revisor `/root/task02_security_review`, lectura del delta `45c4c8c48269495b74241004f92f68acf4770708..922a0669d771d6778cd3d264bcc5be91f0e96ec6`, checkout `.worktrees/task-02-library`.

## Resultado por hallazgo original

| Original / fix | Veredicto | Evidencia y límite |
|---|---|---|
| S1 / F10 | PARCIAL, abierto | Raíz ligada al actor, handles RAII, archivo retenido, borrado por handle y rename `ReplaceIfExists=false` sí implementados. No se acredita estabilidad ante modificación reparse de los padres: K1. |
| S2 / F11 | ADDRESSED para propiedad; regresión Important | Store compara identidad activa, UUIDs y namespace exacto. Cleanup relee fila y referencias bajo TX con exclusión de la saga. El predicado adicional rechaza planes legítimos: K2. |
| S3 / F1 | ADDRESSED | Replay de selección devuelve receipt sin `remember_token`. La regresión con otro servicio rechaza confirmación y cancelación del token reproducido. |
| S4 / F2 | ADDRESSED | `RecoveryStatus` conserva Pending/issues/error global; Settings combina diagnóstico con pendientes y detecta COMMITTED ausente. Fallo global bloquea mutaciones, issues aislados no bloquean toda la biblioteca. |
| S5 / F5 | ADDRESSED para el defecto observado | Segundo read MAX+1 del mismo archivo retenido, comparación de tamaño/hash y serialización del probe. Crecimiento/cambio determinista cubierto. Es límite de entrada, no certificación de heap total del parser. |
| S6 / F7 | PARCIAL, abierto | Se elimina el falso cleanup DONE genérico y se conservan residuos desconocidos. Se pierde la prueba de cleanup exitoso en rechazos ordinarios: K3. |
| S7 / F12 | ADDRESSED con alcance explícito | Test registra dialog real, Library conectado y capability main-local; `plugin:dialog|open` denegado por autoridad. `fs` carece de plugin/handler: evidencia de indisponibilidad, no prueba de ACL de un plugin instalado. |

F8/F9 corrigen riesgos de admisión introducidos en la saga: el job conserva permisos/exclusiones aunque se descarte el caller; requestId se adquiere antes de token y de efectos externos. Las regresiones cubren caller descartado, cierre, recovery, select compatible y cancel incompatible. No se observa en este delta nueva IPC, permiso frontend, ruta enviada por UI ni acceso fs genérico.

## Important

### K1 — S1/F10: los padres retenidos admiten escritura; reparse no queda inmovilizado

**Ubicación:** `src-tauri/src/adapters/windows/managed_files.rs:82`, `:357`, `:308`; comentario de garantía `:5`.

Ambas aperturas de directorio usan `FILE_SHARE_READ | FILE_SHARE_WRITE`. Retener esos handles sin SHARE_DELETE impide rename/delete, pero permite aperturas con escritura compatibles. El rechazo de `FILE_ATTRIBUTE_REPARSE_POINT` y la comprobación de ruta final ocurren al adquirir el padre. Después `rename_no_replace` vuelve a usar una ruta absoluta derivada de ese padre. El comentario que promete negar write/delete sharing no coincide con el código.

**Amenaza/impacto:** un actor local con acceso suficiente puede intentar convertir en junction un padre destino vacío ya validado. Si la modificación prospera antes del rename absoluto, el archivo podría promoverse fuera de la raíz autorizada aunque la operación siga siendo no-clobber. Es una garantía requerida por F10/MANAGED_FILES, no una afirmación de explotación ejecutada.

**Acceso y evidencia primaria:** Microsoft especifica para `FSCTL_SET_REPARSE_POINT` acceso `FILE_WRITE_DATA` **o** `FILE_WRITE_ATTRIBUTES`, y rechaza directorios no vacíos; exige acceso de creación de enlace para el tag SYMLINK. La página Win32 también menciona `SE_CREATE_SYMBOLIC_LINK_NAME`; no generalizo esto a una explotación sin privilegios para todos los tags. [MS-FSA: FSCTL_SET_REPARSE_POINT](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fsa/4aeefef8-92c3-4abc-af7a-a610caf8a165), [Win32 FSCTL](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_set_reparse_point). `CreateFileW` indica que compartir escritura permite nuevas aperturas write y que los flags de compartición no afectan al acceso de atributos/extended attributes. Por tanto, retirar SHARE_WRITE por sí solo no prueba que el canal WRITE_ATTRIBUTES esté cerrado. [CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew).

**Remedio mínimo y aceptación:** corregir la compartición de escritura y establecer protección efectiva frente a cambio reparse durante toda apertura/rename por ruta, conservando raíz y padres ligados, mismo handle de archivo y `ReplaceIfExists=false`. No reemplazarlo por otro `exists` ni por una segunda validación seguida de un rename desprotegido. La regresión debe intentar realmente abrir el padre vacío con FILE_WRITE_ATTRIBUTES y aplicar FSCTL a un mount point: control sintético sin guarda que logre la operación, operación con guarda que la rechace, además de conservar bytes y raíz final. Un fallo por privilegio, buffer o tag inválido no prueba el guarda. Si los controles de compartición disponibles no establecen esta propiedad, elevar la estrategia concreta al orquestador antes de implementar otra API/política; hasta establecerla, fallar conservadoramente y conservar el recurso. El test actual de `fs::rename` del padre acredita rename denial y no-clobber, no reparse denial.

**CannotVerify:** no se ejecutó FSCTL ni se probó la combinación de derechos en este entorno durante esta revisión. El hallazgo acredita la contradicción estática y la garantía ausente; el escenario de escape es inferencia de las APIs.

### K2 — nueva regresión F11: cleanup rechaza una importación ordinaria preparada

**Ubicación:** `src-tauri/src/adapters/sqlite/library_repository.rs:705`, `:737`.

`no_destination_is_consistent` solo es verdadero cuando no hay documento reservado o cuando metadata representa `reuseExisting`. El método exige ese booleano incondicionalmente, incluso con `destination_path=Some(...)` válida. Una confirmación ordinaria preparada tiene documento reservado, destino derivado y metadata de confirmación normal; siempre recibe `ImportRecoveryRequired` al validar su posterior cancelación.

**Trigger/impacto:** interrupción/fallo entre preparación y promoción, seguido de cancelación de ese import. Se conserva un staging propio cuya eliminación es verificable, pero cancelar y reconciliar una cancelación PENDING no pueden completarse. La conservación evita borrado ajeno; no convierte en correcta la regresión de cancelación/recovery exigidos.

**Remedio mínimo:** aplicar el predicado de ausencia de destino solamente cuando `destination_path.is_none()`; mantener la comprobación positiva del destino reservado cuando es Some y las comprobaciones de biblioteca/UUID/refs. Añadir prueba de cancelación de confirmación normal preparada antes de promoción y su reapertura/recovery con cleanup completado. La regresión de referencia documental debe llegar a las consultas de referencias: su fixture actual también cae en este predicado anterior, por lo que su resultado verde no aísla esa protección.

### K3 — S6/F7: rechazo con cleanup demostrado deja STAGING huérfano permanente

**Ubicación:** `src-tauri/src/adapters/documents/store.rs:188`, `:202`; `src-tauri/src/modules/library/service.rs:196`; aserciones `src-tauri/tests/library_integration.rs:1076`.

Tras exceder 500 MiB, `stage_pdf` elimina correctamente su archivo por handle y devuelve `InvalidInput`. El resultado solo transporta un error, así que el servicio no recibe/persiste la prueba de ausencia y deja STAGING/hash ausente/error_json NULL. Recovery lo conserva como issue. La prueba ahora exige ese estado sin archivo; el reporte del autor lo reconoce. Los rechazos del parser que eliminan staging tienen el mismo problema de información descartada.

**Impacto:** un rechazo válido y repetible de un PDF sobredimensionado deja `recoveryRequired` persistente sin recurso a reconciliar ni token entregado con el que cancelar. Se degrada el diagnóstico y se acumulan intenciones huérfanas; no está demostrado un mecanismo posterior que las resuelva. No hay pérdida del original ni publicación de Paper/Document.

**Remedio mínimo:** distinguir en el resultado interno de stage entre fallo con cleanup por handle demostrado y fallo con cleanup no demostrado, preservando el error original. El job propietario debe persistir estado terminal/cleanup DONE únicamente con esa prueba; fallos de remove/sync/persistencia conservan intención con diagnóstico recuperable. No deducir DONE del código InvalidInput ni borrar a ciegas. Probar rechazo >MAX seguido de reopen sin pendientes/issue espurio y fallo de cleanup seguido de reopen conservando archivo/issue. La eliminación del falso DONE original es correcta, pero F7 no se cierra convirtiendo un cleanup conocido en ambigüedad permanente.

## Evidencia, alcance y calidad

Se leyó el paquete completo por porciones, con recuperación de dos fragmentos truncados; no se reabrió código de producto cambiado por separado. Fuera del paquete: contexto/brief/reporte central y MANAGED_FILES actuales, por las garantías de root/cleanup solicitadas; documentación Microsoft acotada al acceso necesario de FSCTL y compartición. No se inspeccionó código de fases posteriores.

Se comprobaron los cuatro sidecars finales del autor, todos `0`: fmt, cargo test, clippy y Tauri debug build. `fix1-final-cargo-test.log` confirma runners 6+7+13+41+3+6+4+1=81 entradas, incluyendo helper de proceso; clippy termina limpio y build genera ejecutable Windows debug. Son resultados del autor inspeccionados, no suites ejecutadas por este revisor. Las pruebas verdes no cubren K1/K2 y la expectativa verde de tamaño conserva K3. Evidencia UI previa sin cambios TypeScript; sin nueva afirmación de validación UI.

La separación transaccional, serialización exhaustiva y conservación de permisos mejoran el delta. Ninguna de esas mejoras ni la aprobación documental previa dispensan garantías del plan. No se añaden hallazgos Minor adicionales en este informe. Las discrepancias anteriores requieren corrección y revisión independiente antes de integración.

**Límites:** sin scans globales, dependencias/audit de red, pruebas sobre bibliotecas personales, GUI, instalador/equipo limpio, corte de alimentación ni benchmark de memoria. No se afirma heap acotado del parser. Rust reviewer ejecuta sus comprobaciones obligatorias coordinadas; este informe no las sustituye. Única escritura de esta revisión: este documento central; sin cambios de producto/Git/subagentes.
