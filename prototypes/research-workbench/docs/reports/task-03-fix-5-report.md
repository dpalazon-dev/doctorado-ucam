# T03 fix5 — proporción visual del canvas PDF

Estado: implementación y gate completados; confirmación de geometría nativa pendiente del orquestador.

## Cambios

- BASE: `cb66da1124e553ddd5958f1d8619dc7a3686324d`.
- Commit de producto: `60891e56f1b31643762b7fb56fb01a44e3b3d9a3` (`fix(reader): preserve PDF canvas aspect ratio`).
- HEAD de producto al iniciar la documentación: `60891e56f1b31643762b7fb56fb01a44e3b3d9a3`.
- Único cambio de producto: se añadió `align-items: flex-start` a `.pdf-page-wrap` en `src/app/tokens.css`. No se modificaron el renderer, dimensiones del raster, zoom, IPC ni contratos.

El diagnóstico nativo previo `run55958ac426eb44d3b3088ff62d65b9b6` registró raster de 612×792 a 100% y 674×872 a 110%, pero altura CSS constante de 304.58334 px. La inspección atribuía la compresión al estiramiento transversal del item flex dentro del wrapper limitado por `max-height`. Informe diagnóstico: `.superpowers/sdd/IMPLEMENTATION/task-03-native-layout-diagnosis.md`; evidencia previa: `work/evidence/task03-native-fix4-executed.log` y `work/qa/t03-native/native-run-55958ac426eb44d3b3088ff62d65b9b6`.

El cambio evita ese estiramiento transversal y permite que el canvas conserve su tamaño intrínseco mientras el wrapper ofrece scroll. Esto todavía necesita medición en WebView2: no se afirma aquí que la geometría ya esté corregida.

## Validación

Entorno de desarrollo cargado con `. .\scripts\development-env.ps1` antes de la build. El gate también carga ese entorno internamente.

| Comando | Resultado | Evidencia |
|---|---|---|
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts\check.ps1` | exit 0; typecheck y build frontend OK; 69 pruebas frontend; pruebas Rust y suites de integración sin fallos | `work/evidence/task03-fix5-gate.log`, `work/evidence/task03-fix5-gate.exit` |
| `npm.cmd run tauri:build -- --debug --no-bundle` | exit 0; binario debug creado | `work/evidence/task03-fix5-tauri-debug.log`, `work/evidence/task03-fix5-tauri-debug.exit` |

El test frontend reportó 10 archivos y 69 tests pasados. El gate Rust pasó, entre otras, las suites `library_integration` (55), `reader_integration` (19), `desktop_bootstrap` (13), `db_actor` (8), `scaffold_contracts` (6), `document_protocol` (4), `schema_diagnostic` (4) y `lifecycle_close` (3). Build Vite informó el aviso conocido de chunk JavaScript mayor a 500 kB; no impidió el éxito. No se añadió un test DOM que imitase una comprobación de CSS: JSDOM no calcula layout flex.

Binario construido: `C:\Users\david\Projects\Research-Workbench\work\cargo-target\debug\research-workbench.exe`
SHA256: `3BF7CAC2920E41C3E7294E3CA414313D8DADBBFCF3C4CD48577F994590966E40`
Tamaño: 25,866,752 bytes.

## Autorrevisión y límites

El delta de producto tiene una sola declaración CSS en el archivo autorizado. La regla mantiene el scroll y centrado horizontal existentes. El fallo visual está demostrado en el binario anterior; el nuevo gate/build verifica compilación y regresiones de software, pero no motor de layout. Root debe repetir el recorrido sintético WebView2 y medir el canvas al 100% y 110%, sus razones CSS/raster y el scroll antes de confirmar la corrección.

No se ejecutó GUI ni se usaron bibliotecas personales. Árbol de trabajo verificado limpio después de los commits de producto y documentación.
