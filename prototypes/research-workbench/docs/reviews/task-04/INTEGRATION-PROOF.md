# T04a — integración de prueba documental tipada

Fecha: 2026-10-03. Orquestador: Sol en este chat. **Completada e integrada** como corte interno de T04; no acredita todavía Workflow PRE/P1.

- BASE autor: `9188ac5dc4f1cc5ad22347ef1422a2c51d795b1b`.
- HEAD final: `4e65bbec627bea4c50e0b42b91204f8f5cbea68e`; código aprobado en `9ad0bc269909e8013f3add0dc9ac389290fb0b81`. El último commit sólo precisa la fixture documental.
- Integración previa: `64827a5d9419a59332f1ecb1b736e22d92771503`.
- Merge confirmado: `677e05eee97c6a39bd766529762facb14ad5b6e4`.
- Autor: `/root/task04a_document_proof`, Luna; revisores independientes Rust, seguridad y SPEC/calidad. Informes iniciales y focales archivados en esta carpeta.

## Resultado y revisiones

DocumentProofAccess comparte apertura y guardas con Reader. Available posee handle/pins; sólo causas de apertura Missing/AccessDenied/SharingViolation producen Unavailable. Guardas y errores internos conservan Err; Reader mantiene sus códigos anteriores. No hay cambios de schema, wire, UI, permisos ni dependencias.

La revisión inicial SPEC bloqueó la cobertura de inyección por el store. Corrección1 añadió una inyección privada por instancia, exclusiva de tests, consumida dentro del trabajo bloqueante real. Ambos puertos prueban denegación, errores fatales y panic, más apertura positiva posterior. SPEC y Rust focales aprobaron y cerraron todos los hallazgos. Seguridad inicial permanece aplicable a la apertura OS, sin cambios posteriores de permisos/guardas; el delta posterior fue revisado íntegramente por Sol y los revisores focales. Los nombres internos equivalentes fueron aceptados por Sol sin cambiar el contrato consumidor.

Sol leyó el delta completo original y las correcciones. Verificó HEAD limpio, informe coincidente y cambios finales sólo documentales. Se preparó merge con `git merge --no-ff --no-commit 4e65bbec627bea4c50e0b42b91204f8f5cbea68e` sobre integración limpia, sin conflictos.

## Gate ejecutado sobre el merge preparado

`powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1`: **exit 0**. Typecheck, 69 pruebas frontend, build frontend, Rust fmt/Clippy all-targets, 138 entradas Rust incluidos helpers y contratos sin drift. Evidencia: `work/evidence/task04a-integration-gate.log` y `.exit`.

`git diff --cached --check` y diff sin staged: correctos. Comparación de código/configuración con HEAD del autor: sin diferencias. Sólo entonces se confirmó el merge; integración quedó limpia. El autor había construido Tauri debug sin bundle con exit 0 (`work/task-04a/fix1-build-debug.*`); no se atribuye aquí una nueva prueba GUI.

## Límites

Fixtures de tests aisladas, sin bibliotecas personales. Denegación inyectada no prueba ACL real; sharing sí se provocó en NTFS. La nueva fixture del store comprueba acceso/tamaño, no parser PDF. No se ejecutó fallback no Windows, aplicación ni instalador en este corte. El piloto corregido anterior y su QA nativa conservan su propia evidencia; instalación limpia pendiente. T04b consume este puerto y T04c añadirá formularios; T04 completa sigue en curso.
