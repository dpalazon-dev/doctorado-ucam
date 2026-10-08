# T03 — integración local revisada

Fecha: 2026-10-03. Orquestador: /root. Autor de producto: /root/task03_library_reader.

- BASE original de tarea: cb60ee2d375aafc60dd2f8a5d696f1f375321cff.
- HEAD final del autor: 5a512a7bba610e90ecb10d657ed1b36095df4078; último producto e268862f61f7896f2efe6baab65b25f01c4c9f18.
- Target de integración limpio: 6af0faf0d077fa09fc6a37501cc0be092874ce3c; dependencia T02 c4436bdf verificada por ancestry.
- Merge preparado con `git merge --no-ff --no-commit 5a512a7bba610e90ecb10d657ed1b36095df4078`, sin conflictos.
- Merge confirmado únicamente después de checks: **88bc912f4fd0e81e31c5bb0f9ab0801fecc40d78**. Árbol limpio al terminar.

## Revisión y validación

El orquestador revisó el delta completo de la tarea y sus tres correcciones. Las revisiones finales Rust, TypeScript y SPEC están archivadas junto a este informe. Seguridad fix2 dejó solo R2a compartido; el cambio final afecta al wrapper de test y Rust fix3 lo cierra con evidencia de primer poll Pending tras admisión real. Se conservan los cierres de asociación documental, persistencia atómica, errores/duplicados, recuperación y recursos locales. No quedan hallazgos críticos/importantes abiertos en el corte integrado.

Comandos sobre el merge preparado, desde .worktrees/integration:

1. `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1` — exit0. Frontend 67/67 en 10 archivos; Rust 128 entradas en 10 targets, incluidos dos helpers de procesos hijos. Typecheck, formato, Clippy, build web y generación de contratos sin drift incluidos.
2. `. .\scripts\development-env.ps1; npm.cmd run tauri:build -- --debug --no-bundle` — exit0. Aviso de chunk Vite mayor de500kB, sin fallo.
3. `git diff --exit-code`, `git diff --cached --check` y ausencia de archivos no versionados — sin cambios inesperados antes de confirmar.

Logs originales en `work/evidence/task03-integration-gate.log/.exit` y `task03-integration-native-build.log/.exit` bajo la raíz principal. Un primer intento de redirección falló antes de ejecutar el gate porque no existía ese directorio; se creó explícitamente y la ejecución válida posterior produjo los logs y exit0 anteriores. No se considera esa primera llamada una prueba.

Binario integrado congelado: `work/qa/t03-native/candidate-69fd578d/research-workbench.exe`, SHA256 `69FD578D817849DFB60F2B0D15A82DE0A66686EF6D1F9D79F946D5978931D938`. Se verificó copia/hash; distinto del binario del worktree del autor, que se conserva separado. No se arrancó este nuevo binario durante la integración.

## Límites y siguiente corte

La evidencia nativa anterior pertenece al candidato fix1. Un diagnóstico adicional del selector obtuvo su estructura real: Edit1148 y Button1 propios, presentados como Pane sin Value/Invoke por UIA. Inventario completo dentro de límites; cierre del diálogo observado, misma app exit0 antes de DELETE session y cleanup completo. No seleccionó archivo ni validó Reader. QA prepara un ajuste dirigido a esos controles, con el nuevo candidato integrado identificado arriba.

Esta integración no acredita dibujo PDF real, funcionamiento de fallback WASM, instalador o equipo limpio. T10-piloto debe producir el candidato NSIS y registrar por separado la evidencia instalada disponible. Se conserva el orden de fases; no se inicia PRE/P1 antes del corte de empaquetado del piloto.
