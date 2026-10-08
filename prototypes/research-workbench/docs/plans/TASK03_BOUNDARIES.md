# T03 — fronteras y decisiones del lector

Resolución del coordinador, 2026-10-02. Complementa el brief T03. IPC v1, DTOs y esquema 0001 permanecen iguales. [TASK03_PORTS](TASK03_PORTS.md) concreta las firmas internas y propiedad de handles; se contrastan con el producto T02 revisado antes de dispatch.

## Aplicación y persistencia

ReaderService consume un puerto específico ReaderPersistence y un acceso lector pequeño producido por el mismo DocumentStore de T02. No importa DbActor, Connection, adaptadores SQLite ni AppHandle. SqliteReaderPersistence posee el actor, abre UoW/with_receipt y llama helpers de aplicación con repositorios transaccionales. Se conserva la excepción explícita de &rusqlite::Transaction; ningún repositorio hace commit.

ReaderPersistence ofrece candidato de apertura, confirmación de apertura, último abierto, documento registrado, consulta y guardado de posición. DocumentReadAccess ofrece apertura verificada de un RegisteredDocument y lectura mediante el handle obtenido. Es una interfaz lectora separada sobre el mismo store, sin conceder promoción/borrado al protocolo. Los records internos contienen biblioteca, IDs, referencia relativa, estado, hash y tamaño; no son DTOs ni rutas que React pueda enviar.

openPaper obtiene candidato registrado, comprueba el archivo fuera de TX y confirma en TX corta volviendo a verificar asociación/biblioteca/documento. Actualiza lastOpenedAt, contexto de sesión, auditoría y receipt; no cambia Paper.revision ni Paper.updatedAt bibliográficos, metadatos o fase. Reader llama un helper público de aplicación Library para actividad dentro de esa misma TX, sin escribir tablas bibliográficas mediante un repositorio privado ajeno. Abrir un archivado no lo restaura. getPaper/lastOpened/posición no producen actividad.

El replay no vuelve a tocar actividad ni el receipt. Un receipt previo no concede acceso a un archivo ahora ausente o no autorizado: verificar acceso actual antes de emitir documentUrl, conservando el resultado durable original ante error de acceso posterior. Guardado de posición usa expectedRevision; omisión devuelve default determinista sin insertar. UI serializa por documento y solo muestra Guardado tras respuesta confirmada.

El namespace de receipts es global a la biblioteca. Al introducir Reader, sus mutaciones openPaper/saveReadingPosition comparten la misma exclusión por requestId con Library antes de efectos y hasta completar el trabajo admitido. Extraer únicamente el registro de exclusión existente a un componente de aplicación compartido, inyectado una vez por composición; no crear otro registro por servicio ni un executor genérico. Conservar orden requestId antes de token. La ABI se concreta desde T02 revisada. Regresión exigida: una saga Library pausada compite con una mutación Reader de igual requestId; la petición incompatible no modifica el segundo recurso ni roba el receipt. Los futuros consumidores de receipts reutilizan este componente.

## Interacción con duplicados

Los candidatos del preview, del DOI introducido al confirmar y de un duplicado aparecido entre preparación y commit se presentan con el mismo flujo. DuplicateDecisionRequired entrega `error.details.candidates` con el DTO existente, desde la TX que detecta la coincidencia; UI valida su estructura antes de usar IDs. El usuario elige explícitamente si hay varios candidatos. «Abrir existente» espera cancelImport confirmado (requestId propio y estable en reintentos) y después solicita Reader.openPaper con otro requestId. «Cancelar importación» solo cancela. No se cambia el payload de una intención ya ligada ni se invoca listPapers para descubrir coincidencias.

Si cancelación falla, conservar diálogo, candidato y borrador; no abrir ni anunciar éxito. Si cancelación terminó pero falla la apertura, reintentar solo Reader; el token ya no vuelve a usarse. Details inválidos/ausentes muestran error recuperable y permiten cancelar, sin inventar IDs. No restaurar implícitamente un paper archivado. ReuseExisting continúa disponible en backend, pero esta UX no necesita inferir si aún es posible ligar esa decisión.

Regresiones: DOI añadido después del preview; razones DOI+hash combinadas por ID; candidatos distintos; carrera determinista que crea un ganador antes del commit final; cancelación/replay del perdedor conserva PDF/metadatos/posición del ganador. UI prueba que cancel pendiente/fallida nunca abre y que apertura fallida después de cancel confirmado no vuelve a importar. Backend produce candidatos reales; mocks de UI no lo demuestran.

## Protocolo Windows

