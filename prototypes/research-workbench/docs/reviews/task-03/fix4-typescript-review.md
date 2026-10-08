# T03 fix4 — revisión focal TypeScript UTC

**DONE — PASS.** No hay hallazgos Critical/Important en el delta TypeScript revisado.

## Identidad y alcance

- Reviewer: `/root/task03_typescript_review`.
- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-03-reader`.
- BASE: `5a512a7bba610e90ecb10d657ed1b36095df4078`.
- HEAD: `cb66da1124e553ddd5958f1d8619dc7a3686324d`; producto `55407d12d6b7ca403b2ab2c10390e524f43ffdbd`.
- Fuentes: brief fix4, brief de revisión, informe del autor, diagnóstico nativo central, ADR-019 y CONTRACTS del repositorio principal. Revisados `wire.ts`, `wire.test.ts`, `client.test.ts` y el validador ISO de la versión instalada de Zod como contexto.
- Revisión local BASE..HEAD, árbol limpio; sin metadata de PR/CI remoto verificable. Sin edición de producto, aplicación/GUI, subagentes, merge ni cambios de configuración. No se reabren revisiones anteriores de T03. Única escritura: este informe.

## Resultado

`src/shared/contracts/wire.ts:4` conserva el validador ISO completo de Zod y permite offsets antes de restringir el resultado a los sufijos `Z` o `+00:00`. La condición no sustituye la validación de calendario/hora. No hay coerción, Date, transformación ni normalización: se conserva la cadena recibida y su precisión.

El delta no relaja otras propiedades de DTO/envelope. `-00:00`, offsets distintos de cero, fecha/hora local, valores no string y fechas/horas inválidas siguen rechazados. La implementación instalada de Zod comprueba mes/día/año bisiesto, rango de hora/minuto/segundo y segundos en timestamps calificados; no se ha supuesto ese comportamiento a partir del nombre de la función.

La nueva prueba wire comprueba representaciones `Z` y `+00:00` de nueve decimales con igualdad exacta, así como rechazos. La nueva prueba del cliente usa un Paper con `lastOpenedAt` realista no null y el timestamp exacto del diagnóstico `2026-10-03T14:36:22.575865800+00:00`; pasa por `createTauriApis` para open/last/get/save y conserva los timestamps de Paper/Position. Las pruebas anteriores del cliente con fechas Z y envelopes inválidos permanecen verdes.

## Comprobación independiente del receipt que originó el fallo

Leí exclusivamente la base sintética identificada en el diagnóstico, con `node:sqlite DatabaseSync(..., {readOnly:true})`, sin ejecutar la aplicación ni escribir datos:

`work/qa/t03-native/native-run-447da86b3bf44d14ba72c0d18555ee9f/synthetic-library/research.sqlite`.

El `result_json` persistido de `reader_open_paper`, con `lastOpenedAt=2026-10-03T14:36:22.575865800+00:00`, pasa ahora por el `ipcSchema(openPaperDtoSchema)` real. `assert.deepStrictEqual(parsed.data, original)` pasa; lastOpenedAt y updatedAt de posición conservan exactamente sus strings. La primera comparación con JSON.stringify detectó distinto orden de claves del objeto parseado; se comprobó igualdad estructural para distinguir ese orden de una modificación de datos. No se normalizó ni reescribió el receipt.

Probe adicional del schema real:

- Cuatro fechas válidas preservadas exactamente: año bisiesto 2000 con nueve decimales y +00:00, bisiesto 2024 con Z, Z con milisegundos, +00:00 sin fracción.
- Doce entradas rechazadas: 29 de febrero de 1900/2026, 31 de abril, hora24, minuto60, ausencia de segundos, -00:00, +00:01, ausencia de zona, texto con sufijo +00:00, null y número.
- Propiedades desconocidas rechazadas tanto en OpenPaperDto como en el envelope.

## Validación propia

Desde el worktree indicado:

1. `git status --short`, `git rev-parse HEAD` y diff BASE..HEAD: limpio y HEAD indicado; inspección completa del delta TypeScript.
2. `npm.cmd run typecheck`: exit0.
3. `npm.cmd test -- src/shared/contracts/wire.test.ts src/shared/adapters/tauri/client.test.ts`: exit0, 2 archivos/15 pruebas.
4. Probes inline mediante here-string PowerShell a `node --input-type=module`, importando schemas reales y leyendo el receipt sintético en readOnly: exit0, resultados descritos arriba. Conexión cerrada en finally, sin archivos de test creados.
5. `git diff --check`: exit0; `git status --short`: limpio al terminar.
6. ESLint sigue ausente del manifest/bin local; no instalado ni ejecutado. No se declara lint verde.

## Límites y veredicto

**PASS TypeScript conforme a ADR-019.** La compatibilidad de lectura conserva los datos UTC anteriores y la validación estricta requerida. La revisión Rust de escritores/replay y el gate de integración conservan su responsabilidad propia.

El gate 69 UI y build SHA-256 `0C4BE713C8D2DD9618B8ACD6AF0A3C6E334D957DDA49B1530A36544A81CC7537` son evidencia del autor/orquestador, no reejecutados aquí. Este PASS no acredita todavía render WebView2, selector, protocolo en ventana nativa ni instalación; permite retomar ese recorrido desde el fallo de decoding corregido.
