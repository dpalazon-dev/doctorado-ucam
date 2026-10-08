# T03 — diagnóstico read-only de apertura nativa

Estado: DONE. Causa demostrada en el payload persistido y reproducida con el validador real. No se modificó código de producto ni SQLite; no se inició la app ni se operó su UI.

## Corte y evidencia

Código inspeccionado: `.worktrees/integration`, HEAD `f2d39a25feb81141b51ca6cf028777b38503818b` (T03 integrada `88bc912`). Run autorizado: `work/qa/t03-native/native-run-447da86b3bf44d14ba72c0d18555ee9f/native-result.json`; SQLite abierto mediante `new DatabaseSync(path, {readOnly:true})`, exclusivamente `synthetic-library/research.sqlite` de ese run. OpenViking no consultado; peer no verificado; fuentes locales actuales.

El run confirma importación y tarjeta real, después permanece Biblioteca con «No se pudo completar la operación. Revisa la biblioteca.» y sin canvas. No alcanza PaperReader/protocolo. Los logs locales sólo muestran arranque y avisos de WebView2; no capturan un envelope IPC.

La base conserva:

- Paper `28430cda-489f-4bb8-a49e-c49103ed2b54`, Document `483816d8-2fa0-4a94-9243-ec95ca070c15`, PDF 968 B, SHA256 `a65c300b22ebe0a8cdcd7253c47be5b8752680470b9d3b89f6bd4a5d57703e95`.
- Receipt `reader_open_paper`, requestId `ca88f0da-dc80-4c3e-b259-ced123e56508`, result_json completo y documentUrl `http://research.localhost/483816d8-2fa0-4a94-9243-ec95ca070c15`.
- `papers.last_opened_at` y `result_json.paper.lastOpenedAt`: `2026-10-03T14:36:22.575865800+00:00`.
- `app_session.last_opened_paper_id` apunta al Paper; auditoría `reader.paper_opened` y `reader_open_paper` presente.

Esto acredita que dispatch, servicio, prepare_open, apertura verificada y transacción de confirmación se ejecutaron. No hay base para atribuir este fallo a ACL, sharing o resolución de ruta: la apertura verificada precede a confirm_open y el receipt ya existe. No se afirma una captura directa de la respuesta enviada por WebView2.

## Causa y cadena exacta

1. `src-tauri/src/adapters/sqlite/reader_repository.rs:166`: `SqliteReaderPersistence::confirm_open` pasa `chrono::Utc::now().to_rfc3339()` a `open_paper_in_tx`.
2. `src-tauri/src/application/reader.rs:33` llama `record_open_activity_in_tx`. `application/library.rs:32` escribe el mismo `now` mediante `set_last_opened_at`; `adapters/sqlite/library_repository.rs:318` lo devuelve sin normalizar en PaperDto.
3. `src/shared/contracts/wire.ts:4`: `utcSchema=z.iso.datetime({offset:false})`. `paperDtoSchema` línea23 exige ese esquema para lastOpenedAt; `openPaperDtoSchema` línea35 incluye PaperDto.
4. `src/shared/adapters/tauri/client.ts:18-19` rechaza el envelope si `ipcSchema(payload).safeParse(response)` falla. Devuelve `IntegrityFailure` mediante safeFailure, cuyo mensaje línea9 coincide exactamente con el observado. El mensaje de AppError backend para IntegrityFailure es distinto.
5. `src/features/library/LibraryPage.tsx:60-66` muestra ese error en Biblioteca; sólo llama `onOpenPaper` si result.ok.

El dato es RFC3339 UTC válido, pero no satisface la representación Z que acepta actualmente el wire. El error concreto reproducido es único: path `[paper,lastOpenedAt]`, `invalid_format`, `datetime`, `Invalid ISO datetime`. Sustituir sólo `+00:00` por `Z` en una copia en memoria hace que el DTO completo pase. La precisión de nanosegundos no es la causa.

## Reproducción mínima ejecutada

Desde `.worktrees/integration`, Node v24.14.1, PowerShell here-string enviado a `node --input-type=module`:

