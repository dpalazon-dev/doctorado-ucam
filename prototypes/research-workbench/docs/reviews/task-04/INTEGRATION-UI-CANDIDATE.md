# T04c — candidato de integración UI

Estado: checks y build aprobados; QA nativa pendiente, merge preparado sin confirmar. 2026-10-05.

Producto aprobado por SPEC/TypeScript:5d35f03b13c4e85a0944296bac20e593cf494b5c, entrega67446412f9c2f82cab63ccb5bf02d3dff695bb84. Integration HEAD702d08b02f2d2655584a932455ac394fb026f8d2; merge preparado sin conflictos, treea85fc499718fe3f75c629ac4b04da3d901233989. Sólo14paths UI/tests/reportes revisados; backend sin cambios. Root leyó delta completo original y ambas correcciones; F1/F2/S2/TS9 cerrados por revisiones finales archivadas. Minor fixtureReader de fix1 permanece, sin impacto en guarda que prueba.

Comandos desde .worktrees/integration:

- powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1: exit0.129UI/11archivos;181entradas Rust incluidoshelpers; tipos, buildfrontend, fmt, Clippy, contratos sin drift. Logsreales work/task-04c-integration/check.*. Root comprobó exit, resúmenes y colas; no se afirma lectura íntegra de cada línea de log.
- Dot-source scripts/development-env.ps1; npm.cmd run tauri:build -- --debug --no-bundle: exit0. Logsreales tauri-build.* en misma carpeta. Advertencia Vite por chunk superior500kB; no fallo ni evidencia de rendimiento medido.
- Git diff unstaged vacío y write-tree idéntico antes/después.

Ejecutable debug congelado: work/qa/t04-native/candidate-32249b7a/research-workbench.exe;27037184bytes; SHA25632249B7A168D8B8AA53E65D82BE11A55FA6EE0B2294DE94682A424CC0921BDEC. Manifest adyacente conserva padres/tree/build; no inventa commit de merge todavía inexistente. Se confirmó guard debug_assertions para RESEARCH_WORKBENCH_TEST_ROOT, no se ha arrancado este ejecutable.

Runner separado en worktree task-04c-native-runner: QA1commit17ca5d5, revisión independiente en curso. Persisten discrepancias del recorrido/esperas y guardas propias respecto al histórico; no apto aún. No tests nativos ni instalación/Windowslimpio se deducen de los checks anteriores. Cierre de T04c y comienzo de T05 esperan validación del recorrido nativo pertinente.
