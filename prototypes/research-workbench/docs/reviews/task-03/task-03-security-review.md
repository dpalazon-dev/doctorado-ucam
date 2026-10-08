# T03 — revisión independiente de seguridad

Fecha: 2026-10-02. Revisor: `/root/task03_security_review`.

- BASE: `cb60ee2d375aafc60dd2f8a5d696f1f375321cff`.
- HEAD revisado: `870b3321c4e3d46597848ae5d3faf40395229621`.
- HEAD de producto declarado: `ccc1f796f1b8af2ecaa8838f9c357c19949ae444`.
- Worktree: `.worktrees/task-03-reader`, limpio antes y después de la revisión.
- Alcance: cambios T03 en Reader, persistencia, extensión lectora de DocumentStore, protocolo, composición, transporte, CSP y recursos PDF.js; interacciones nuevas con T02. No se repitió la auditoría integral F10.
- Fuentes: AGENTS, INTENT, STATUS, brief central de revisión, TASK03_PORTS, TASK03_BOUNDARIES, MANAGED_FILES e informe del autor. Se aplicó la skill `security-review` de `C:/Users/david/.agents/skills/security-review/SKILL.md` al producto local, sin trasladar controles de cuentas/servidores que no existen en este alcance.

## Veredicto

**DONE_WITH_CONCERNS — no aprobar integración todavía.** Hay un hallazgo Important de integridad/autorización contextual y un Important de validación de una frontera de seguridad exigida. No se identificó un Critical. No se modificó producto ni se realizó merge.

## SEC-T03-1 — Important: el replay permite que el documento ya pertenezca a otro Paper

Localizadores: `src-tauri/src/adapters/sqlite/reader_repository.rs:145` y `:168`.

Caso: abrir Paper A con documento D y conservar su receipt; cambiar después la asociación registrada para que D pertenezca a Paper B y B.active_document_id=D, retirando el vínculo activo de A; repetir el mismo requestId/payload de A. La rama replay de prepare_open consulta D por el ID conservado en el receipt, pero no compara su paper_id actual con command.paper_id. registered_document acepta la asociación actual D↔B. open_verified comprueba ese registro actual y el archivo administrado. confirm_open obtiene el receipt original sin ejecutar open_paper_in_tx y compara el registro actual con esa misma lease actual; ambos contienen B, por lo que la comprobación pasa. Se devuelve el OpenPaperDto original A + URL de D aunque D ya no está autorizado como documento de A.

Evidencia: seguimiento directo del flujo del HEAD indicado. En aperturas nuevas, `application/reader.rs:23` sí compara current.document.paper_id con command.paper_id; with_receipt omite ese helper en replay. No se ejecutó una reproducción nueva de reasociación en esta revisión, para conservar el checkout intacto. La prueba existente de replay solo cambia last_opened_at y elimina el archivo posteriormente.

Impacto: incumple la autorización/asociación actual exigida por TASK03_PORTS y puede presentar contenido bajo una ficha/procedencia incorrecta. Requiere una asociación durable alterada; no se ha demostrado que la UI actual pueda producirla, ni una fuga remota o entre usuarios. Es un fallo del control de integridad declarado, no una afirmación de explotación nativa.

Corrección: verificar en prepare_open y en la confirmación final que el documento del receipt sigue asociado al Paper solicitado y que ese Paper conserva active_document_id=D. Preservar receipt y auditoría originales ante rechazo. Añadir regresión con dos Papers y reasociación durable, y mantener el replay permitido cuando solo cambian actividad/metadatos autorizados. La confirmación final debe cubrir también un cambio entre prepare y confirm.

## SEC-T03-2 — Important (validación): no se ejercita la lectura/respuesta real del protocolo

Localizadores: `src-tauri/tests/document_protocol.rs:50`, `:62`, `:93`; `src-tauri/tests/reader_integration.rs:185`; implementación a proteger en `src-tauri/src/modules/reader/protocol.rs:63` y `:111`, `src-tauri/src/adapters/documents/store.rs:93`.

Las tres pruebas document_protocol cubren UUID desconocido, parser/traversal y junction rechazada al abrir el handle. Ninguna llama read_all, ReaderService.read_document o el responder del handler. El caller-drop probado corresponde a openPaper con un acceso retenido antes de confirmación; no prueba una lectura bloqueante ni la propiedad hasta responder. Por tanto, el gate verde no comprueba cuerpo del mismo handle, longitud real, 200 completo ante Range ni retención de mantenimiento/lease durante lectura y respuesta. Son regresiones backend explícitamente exigidas en TASK03_PORTS y en el brief de revisión, distintas de la QA WebView2 pendiente.

