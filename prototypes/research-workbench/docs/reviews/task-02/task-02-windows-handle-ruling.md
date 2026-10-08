# T02 F10 — Investigación de handles y modificación reparse in-place

**DONE_WITH_CONCERNS. Investigación completada, opción mínima demostrada y propuesta normativa entregada al orquestador. F10 de producto sigue abierto hasta implementación y regresiones exactas.** Autor `/root/windows_handle_ruling`; producto leído en HEAD `922a0669d771d6778cd3d264bcc5be91f0e96ec6`, `.worktrees/task-02-library`. Ownership de investigación: `work/research/windows-handles/**` y este informe. No hay cambios de producto, commits, merges, ACL, administración, configuración global ni bibliotecas personales.

## Problema y fuentes primarias

F10 no queda demostrado por retener padres sin SHARE_DELETE ni por retirar SHARE_WRITE. El acceso de atributos no depende de esos flags y FSCTL admite WRITE_ATTRIBUTES además de WRITE_DATA. Control adversarial necesario: directorio vacío sintético, handle con FILE_WRITE_ATTRIBUTES exclusivamente y mount-point válido.

- [CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew): los flags de compartición no limitan acceso de atributos/EA; sin SHARE_DELETE se impide nueva apertura DELETE de datos compartidos. La sola presencia de un handle metadata-only no se adopta como guarda efectiva sin comprobar su efecto.
- [MS-FSA FSCTL_SET_REPARSE_POINT](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fsa/4aeefef8-92c3-4abc-af7a-a610caf8a165): acepta WRITE_DATA o WRITE_ATTRIBUTES; rechaza directorios no vacíos; distingue el privilegio necesario para tag SYMLINK. Es especificación abstracta del object store, no prueba universal de todo filesystem local.
- [MS-FSCC Mount Point Reparse Data Buffer](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/ca069dad-ed16-42aa-b057-b6b207f447cc): layout del buffer para tag A0000003, offsets/longitudes en bytes y nombres UTF-16.
- [NtCreateFile user mode](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile) y [WDK](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/nf-ntifs-ntcreatefile): admite nombre relativo a RootDirectory, OPEN_REPARSE_POINT evita seguir el objeto final y SYNCHRONOUS_IO_NONALERT necesita SYNCHRONIZE. Binding user-mode en ntdll.
- [OBJECT_ATTRIBUTES](https://learn.microsoft.com/en-us/windows/win32/api/ntdef/ns-ntdef-_object_attributes): OBJ_DONT_REPARSE deniega reparses encontrados al parsear el nombre. No se presupone que vuelva a parsear o inmovilice RootDirectory retenido.
- [FILE_RENAME_INFORMATION](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information) y [NtSetInformationFile](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/nf-ntifs-ntsetinformationfile): nombre simple relativo al directorio destino retenido, DELETE en origen, ReplaceIfExists=false y clase10. Windows87 observado en la variante Win32 no es evidencia sobre NTAPI.

Consultadas el 2026-10-02. Se distingue documentación de inferencia y de prueba ejecutada.

## Opciones consideradas

1. Retirar SHARE_WRITE solamente: insuficiente por canal WRITE_ATTRIBUTES.
2. Abrir/crear todo relativo con NtCreateFile y promover con NtSetInformationFile relativo: documentado, pendiente comprobar in-place reparse sobre RootDirectory y caso positivo. No resuelve SQLite Win32 por ruta por sí solo.
3. Adquisición relativa mínima + directorio mantenido no vacío: candidato más pequeño. Se retiene cada siguiente ancestro sin DELETE sharing; para el último directorio se retiene un hijo regular. Luego se verifica el mismo handle del directorio para asegurar que no es reparse antes de publicar el binding. El hijo retenido impide vaciar el directorio y, en el filesystem demostrado, la condición no vacío bloquea SET_REPARSE_POINT. Permitir compartir WRITE en el directorio sigue siendo necesario/compatible con las operaciones internas de Windows. No se afirma que esa compartición bloquee atributos.

El pin de raíz existente puede ser research.sqlite abierto sin mutación, con FILE_READ_DATA|FILE_READ_ATTRIBUTES, share READ|WRITE sin DELETE. El diagnóstico de esquema futuro no crea sentinelas. Raíz nueva o leaf administrado vacío necesita un hijo reservado creado/adquirido relativamente antes de permitir I/O por rutas; un create absoluto del pin antes de proteger padre repetiría el hueco. El nombre reservado, persistencia/backup/restauración y exclusión de cleanup necesitan resolución normativa del orquestador antes de producto.

## ABI y dependencias mínimas candidatas

windows-sys0.61.2 ya contiene bindings ntdll, sin dependencia nueva. Features adicionales para NtCreateFile: `Wdk_Foundation`, `Wdk_Storage_FileSystem`, `Win32_Security`, `Win32_System_IO`; se conservan `Win32_Foundation` y `Win32_Storage_FileSystem`. `Win32_Security` es necesario para OBJECT_ATTRIBUTES generado, no autoriza cambios de ACL.

Imports: `Wdk::Foundation::OBJECT_ATTRIBUTES`; `Wdk::Storage::FileSystem::{NtCreateFile, FILE_OPEN, FILE_CREATE, FILE_OPEN_IF, FILE_DIRECTORY_FILE, FILE_NON_DIRECTORY_FILE, FILE_OPEN_REPARSE_POINT, FILE_SYNCHRONOUS_IO_NONALERT}`; `Win32::Foundation::{UNICODE_STRING, OBJ_CASE_INSENSITIVE, OBJ_DONT_REPARSE}`; `Win32::System::IO::IO_STATUS_BLOCK`; derechos/flags en Storage::FileSystem. NtRename añadiría `NtSetInformationFile`, `FILE_RENAME_INFORMATION`, union y clase `FileRenameInformation`, pero no se recomienda ampliar la API si la opción3 resulta suficiente.

Las pruebas FSCTL pueden necesitar además `Win32_System_Ioctl` para FSCTL_SET_REPARSE_POINT y `Win32_System_SystemServices` para IO_REPARSE_TAG_MOUNT_POINT; REPARSE_DATA_BUFFER está disponible en Wdk::Storage::FileSystem. Esos dos features son para constantes del test, no requisitos de adquisición NtCreateFile. GetVolumeInformationByHandleW ya está en Storage::FileSystem para verificar filesystem local sobre handle; FILE_SUPPORTS_REPARSE_POINTS sólo indica capacidad, no demuestra la propiedad no-vacío requerida.

Wrapper privado seguro: un componente cerrado validado, sin separadores, colon, NUL, . o ..; longitud UTF-16 checked en bytes u16 y sin truncado; UNICODE_STRING/OBJECT_ATTRIBUTES/IO_STATUS_BLOCK vivos durante llamada síncrona; parent handle retenido; NTSTATUS comprobado antes de convertir HANDLE a File RAII. Flags tipo directorio/no-directorio y OPEN_REPARSE_POINT; verificar atributos/identidad sobre el handle devuelto. No EXISTS seguido de CREATE por ruta ni reabrir PathBuf como sustituto del handle autorizado.

## SQLite y bootstrap

`actor.rs:prepare`, `schema_probe.rs:fingerprint/probe` y `migrations.rs` siguen usando rutas. Store relativo no cierra esas aperturas. La raíz ligada debe incluir la condición de no-vacío/no-reparse durante todo probe, SQLite, WAL, manifest/lock y vida del actor; los ancestros quedan retenidos también. DB pin debe ser compatible con WAL y con probe read-only/future y conservarse hasta después de cerrar conexión. El fuente local `libsqlite3-sys0.38.2/sqlite3/sqlite3.c` winOpen54175–54199 usa GENERIC_READ o READ|WRITE con SHARE_READ|SHARE_WRITE para apertura ordinaria: apoya compatibilidad del pin de lectura, pero no reemplaza test exacto rusqlite/WAL. No se propone VFS nuevo.

## Probe aislado y revisión

Fuente: `work/research/windows-handles/HandleProbe.cs`; runner `run-probe.ps1`. Fixture GUID propio dentro de ese directorio, retenido como evidencia, sin limpieza recursiva a través de junctions.

Revisor inicial `/root/task02_security_review`: hallazgo de evidencia falso positivo por aceptar cualquier error NT sin controles positivos. Corregido antes de ejecutar: create/rename relativos con éxito, ruta final y bytes; collision exige STATUS_OBJECT_NAME_COLLISION; asserts ABI x64. Revisión de `/root/task02_spec_review` sobre SHA2561062CB219B5502B1A825EAEE9F7B5CE8B54279219F7489C12DE3F3E7CF480506: SPEC APPROVE, calidad APPROVE, dos Minor (HRESULT genérico al borrar y fail-fast ante hipótesis NT refutada). Informe `task-02-windows-probe-code-review.md`. Ejecutado sólo después de esa aprobación.

Entorno comprobado: Windows10.0.26200.0 x64, volumen C:NTFS Healthy. ABI esperado OA48/UNICODE_STRING16/IO_STATUS_BLOCK16/rename tail20. Los accesos del probe son directorio LIST|TRAVERSE|READ_ATTRIBUTES|SYNCHRONIZE; pin READ_DATA|READ_ATTRIBUTES. Se observa separadamente un pin attrs-only sin atribuirle garantía no-delete.

## Primera ejecución y limitación de captura

Comando desde root PowerShell:

```powershell
& ./work/research/windows-handles/run-probe.ps1 *> ./work/research/windows-handles/probe-1.log
$probeExit = if($?){0}else{1}
Set-Content ./work/research/windows-handles/probe-1.exit.txt $probeExit
Get-Content ./work/research/windows-handles/probe-1.log
exit $probeExit
```

Exit0 observado por herramienta, fixture `fixture-9474c409d59c441fb8b10d1ca5c5d3a3`. `Console.WriteLine` emitió directamente al stdout de la herramienta y el log PowerShell quedó vacío; el orquestador detectó esa limitación. No se presenta probe-1.log como evidencia bruta. La siguiente ejecución usará proceso hijo PowerShell con captura de stdout nativo. Los siguientes son hechos observados del output de herramienta, no un log reconstruido:

| Escenario | Resultado observado |
|---|---|
| ABI x64, tamaños y tail | PASS |
| Control positivo NTcreate/rename relativos | STATUS_SUCCESS0, ruta final correcta y bytes conservados |
| Directorio vacío sin guarda, WRITE_ATTRIBUTES-only | SET_REPARSE_POINT éxito0 y lectura por junction llega al marker exterior |
| Directorio guardado sin SHARE_WRITE | SET_REPARSE_POINT éxito0: canal atributos permanece abierto |
| RootDirectory retenido convertido in-place | NTcreate y NTrename relativos devuelven C0000280, exterior sin nuevos archivos |
| NTrename relativo destino existente | C0000035, origen y destino conservan bytes |
| Pin READ_DATA|READ_ATTRIBUTES sin DELETE sharing | SET_REPARSE_POINT falla145/DIR_NOT_EMPTY; File.Delete pin denegado |
| Pin READ_ATTRIBUTES únicamente sin DELETE sharing | File.Delete permitido: esa variante no sirve como guarda de entrada |

C0000280 corresponde a STATUS_REPARSE_POINT_NOT_RESOLVED según [MS-ERREF](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-erref/596a1078-e883-4972-9bbc-49e60bebca55). No es Windows87 ni prueba de soporte universal de NTAPI; sí es rechazo seguro en el escenario probado con controles positivos.

Delta2: pin creado/reabierto relativo con READ_DATA|READ_ATTRIBUTES|SYNCHRONIZE, rootdirectory creado relativo, Win32 absoluto no-clobber bajo pin, y delete pin exige HRESULT80070020 con control positivo tras cerrar guarda. Fuente SHA2561AD735D72D5719B52B197F073C38425E831E9F503F3FB286EAE27F6A5EAE80F4. `/root/task02_spec_review` dio APPROVE antes de ejecución, sin Critical/Important; Minor de fail-fast permanece limitado a que una hipótesis refutada detendría escenarios posteriores. En ejecución2 se completaron todos.

## Evidencia durable de ejecución2

Comando exacto en PowerShell desde root:

```powershell
$probeShell = (Get-Process -Id $PID).Path
& $probeShell -NoProfile -File ./work/research/windows-handles/run-probe.ps1 *> ./work/research/windows-handles/probe-2.log
$probeExit = $LASTEXITCODE
Set-Content ./work/research/windows-handles/probe-2.exit.txt $probeExit
Get-Content ./work/research/windows-handles/probe-2.log
exit $probeExit
```

Shell hijo: `C:/Users/david/.cache/codex-runtimes/codex-primary-runtime/dependencies/native/powershell/pwsh.exe`. `probe-2.log` contiene stdout real del proceso hijo, no un resumen; `probe-2.exit.txt` contiene0. Log2394bytes, SHA256D37CD4C6AFE3AC0575373949C9831D70AA8A71C740D362C169C825A803EA65E8, 31 líneas PASS y ALL ASSERTIONS PASSED. Fixture `fixture-a3d547f723a5468589789fb7847659d6`. Fuente aprobada antes de ejecución y sin cambios después.

Además de reproducir toda la matriz1, ejecución2 demostró:

| Escenario mínimo adicional | Resultado |
|---|---|
| Crear directorio nuevo relativamente a anchor | NTSTATUS0 |
| Crear pin con acceso READ_DATA|READ_ATTRIBUTES|SYNCHRONIZE | NTSTATUS0, mismo padre y ruta final |
| Abrir pin existente con FILE_OPEN/acceso sólo lectura | NTSTATUS0, sin permiso write ni create disposition |
| SET_REPARSE_POINT sobre directorio con pin relativo | error145/DIR_NOT_EMPTY usando WRITE_ATTRIBUTES-only |
| Intentar borrar pin retenido | HRESULT80070020/SHARING_VIOLATION |
| Borrar pin después de cerrar handles | éxito, prueba que no era denegación de ACL/privilegio |
| Win32 rename absoluto por handle bajo pin | éxito0, ruta final correcta y bytes correctos |
| Win32 rename absoluto con destino aparecido | error183/ALREADY_EXISTS, ambos archivos conservados |

## Resolución mínima propuesta para ADR-016/MANAGED_FILES

**Recomendar opción3.** No trasladar promoción a NtSetInformationFile: NtRename relativo funciona en positivos y falla seguro tras reparse en este entorno, pero el pin demostrado permite conservar SetFileInformationByHandle absoluto y protege también SQLite por ruta. La dependencia de protección pasa a ser directorio no vacío con hijo retenido, no rechazo de WRITE_ATTRIBUTES mediante sharing.

Texto propuesto para sustituir la garantía insuficiente:

> En Windows sobre el filesystem local demostrado, cada componente de la cadena autorizada permanece abierto con FILE_LIST_DIRECTORY y sin SHARE_DELETE. Cada padre contiene el siguiente hijo/directorio retenido. Antes de habilitar operaciones por ruta en el último directorio, se adquiere mediante NtCreateFile relativo un hijo regular retenido con FILE_READ_DATA|FILE_READ_ATTRIBUTES|SYNCHRONIZE, compartición READ|WRITE sin DELETE, y se verifica otra vez sobre el mismo handle que el directorio es regular/no-reparse y mantiene la identidad autorizada. Mientras ese hijo existe y no puede borrarse, el padre no puede quedar vacío; SET_REPARSE_POINT mount-point fue rechazado con DIR_NOT_EMPTY bajo WRITE_ATTRIBUTES en NTFS. No se afirma que compartir escritura ni un handle de atributos inmovilice reparse. Si no se puede establecer la propiedad, se devuelve error seguro y no se habilita I/O por ruta.

Adquisición/camino mínimo exacto:

1. Abrir el ancla de volumen local y conservarla. Caminar cada componente mediante NtCreateFile `RootDirectory=handle_parent`, `ObjectName=único componente validado`, `OBJ_CASE_INSENSITIVE|OBJ_DONT_REPARSE`, `FILE_OPEN_REPARSE_POINT|FILE_SYNCHRONOUS_IO_NONALERT`, tipo DIRECTORY, acceso LIST|TRAVERSE|READ_ATTRIBUTES|SYNCHRONIZE, share READ|WRITE sin DELETE. Verificar el objeto devuelto. Crear componentes autorizados con FILE_OPEN_IF/FILE_CREATE relativo, sin `std::fs::create_dir` por ruta desprotegida.
2. En raíz con DB existente, abrir `research.sqlite` relativo con FILE_OPEN y acceso READ_DATA|READ_ATTRIBUTES|SYNCHRONIZE, share READ|WRITE sin DELETE; rechazar reparse/no-regular por el mismo handle. Este pin no escribe ni exige writer lock y sirve en futura biblioteca de sólo diagnóstico. La presencia se determina por esa apertura/NTSTATUS, no mediante path.exists como autorización.
3. En inicialización sin DB, usar un hijo reservado de la raíz adquirido/creado relativamente, manteniendo la secuencia de exclusión de writer antes de crear/inicializar DB. No crear research.sqlite como truco de pin antes de LibraryLock. El orquestador puede preferir el lock ya previsto como pin si se adquiere relativamente y su handle cumple share/derechos/vida; eso exige adaptar su adquisición, no asumir que el fs2/OpenOptions actual protege la ruta. Si se elige un sentinel nuevo, fijar su nombre reservado y lifecycle explícitamente. La aparición concurrente de DB exige revalidación/fallo antes de abrir SQLite; no se promete cero efectos de inicialización frente a toda creación externa simultánea.
4. En leaf administrado vacío, crear/abrir pin reservado relativo, conservarlo y comprobar directorio no-reparse después de adquirirlo. Si alguien convirtió el directorio antes de pin, la adquisición relativa probada falla C0000280 y se conserva el recurso; nunca recuperar vía path absoluto. Después del pin y del check del mismo handle, toda la cadena tiene la condición de no vacío y retenciones no-delete. No hay ventana check-then-rename mutable por FSCTL en el mecanismo probado.
5. Mantener guardas/pin mientras SQLite/probe/manifest/lock o Store operan por ruta. Root pin vive hasta cerrar conexión; ManagedDirectory conserva el suyo a través de clones mientras trabaja. Cleanup sólo toca recursos autorizados y nunca elimina el pin. Promoción conserva FileRenameInfo/ReplaceIfExists=false y origen verificado; no sobrescribe un racing target.

### ABI interna propuesta

- `ManagedRoot` mantiene identidad, cadena de handles y pin raíz como RAII compartido; un binding publicado significa protección completa, no sólo canonical_path comprobada.
- `ManagedDirectory` mantiene handle y pin de leaf, o hijo retenido suficiente si ya es padre no vacío; no devolver directorio listo para I/O por ruta antes de completar el pin/check.
- Helper privado `open_relative(parent: &File, component: &str, kind, disposition, access, share) -> Result<File, AppError>` encapsula NtCreateFile síncrono; callers usan enums/derechos cerrados, no una API filesystem genérica ni paths recibidos de UI. Access share de pin y acceso de archivo administrado son propósitos diferentes.
- Mantener `LibraryRoot::at` sin I/O y compartir binding completado del actor con Store. No introducir contratos IPC, DTOs, migraciones o un SQLite VFS nuevo.
- Destino absoluto se deriva de padres protegidos y no de rutas UI; mismo handle de archivo para hash/read/delete. NtRename se queda sólo como evidencia comparativa del probe, sin nuevos imports de producto si se adopta este mínimo.

La política local/NTFS o una comprobación equivalente de capacidad demostrada debe fijarla el orquestador. El experimento no habilita silenciosamente ReFS, SMB, exFAT, reparse tags de filtros o rutas UNC. Un filesystem no demostrado no recibe garantía por inferencia.

### Costes y alternativas

Ventajas: cubre bootstrap SQLite y Store; FFI nuevo reducido a adquisición relativa; preserva rename/borrado por handle existentes. Costes: más handles; nombre reservado o pin lock; nuevas reglas explícitas de cleanup/backup/restore; Busy ante intento de delete externo. Sentinel persistente se excluye del conjunto de documentos y no se confunde con una intención; backup/restauración deben declarar si lo conservan o regeneran. Estos detalles normativos pertenecen al orquestador, no se han implementado unilateralmente.

Alternativa NTcreate+NTrename completa: evita gran parte de I/O por ruta de Store, pero exige otra FFI y deja SQLite sin resolver; no aporta suficiente simplificación frente al pin. Oplocks/ACL no se exploraron: añadir complejidad o cambiar permisos no es necesario para el mecanismo acotado demostrado.

## Regresiones exigidas al autor antes de cerrar F10

- Migrar escenario FSCTL positivo/guardado a prueba Windows sintética real con WRITE_ATTRIBUTES-only y estados exactos. Incluir directorio vacío recién creado y guardas durante toda operación.
- Prueba exacta rusqlite0.40.2 con DB pin READ_DATA/no-delete: conexión writable, WAL/SHM, commit/checkpoint/cierre, reabrir y verificar datos. La lectura del fuente SQLite sólo apoya compatibilidad, no la certifica.
- Future-schema real con DB existente: pin por FILE_OPEN sin creación, inventario/hashes antes/después iguales, sin lock/migración/staging/manifest nuevo. No probar sobre biblioteca personal.
- DB ausente/init: no create DB antes de writer lock; apertura relativa decide ausencia y DB concurrente se rechaza/revalida antes SQLite. Pin adquirido correctamente antes de cualquier create por path.
- Retención de ancestros y pin frente a rename/delete; atributos de todos los handles rechazan reparse; fallos de adquisición conservan bytes y recurso. Params limitados a componentes internos válidos; root sin protección completa no se publica a Store.
- Win32 promoción por handle bajo pin, destino racing no-clobber, cleanup que preserva sentinel/lock, nueva revisión de seguridad/Rust y gate del autor. No sustituirlo por el probe de investigación.

## Autorrevisión y límites

No código producto modificado; HEAD de producto comprobado antes y después `922a0669d771d6778cd3d264bcc5be91f0e96ec6`. Dos revisiones independientes de fuente del probe precedieron sus ejecuciones. El fallo de captura1 queda documentado y corregido mediante captura real2. Dos Minor: HRESULT se corrigió y verificó; fail-fast quedó sin impacto en ejecución2 completa. No se afirma control ACL ni administración.

No hay cierre de F10, prueba de producto, suite nativa, instalador, VM limpia ni prueba de corte eléctrico. El alcance de la propuesta de pin queda restringido al filesystem/tag/entorno demostrado; MS-FSA por sí solo no extiende esa garantía a todos los filtros o filesystems. Compatibilidad exacta rusqlite/WAL y rama futura permanecen regresiones del autor después de la decisión del orquestador.