```js
import {DatabaseSync} from 'node:sqlite';
import {openPaperDtoSchema,paperDtoSchema,ipcSchema} from './src/shared/contracts/wire.ts';
const db=new DatabaseSync('C:/Users/david/Projects/Research-Workbench/work/qa/t03-native/native-run-447da86b3bf44d14ba72c0d18555ee9f/synthetic-library/research.sqlite',{readOnly:true});
const row=db.prepare('SELECT request_id,result_json FROM operation_receipts WHERE command=?').get('reader_open_paper');
const value=JSON.parse(row.result_json);
const parsed=openPaperDtoSchema.safeParse(value);
console.log(JSON.stringify({receipt:row.request_id,parsed:parsed.success,issues:parsed.success?[]:parsed.error.issues},null,2));
const env=ipcSchema(openPaperDtoSchema).safeParse({contractVersion:1,requestId:row.request_id,ok:true,data:value});
console.log(JSON.stringify({envelopeAccepted:env.success}));
const normalized=structuredClone(value);
normalized.paper.lastOpenedAt=normalized.paper.lastOpenedAt.replace('+00:00','Z');
console.log(JSON.stringify({samePayloadOnlyZuluChanged:openPaperDtoSchema.safeParse(normalized).success,originalPaperAccepted:paperDtoSchema.safeParse(value.paper).success}));
db.close();
```

Resultado Node exit0: `parsed:false`, `envelopeAccepted:false`, `samePayloadOnlyZuluChanged:true`, `originalPaperAccepted:false`. No escribió DB ni una copia de ésta. Se intentó adicionalmente cargar el cliente completo mediante bundling en memoria; no se ejecutó porque `esbuild` no es dependencia resoluble (`ERR_MODULE_NOT_FOUND`). No se considera ese intento una prueba del cliente. La reproducción exitosa anterior sí ejecutó los schemas reales, y el comportamiento posterior del cliente está trazado por lectura.

## Alcance concreto y corrección para el autor

Existen exactamente dos productores `Utc::now().to_rfc3339()` en módulos Reader:

- `reader_repository.rs:166`, apertura: persiste `papers.last_opened_at` y lo incluye en PaperDto/receipt.
- `reader_repository.rs:219`, guardado: pasa el formato a `save_reading_position_in_tx`; `application/reader.rs:94` lo pone en ReadingPositionDto.updatedAt y lo persiste/recibe. Al llegar a guardar, ese resultado también fallaría `readingPositionDtoSchema` (inferencia directa del código, no recorrido nativo observado).

Recomendación acotada: usar el formato canónico ya empleado por Library y receipts, `to_rfc3339_opts(chrono::SecondsFormat::Millis, true)`, en esos dos productores. No hace falta ampliar permisos ni relajar el schema. Revisar una regresión que cruce resultados del backend real con el wire real, tanto apertura como guardado, y comprobar navegación/render nativo después.

Lecturas que conservan el fallo si sólo se corrigen escritores:

- Paper DTO desde Library get/list y Reader last_opened sigue devolviendo last_opened_at persistido sin normalizar. En este run paperDtoSchema rechaza también ese Paper, demostrado.
- Reader position obtiene updated_at directamente de `reading_positions` (`reader_repository.rs:75`); el default usa Document.importedAt, que en este run sí termina en Z.
- `with_receipt` (`adapters/sqlite/receipts.rs:23-29`) devuelve result_json previo sin ejecutar action. Reader prepare_open también deserializa su receipt previo. Cambiar el escritor no repara un replay del mismo requestId. Una nueva apertura con requestId nuevo puede actualizar last_opened_at, pero no reescribe receipts históricos ni posiciones previas.

No se ha modificado ningún timestamp ni receipt. Los datos afectados observados son sintéticos pre-release; el orquestador decide explícitamente su manejo. No conviene ocultar esta limitación dando por reparada la biblioteca antigua con sólo cambiar escritores.

Cobertura que permitió escapar el fallo: `reader_integration.rs:292` sólo exige last_opened_at.is_some(); el test de routing del cliente (`client.test.ts:23-27`) usa lastOpenedAt:null y fechas Z prefabricadas. No valida el dato que genera confirm_open real contra Zod.

## Límites y autorrevisión

Sólo informe nuevo, lectura selectiva de código y evidencia sintética. Sin subagentes, merges, configuración, instrumentación, lanzamiento de aplicación, reparación de SQLite ni acceso a bibliotecas personales. La causa explica el bloqueo anterior a PaperReader; no demuestra que PDF.js, CSP, protocolo o canvas funcionen una vez corregida. Debe retomarse QA nativa desde un corte corregido y bibliotecas de prueba controladas.
