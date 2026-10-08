# T02 — integración del backend Library

Integrado el 2026-10-02 en `integration/v0.1`, commit `c4436bdf611b54114ed969fb71abe9e78b883d58`.

- Autor: task02_library, Luna; orquestador: este chat.
- BASE original: `94f1796513383ae75486f0b429ab25bae0fb63c1`.
- HEAD aprobado del autor: `c2b35e21f7bd2f785670b4544b741908d58fc472`.
- Target limpio anterior al merge: `67697523944c85a56e22a95fed67ce0860c5557b`, con T01 integrada.
- Revisiones finales independientes SPEC, Rust y seguridad: aprobadas, sin Critical/Important/Minor abiertos. Informes fix4 en este directorio; seguridad cubrió el acumulado fix3+4 y los cierres anteriores conservan su evidencia.

El orquestador leyó los diffs completos de las sucesivas entregas y los resultados. Preparó `git merge --no-ff --no-commit c2b35e21f7bd2f785670b4544b741908d58fc472` sin conflictos. Ejecutó los comandos siguientes sobre el árbol combinado, comprobó ausencia de drift y confirmó el commit únicamente tras su éxito.

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
. ./scripts/development-env.ps1
npm.cmd run tauri:build -- --debug --no-bundle
git diff --cached --check
```

Todos exit0. El gate incluye TypeScript, Vitest14/14, build web, fmt, Clippy con all-targets y warnings prohibidos, Rust y contratos generados sin cambios. Rust: unit12, actor8, bootstrap13, Library55, lifecycle3, contracts6, schema4, settings1; 102 entradas del runner, equivalentes a101 comprobaciones de comportamiento y un helper hijo. No fallos ni tests omitidos en las suites finales.

Logs y exitcodes reales: [gate](evidence/task02-integration-check.log), [build nativo](evidence/task02-integration-native-build.log). El binario debug se generó en `work/cargo-target/debug/research-workbench.exe`, SHA256 `C3FB989E17B20640FB4C530CC08959CDC839A72E5DF54E5442827F20837924D8`. Ese path de caché puede cambiar con fases posteriores y no identifica por sí solo una entrega congelada.

Resultado: importación PDF recuperable, metadata/archive/restore, receipts y auditoría, validación y guardas Windows, cierre coordinado y diagnóstico del backend. La interfaz Library y Reader corresponde a T03. No se abrió una nueva ventana en esta integración; no se acredita selector GUI, instalador NSIS, ejecución offline en Windows limpio ni resistencia a pérdida de alimentación. Las pruebas utilizaron fuentes/bibliotecas sintéticas.

Main conserva documentos/hitos; la promoción del piloto espera T03 y T10-piloto. El cambio ajeno de AGENTS.md se preservó sin incluirlo en los commits de esta tarea.
