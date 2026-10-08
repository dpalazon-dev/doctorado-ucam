# T03 fix2 — revisión focal TypeScript

**DONE_WITH_CONCERNS — BLOCK.** Tres Important reproducidos en F2-1. F2-2 queda cerrado por la invalidación al iniciar importación. No se reabren las correcciones aceptadas de cargas PDF, identidad de fichas o paginación.

## Corte y alcance

- Reviewer: `/root/task03_typescript_review`.
- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-03-reader`.
- BASE: `b36ae5ae0529ba429edf7a360ade7319e353e8fe`.
- HEAD: `30d532714d053c09d4021bc89bafbb0168b1602a`; producto `e0a882267d1e5f39bb12f6bf9124d8e33edd4dd8`.
- Fuentes: briefs fix2 y revisión fix2, informe del autor, revisión TS fix1 y contratos/requisitos leídos durante las rondas anteriores. Inspeccionados delta TS/TSX, pruebas, consumidor Reader, cliente IPC real y cambios acotados Library/App/Vite.
- Revisión local BASE..HEAD: staged/unstaged vacíos, HEAD verificado y árbol limpio al terminar. No hay metadata de PR/CI remoto verificable. Sin producto editado, commits, merge, GUI, instalaciones ni subagentes. Única escritura: este informe central.

## F2-TS-01 — Important: «Guardar mi posición» tiene éxito pero no termina el conflicto

**Ruta:** `src/features/reader/useReadingPosition.ts:145-147`, consumidor de esa salida `:313-319,325-334`; bloqueo posterior `:162-163`.

`persist` actualiza revision y limpia uncertain después del éxito, pero conserva `entry.recovery` y su snapshot durable anterior. `resolveConflict("save-local")` lo vuelve a publicar, por lo que la UI conserva el conflicto después de confirmar la elección local. El siguiente cambio normal de página es rechazado por `entry.recovery` antes de llamar al API.

**Probe ejecutado con hook real y React/JSDOM:**

1. UI revision0; durable página9/revision1.
2. save(página2) → Conflict; se lee durable página9/revision1.
3. El usuario elige save-local; CAS revision1 guarda página2/revision2.
4. La vista queda con `status="saved"` y `recovery.kind="conflict"`, cuyo durable todavía es página9/revision1.
5. save(página3) → error Conflict sin tercera escritura: durable sigue página2/revision2, llamadas save=2.

**Impacto:** la resolución explícita parece completada y la cola sigue bloqueada. El panel puede incluso ofrecer «Conservar posición guardada» con el snapshot de página9 anterior al éxito propio. No es un conflicto nuevo del backend.

**Corrección esperada:** completar atómicamente el estado de recuperación tras la confirmación de la intención resuelta y verificar la siguiente escritura ordinaria. No limpiar una recuperación nueva que haya sustituido a esa operación durante una transición concurrente.

**Hueco del test:** `conflict_stops_prequeued_saves_instead_of_rebasing_them` verifica el nuevo requestId/CAS pero termina antes de comprobar recovery=null y un siguiente save. El caso de incertidumbre también acaba comprobando status saved y no comprueba que se haya retirado el conflicto.

## F2-TS-02 — Important: el replay de una posición anterior marca Guardado aunque la posición visible siga pendiente

**Ruta:** `src/features/reader/useReadingPosition.ts:224-240,247-257`; representación `src/features/reader/PaperReader.tsx:200-209`.

Cuando el replay confirma la petición anterior pero entry.local contiene otra posición, el hook crea un conflicto explícito para conservar la intención nueva. Sin embargo, la continuación de éxito asigna `status="saved"` igualmente. Reader representa Guardado a partir de status, sin comprobar recovery ni cuál de las posiciones se confirmó.

**Probe ejecutado:** save(página2) confirma en el backend pero devuelve StorageUnavailable; mientras tanto el usuario se mueve a página3, que queda retenida. RetryUncertain reproduce exactamente página2/requestId original y obtiene éxito. Resultado: durable página2/revision1, local página3, `recovery.kind="conflict"` y `status="saved"`.

**Impacto:** el canvas conserva página3 y el indicador afirma Guardado, aunque solo está persistida página2. Se cumple la identidad del replay, pero se incumple REQ-002-06 y la distinción entre intención local y estado confirmado de F2-1. La presencia de un panel de conflicto no hace correcto el indicador de guardado de la vista.

**Corrección esperada:** mantener el estado visible como pendiente mientras haya una intención local distinta no confirmada; Guardado debe corresponder a la posición actual confirmada. Probarlo también en el Reader real, además del hook.

## F2-TS-03 — Important: reabrir antes de recibir el conflicto deja la recuperación sin controles

**Ruta:** `src/features/reader/useReadingPosition.ts:78-91,94-99,162-163`.

El nuevo hook solo copia recovery del mapa al montar/cambiar de documento. Una respuesta pendiente del hook anterior publica en el mapa global y en el setRecovery de la instancia antigua. No notifica a la instancia que acaba de reabrir el mismo documento. Si esta instancia intenta guardar, detecta el bloqueo en el mapa, pero solo publica error; nunca copia esa recuperación al estado que utiliza Reader.

**Probe ejecutado con IpcResult, sin excepciones de transporte:**

1. Montar A revision0 y comenzar save(página2), manteniendo su respuesta pendiente.
2. Desmontar A y reabrir A con durable revision1 antes de completar el save anterior. El nuevo hook empieza con recovery=null.
3. Resolver el save anterior con Conflict; getReadingPosition devuelve página9/revision1.
4. Guardar página3 desde la instancia reabierta.

Resultado: `status="error"`, `error="Conflict"`, `recovery=null`; la cola global sí está bloqueada. Reader solo presenta Cambios pendientes/error y no monta el grupo «Resolver guardado de posición». La intención no puede resolverse desde esa vista; hace falta otra reapertura para copiar el mapa de nuevo.

**Impacto:** contradice recuperación explícita sin reiniciar y continuidad al cerrar/reabrir mientras termina el trabajo admitido. Es una regresión de la recuperación nueva de fix2, no una reapertura especulativa del aislamiento A/B previamente aceptado.

**Corrección esperada:** que la vista actual observe/rehidrate el estado vigente por documento cuando termina una operación anterior, manteniendo la persistencia independiente de la vida del componente y descartando notificaciones de otros documentos. Añadir la secuencia de desmontaje/reapertura con respuesta tardía tanto para Conflict como para incertidumbre.

## Sospecha de rechazo directo de Promise: no se eleva a Important

El probe con una implementación simulada de ReaderApi que lanza/rechaza directamente confirma que `persist:125` salta las ramas que publican recovery y que el siguiente save crea otro requestId. Sin embargo, `src/shared/adapters/tauri/client.ts:15-18` atrapa el rechazo de invoke y lo devuelve como IpcResult StorageUnavailable (o envelope estructurado), ruta manejada por el hook. No se ha identificado un origen alcanzable de esa excepción directa en la composición de producción actual.

Se registra como observación defensiva del puerto y de sus mocks, no como defecto bloqueante de producto ni ampliación obligatoria del alcance. Los tres Important anteriores usan envelopes normales y sí son alcanzables con el adaptador real.

## Cierres y evidencia delimitada

- El escenario fix1 de dos saves preencolados que atravesaban Conflict ya no sobrescribe automáticamente: recovery detiene la cola. Quedan abiertos los tres problemas de resolución/representación anteriores.
- F2-2: `LibraryPage.beginImport` incrementa openIntent y ambos accesos Importar PDF lo usan. Una respuesta de Leer anterior ya no supera el guard. La suite añadida cubre selección pendiente y preview visible sin navegar ni cancelar el token nuevo; inspeccionada, con resultado verde atribuido al gate del autor.
- Reader ofrece botones explícitos para replay, conservar durable y guardar local. El test del Reader comprueba que conservar durable cambia la página y zoom enviados al canvas simulado; esto no es evidencia de layout/WebView2.
- App notifica/consulta estado tras terminales de importación y al entrar Settings, y descarta respuestas de estado por secuencia. Vite añade MIME de JavaScript/WASM manteniendo inventario cerrado. La validación especializada de conformidad/seguridad pertenece a sus revisores; no se sustituye con este dictamen.
- No se reejecutaron gate completo, Rust ni build. El build congelado `32C7DC86458F16D594EDFB5EABC5163EED05C015BC651418E6ACE2701C1D9CD7` y los 59 tests UI del gate son evidencia del autor/orquestador. No se acredita QA nativa fix2 ni instalación.
- Precisión documental: el reporte del autor llama 128 «pruebas de comportamiento», aunque el conteo incluye helpers de proceso. Los logs locales se encuentran bajo el worktree según la comprobación del orquestador. No usar esos enunciados para ampliar la evidencia de esta revisión.

## Comandos propios

1. `git status --short`, `git diff --staged --stat`, `git diff --stat`, `git rev-parse HEAD`: limpio/HEAD indicado.
2. `git diff b36ae5ae0529ba429edf7a360ade7319e353e8fe 30d532714d053c09d4021bc89bafbb0168b1602a --stat` y diff/lecturas focales del delta.
3. `npm.cmd run typecheck`: exit0.
4. `npm.cmd test -- src/features/reader/reader.test.tsx src/features/reader/useReadingPosition.test.tsx`: exit0, 2 archivos/14 tests.
5. Here-string PowerShell enviado a `node --input-type=module`, importando hook real, renderHook/act y JSDOM: exit0. Cuatro probes inline, resultados descritos arriba; sin archivos de tests ni producto modificados. Todos ejecutan cleanup de React.
6. `git diff --check`: exit0; `git status --short`: limpio. `Test-Path node_modules/.bin/eslint.cmd`: false; manifest sin ESLint. No se declara lint ejecutado ni verde.

## Veredicto

**BLOCK** por F2-TS-01, F2-TS-02 y F2-TS-03. Son fallos reproducidos de la recuperación de posición, pese al typecheck y las pruebas focales verdes. Corregir sobre el mismo autor, añadir las transiciones terminales y la reapertura pendiente a las regresiones, y revisar el próximo delta antes de integrar.