Registrar protocolo asíncrono research. En el baseline Tauri2.12.1/Wry0.57.0, documentUrl Windows es http://research.localhost/<UUID>, mientras Wry convierte la URI del callback a research://localhost/<UUID>. Validar esa forma interna exacta, main, origen autorizado, método GET y ruta de un UUID canónico sin segmentos/query extra antes de consultar persistencia. Resolver documentId registrado de la biblioteca activa y abrir archivo administrado comprobando raíz/reparse points; leer desde el mismo handle verificado. Las capabilities IPC no sustituyen autorización del protocolo.

Perfil inicial: respuesta 200 completa, application/pdf y longitud exacta; sin anunciar Accept-Ranges ni streaming incremental. Una petición Range no implementada puede recibir 200 completo, nunca 206 falso. El responder Tauri recibe una sola respuesta de bytes. Las lecturas van a background, fuera de DB/TX/ventana, y cuentan como operaciones para el cierre de T02. El máximo de importación sigue500MiB; medir memoria/latencia con archivo grande antes del piloto. Se acepta coste proporcional al PDF y copias del runtime; no se promete una cota total RAM. Range queda para una optimización motivada por medidas, sin API nueva inferida.

CORS permite solo origen empaquetado verificado y origen dev concreto únicamente en desarrollo; no *. Errors HTTP seguros. Cancelar fetch/render/destroy en PDF.js invalida resultados UI y libera sus recursos; no acredita que el job Rust ya aceptado se haya abortado. No añadir canal IPC de cancelación. Cierre de aplicación espera el punto seguro de ese trabajo.

## PDF.js y recursos locales

Usar PDF.js6.3.289 ya fijado, con worker de esa misma versión y cmaps, standard_fonts, iccs y wasm/fallbacks necesarios copiados determinísticamente al bundle. Vite/script existente, sin dependencia nueva; comprobar dist y licencias. URLs locales terminadas en slash para directorios. No importar viewer/sandbox/quickjs ni activar JavaScript del PDF. No usar isEvalSupported de recetas antiguas: no está en esta API.

Se adopta useWasm=true para decodificadores empaquetados. CSP añade connect-src 'self' para fetch de recursos locales y script-src 'wasm-unsafe-eval' para compilación WASM, manteniendo denegación de unsafe-eval JavaScript, scripts remotos, objetos/frames y hosts no necesarios. Worker local con worker-src 'self'. No ampliar a blob: ni font-src data: sin una necesidad observada y revisión específica. Inspeccionar CSP efectiva release y render offline en WebView2; el servidor Vite no prueba empaquetado.

## Ownership y aceptación

Al dispatch T03, transferir application/reader_ports.rs y application/reader.rs, el adapter sqlite/reader_repository.rs y extensión lectora sobre el mismo DocumentStore; helper Library de actividad; composición lib/desktop/registry; capability/CSP y assets/build; App/adapter/navegación y capability reader. El trait ReaderRepository vive en reader_ports, sin capa duplicada modules/reader/repository.rs. Asignar paths exactos una vez revisado T02. Ninguna edición concurrente del producto de T02. Solo worker asignado escribe estas integraciones; Sol decide/revisa.

Tras la revisión T02, ADR-016 y MANAGED_FILES añaden la base obligatoria para esa extensión: reutilizar el helper Windows de handles/identidad/guardas y la propiedad de trabajos admitidos, sin volver a resolver rutas por nombre ni soltar permisos al desaparecer el caller. T03 no consume LibraryService para simular una transacción cruzada; llama a los helpers de aplicación *_in_tx bajo su Transaction prestada, conforme a TASK02_PORTS resolución7. Las firmas exactas se toman del reporte T02 corregido y revisado, no del informe inicial 45c4c8c.

Además de los tests del brief: openPaper conserva revisión bibliográfica, replay no reescribe actividad; URI interna Windows, origen/ventana, respuesta200/longitud sin Range anunciado; recursos locales/CSP con PDF escaneado y caso que use decodificadores pertinentes; cierre cuenta lecturas admitidas. Tests unit/mocks, protocolo real y render WebView2 se reportan separados. Aviso recoveryRequired obligatorio sin prometer UX de reanudación de drafts.

Investigación primaria y tradeoffs: work/reviews/task03-reader-preflight.md. Fuentes: [Tauri2.12.1 responder](https://raw.githubusercontent.com/tauri-apps/tauri/tauri-v2.12.1/crates/tauri/src/app.rs), [Wry0.57.0 Windows](https://raw.githubusercontent.com/tauri-apps/wry/wry-v0.57.0/src/webview2/mod.rs), [PDF.js6.3.289 API](https://raw.githubusercontent.com/mozilla/pdf.js/v6.3.289/src/display/api.js), [CSP Tauri](https://v2.tauri.app/security/csp/). Son evidencia de APIs disponibles, no del funcionamiento de Research Workbench.
