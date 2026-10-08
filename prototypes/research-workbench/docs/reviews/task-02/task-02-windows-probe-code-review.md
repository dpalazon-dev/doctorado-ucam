# Revisión de código del probe experimental Windows handles

Estado: DONE_WITH_CONCERNS. Fuente aprobada para ejecución del probe en su scratch. No constituye aprobación ni certificación de producto T02.

SPEC experimental: APPROVE. Calidad: APPROVE con una observación Minor no bloqueante abierta tras delta 2. Critical: 0; Important: 0; Minor: 1.

## Alcance y evidencia

Revisión estática de `work/research/windows-handles/HandleProbe.cs` y `run-probe.ps1`, lectura completa una vez y lectura del delta posterior limitado al pin de atributos. No se ejecutaron probe, suites, comandos Git, configuración ni operaciones sobre bibliotecas. Única escritura: este informe central. Fuentes revisadas:

- HandleProbe.cs SHA256 `1062CB219B5502B1A825EAEE9F7B5CE8B54279219F7489C12DE3F3E7CF480506`.
- run-probe.ps1 SHA256 `DAC526ADC50E5530DDDE52F86AF4730A36806112187B1CEFE71E02480973BDE7`.

Objetivo: medir creación y rename relativos a un handle cuyo directorio fue convertido en junction, colisión sin reemplazo y protección de directorio no vacío mediante un child abierto. El experimento mide esas llamadas y máscaras concretas; no demuestra una garantía universal del filesystem ni el comportamiento de una futura implementación Rust.

## Hallazgos

### Minor M1 — RESOLVED en delta 2: causalidad del rechazo a borrar el pin

Archivo: `work/research/windows-handles/HandleProbe.cs:113` (también observación en 118).

La captura de cualquier IOException convierte cualquier fallo de File.Delete en evidencia de que el handle impidió vaciar el directorio. No se registra HResult ni se comprueba una eliminación positiva del mismo archivo tras cerrar el handle. Por tanto, un fallo distinto de sharing violation podría satisfacer el assertion y atribuirse al mecanismo de sharing. La carpeta sintética, el archivo creado por el probe y las máscaras explícitas reducen la probabilidad, pero no eliminan esa ambigüedad causal.

Mejora: registrar y exigir el HRESULT de sharing violation para el pin protegido y comprobar que la eliminación funciona después de cerrar el handle. En la variante attrs-only, registrar la excepción concreta; su resultado ya está correctamente etiquetado como observación y no se exige éxito de seguridad.

### Minor M2 — Una hipótesis refutada oculta los experimentos independientes restantes

Archivo: `work/research/windows-handles/HandleProbe.cs:93` (también 99).

Si la creación relativa alcanza outside, Require lanza antes del ensayo de rename, colisión y guardchild. Si falla la hipótesis de rename, tampoco se mide guardchild. Es un resultado seguro dentro del scratch, pero incompleto para comparar ambas estrategias: una hipótesis refutada no es un fallo del setup que invalide la otra.

Mejora: conservar fail-fast para ABI/controles positivos; registrar los resultados de cada hipótesis por separado, ejecutar los restantes ensayos y emitir al final un resumen con fallos. Alternativamente, ejecutar escenarios independientes explícitamente tras una refutación. No hace falta este cambio para autorizar la primera ejecución; el resultado parcial debe identificarse como tal.

## Fortalezas concretas

- ABI x64 explícito: asserts de ObjectAttributes 48, UnicodeString 16, IoStatus 16 y offset 20 para la cola UTF-16 del rename. El buffer del rename está inicializado, tiene espacio suficiente para el nombre y se libera en finally; los buffers de UNICODE_STRING también. SafeFileHandle se conserva dentro de scopes sin concurrencia ni cierre manual del root durante la llamada síncrona.
- Creación y rename positivos usan las mismas funciones y flags; verifican éxito exacto, ruta final y bytes. El positivo FSCTL además prueba que la junction redirige la lectura al destino sintético. Estos controles distinguen buena parte de los fallos de setup de los resultados del mecanismo.
- La prueba de colisión exige STATUS_OBJECT_NAME_COLLISION C0000035 y conserva los bytes de ambos archivos; ya no interpreta cualquier error como demostración de no-clobber.
- El runner genera un nombre GUID bajo PSScriptRoot, todos los destinos y archivos del probe permanecen dentro de ese fixture y no ejecuta cleanup recursivo a través de junctions. El destino llamado outside también pertenece al scratch.
- Los componentes de Relative rechazan separadores, colon, NUL, punto y doble punto, y limitan la longitud de UNICODE_STRING. Los nombres de rename se fijan dentro del propio probe. No hay entrada de producto ni rutas personales.
- Directory access incluye FILE_LIST_DIRECTORY y el pin principal incluye FILE_READ_DATA. La variante de atributos se observa separadamente, sin proclamar que su máscara participe necesariamente en el mismo sharing accounting.

