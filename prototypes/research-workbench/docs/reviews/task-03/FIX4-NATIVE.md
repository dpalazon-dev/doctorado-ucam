# T03: corrección UTC y recorrido nativo del 3 de octubre

Fix4 integrado: `430eb05b6d83c26c9365b586d1067c721b082452`, desde worker `cb66da1124e553ddd5958f1d8619dc7a3686324d`, BASE `5a512a7bba610e90ecb10d657ed1b36095df4078`. Revisiones Rust y TypeScript PASS en los informes fix4 adyacentes. El orquestador leyó el delta completo de siete archivos y preparó merge sin commit; `scripts/check.ps1` terminó con exit0 (69 frontend, 130 entradas Rust incluidos dos helpers), contratos y árbol sin drift. Sólo después confirmó el merge. Logs `work/evidence/task03-fix4-integration-gate.*`.

## Evidencia nativa

Una ejecución por el orquestador de `native-qa-fix4.ps1 -ExecuteAfterReview`, SHA `8297BB98FBDF5099E744D81F4A8B4488E546CC91FEBD37E247D26752659346DC`. Delta respecto al runner revisado30A453: sólo path y hash del ejecutable. Preflight previo sin switch no lanzó app. Binario congelado `0C4BE713C8D2DD9618B8ACD6AF0A3C6E334D957DDA49B1530A36544A81CC7537`, 25866752 bytes, compilado por autor desde fix4; no se atribuye un segundo build al merge.

Run `55958ac426eb44d3b3088ff62d65b9b6`; datos/perfil sintéticos bajo `work/qa/t03-native/native-run-55958ac426eb44d3b3088ff62d65b9b6`. WebView2 y EdgeDriver154.0.4258.53. `native-result.json` SHA `EF74267831CE23F8BEC9109DD43F987FD76B1CBBAB40ACEAE85C23003464CC1F`; stdout y exit en `work/evidence/task03-native-fix4-executed.*`.

- Selector e importación UI completos, copia del PDF idéntica al fixture.
- Protocolo real HTTP200, application/pdf, Content-Length968, cuerpo968 bytes con cabecera PDF y EOF.
- Raster PDF.js con marcadores diferentes de páginas1/2, navegación y zoom110.
- Cerrar el lector y reabrir recupera página2/110; no equivale a reinicio completo de la aplicación.
- CloseMainWindow del proceso propio, WaitForExit true45.166ms y ExitCode0 observados antes de DELETE session.
- Cleanup completo, cero errores, procesos propios ausentes y puertos18712/18713 libres. No errores JS registrados. stderr Chromium contiene warning unregister class Error1412 pese salida0.

## Defecto visual pendiente

PASS funcional exploratorio, no aprobación visual: raster612x792 a100% y674x872 a110%, pero CSSheight304.583 constante mientras CSSwidth611.994→673.996. Las capturas03/04/05 muestran texto aplastado. El diagnóstico `native-layout-diagnosis.md` identifica stretch en eje transversal Flexbox. Fix5 acotada a align-items:flex-start, mismo autor; se requiere repetir medición nativa sin relajar el detector.

El setup anterior03682f9d sigue siendo histórico y contiene el fallo UTC previo. No se ha probado instalación ni máquina limpia. T10 tooling se integró en `a6c0744e94fabcb39143134e94fde7a8c3b1e5cc` desde510edb1: merge preparado, ocho archivos, revisión independiente PASS y suite scripts/tests/build-release.tests.ps1 exit0; no cambia Rust/TS. Reconstrucción del candidato pendiente de fix5.