La inspección del código es favorable: read_all comprueba tamaño antes/después, lee desde ManagedFile y devuelve la lease; register conserva lease y OperationPermit hasta después de responder, emite 200 y Content-Length calculado y no anuncia Accept-Ranges. No se afirma que exista una lectura fuera de raíz o liberación prematura demostrada. El problema es falta de evidencia ejecutable en la nueva frontera que entrega PDFs a la WebView.

Corrección: añadir pruebas deterministas que recorran la lectura real y la construcción/entrega de la respuesta con PDF sintético; comprobar bytes/longitud/status/ausencia de falso Range, mantenimiento ocupado y handle retenido hasta responder, incluyendo desaparición del caller/fetch. Puede utilizarse un seam mínimo en la frontera del responder; no necesita GUI ni sustituye la prueba nativa posterior.

## Controles observados sin hallazgo adicional

- Parser cerrado a ventana main, GET, research://localhost, UUID canónico único sin query/segmentos/escapes y Origin exacto. Origen dev solo bajo debug_assertions. La autorización del protocolo es independiente de las capabilities IPC.
- Resolución por DB de documento ACTIVE y vínculo actual Paper.active_document_id; store comprueba biblioteca activa, namespace documents/UUID/original.pdf y límite 500 MiB, reutilizando ManagedFile y las guardas T02.
- SQL nuevo parametrizado. La UI recibe UUID y URL interna; no obtiene rutas libres, SQL ni filesystem.
- RequestRegistry único compartido por composición; open/save transfieren permisos al trabajo propietario y confirm_open mueve la lease al job del actor. Extracción de Library mantiene el algoritmo previo.
- CORS de respuesta exitosa usa solo el origen ya autorizado; errores HTTP vacíos y seguros. CSP añade únicamente self para recursos locales y wasm-unsafe-eval aprobado; no incorpora unsafe-eval JavaScript, hosts remotos, frames u objetos.
- Loader usa API de documento/canvas, sin viewer, scripting manager ni sandbox. Recursos y worker locales; XFA desactivado. No se detectó un ejecutor de JavaScript del PDF ni render de HTML no escapado en los componentes nuevos. Inventario presente de dist: 200 rutas que contienen pdfjs/pdf.worker; ninguna coincide con quickjs/sandbox.

## Comandos y resultados de esta revisión

En `.worktrees/task-03-reader`:

```powershell
git status --short
git rev-parse HEAD
git diff --stat cb60ee2d375aafc60dd2f8a5d696f1f375321cff 870b3321c4e3d46597848ae5d3faf40395229621
. ./scripts/development-env.ps1
cargo test --manifest-path src-tauri/Cargo.toml --test reader_integration --test document_protocol -- --nocapture
npm.cmd audit --audit-level=high --json
rg --files dist | rg 'pdfjs|pdf.worker' | Measure-Object
rg --files dist | rg -i 'quickjs|sandbox'
```

- Árbol limpio, HEAD coincide con el corte asignado.
- Cargo exit 0: document_protocol 3/3 y reader_integration 10/10 entradas. La décima entrada incluye el helper de reapertura; no se presenta como una décima prueba independiente de comportamiento. Los resultados no cubren SEC-T03-1 ni SEC-T03-2.
- npm audit exit 0: 0 advisories reportados, 241 dependencias inventariadas. No significa ausencia de vulnerabilidades desconocidas ni auditoría Rust equivalente.
- Inventario de recursos anterior; búsqueda quickjs/sandbox sin coincidencias.
- Se inspeccionaron diff y archivos mediante git/Get-Content/rg. Un intento de leer capabilities/default.json falló porque no existe; se leyó después el archivo real main-local.json. Ese fallo no se contó como comprobación positiva.

No se ejecutó npx eslint --plugin security: ese linter/plugin no forma parte del scaffold fijado y no se instalaron herramientas adicionales. No se ejecutó auditoría nueva de advisories Cargo. No se añadió un reproducer ni se cambió una prueba del autor.

## Límites

Sin GUI, selector nativo, biblioteca personal, instalador ni modificación externa. No se probó la CSP efectiva de release o WebView2, render real de PDF escaneado/decodificadores, ni PDF grande. Se mantiene la separación entre inspección, pruebas Rust/adaptadores, mocks React y aceptación nativa. El informe del autor reconoce estos límites; no se atribuyen resultados de T10 a este corte.