## Cannot Verify

Sin ejecución no se verifica el comportamiento real de NT relativo, FSCTL, sharing, los códigos de error observados ni que todas las assertions pasen en este equipo. En el escenario de directorio convertido, cualquier NTSTATUS negativo satisface la assertion de no escape; el log debe conservar el estado exacto y su interpretación debe limitarse al intento observado. Un fallo de los controles positivos invalida el setup. Un fallo de una hipótesis posterior deja un resultado parcial, según M2.

No se revisan integración Rust, SQLite/WAL, ACL, instalación, carreras concurrentes completas, crash recovery ni capacidades de producto T02. No se ejecutó con privilegios ni se modificó infraestructura.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH / Important | 0 | pass |
| MEDIUM | 0 | pass |
| LOW / Minor abiertos | 1 | note |

Verdict: APPROVE para ejecución experimental en el scratch descrito. La evidencia de runtime y la interpretación arquitectónica siguen pendientes.

## Rerevisión acotada — delta 2

SPEC experimental: APPROVE. Calidad: APPROVE para ejecución en scratch; M1 resuelto para el pin principal, M2 sigue como límite Minor no bloqueante. Sin Critical/Important ni nuevos defectos de confianza alta. Fuente final SHA256 `1AD735D72D5719B52B197F073C38425E831E9F503F3FB286EAE27F6A5EAE80F4`. Se leyeron únicamente las adiciones y cambios solicitados de HandleProbe.cs; no se ejecutó el probe ni se reabrió producto T02.

- `HandleProbe.cs:25`: los parámetros opcionales de acceso y sharing conservan los valores anteriores para las llamadas existentes. El nuevo pin pide READ_DATA + READ_ATTRIBUTES + SYNCHRONIZE y share READ/WRITE, sin DELETE; encaja con el modo síncrono existente de NtCreateFile. No añade entrada arbitraria externa.
- `HandleProbe.cs:39`: WinRename usa SetFileInformationByHandle/FileRenameInfo (3), RootDirectory nulo, Replace false y el mismo layout x64 con cola en offset 20. El nombre UTF-16 absoluto procede exclusivamente de Path.Combine bajo fixture; el buffer se inicializa y libera en finally. GetLastWin32Error se captura inmediatamente tras la llamada. No se detecta defecto de memoria, alineación o rutas en este delta.
- `HandleProbe.cs:123`: la denegación de File.Delete ahora debe ser HRESULT 80070020 y se registra; tras cerrar los handles se exige borrado exitoso y ausencia del pin (126). Esto resuelve M1 para el assertion causal del pin principal. La variante attrs-only continúa como mera observación; no se utiliza para demostrar el mecanismo de seguridad.
- `HandleProbe.cs:130`: creación relativa de directorio y pin FILE_CREATE exigen éxito exacto, y se comprueba la ruta final del directorio y pin. El nuevo FSCTL exige ERROR_DIR_NOT_EMPTY antes del rename. El positivo Win32 valida ruta final y bytes; la colisión exige ERROR_ALREADY_EXISTS 183 y conserva ambos contenidos (138–142). La reapertura del pin usa FILE_OPEN y los mismos permisos de lectura (144). Los controles cubren las operaciones nuevas sin convertir cualquier error en un PASS.

M2 permanece: el fail-fast anterior puede impedir llegar al escenario nuevo si se refuta una hipótesis NT previa. En tal caso el log permite una conclusión parcial y debe identificarse el escenario no ejecutado.

Cannot Verify adicional: no se observó aquí la ejecución anterior. Root y autor informan exit 0 y salida en herramienta con archivo de log vacío; esa salida no constituye evidencia revisada por este agente. La ejecución del delta 2 y su captura stdout en proceso hijo siguen pendientes. La autorización de fuente no anticipa un resultado empírico.
