# T03: lector nativo corregido y verificado

Fecha2026-10-03. Fix5 integrado en `6ad3aa475f72f30798907692f25445361a99ad8b`. BASE del autor `cb66da1124e553ddd5958f1d8619dc7a3686324d`, HEAD `9aee94506cadb4b41571a3691c37643df4c6a055`; cambio de producto exactamente una regla CSS. Revisiones CSS y runner PASS adyacentes. Autor: gate completo69frontend/130 entradasRust incluidos2helpers, builddebug0. Integración: merge preparado, buildfrontend0, diffcheck0; se retiraron únicamente dos espacios de hard-break Markdown del informe. El código runtime/src/contracts del merge coincide con el worker. Commit confirmado después del resultado nativo.

## Ejecución verificada

QA ejecutó una vez con autorización del orquestador `native-qa-fix5.ps1 -ExecuteAfterReview`; runner SHA `1F986F2AA515F6257981122B00E280E0B5D354C5378632206806002C721B119A`. Binario congelado `3BF7CAC2920E41C3E7294E3CA414313D8DADBBFCF3C4CD48577F994590966E40`,25866752bytes. Fixture/driver/runtime mantienen los pins del recorrido anterior; WebView2/EdgeDriver154.0.4258.53. No instalación ni biblioteca personal.

Run `2f04a33846f4449a9e6d254c7128bd60`; evidencia bajo `work/qa/t03-native/`. `native-result.json` y stdout:699440bytes, SHA `82EFDBBC9BD9F23FFD78A40105DE484B4CFFDAFA896AA8D41D1D723EF96D4B5D`. Exitcode0 leído por root de `evidence/exploratory-fix5-b640d0e7b4364484a3eb09becd07e40e/exitcode.txt`.

- Importación con selector y UI reales, PDF administrado íntegro.
- Fetch HTTP200/application/pdf/968bytes, cabecera%PDF-1.4 yEOF; raster distinto de páginas1/2.
- A100%: raster612x792, CSS611.994x791.994. A110%: raster674x872, CSS673.996x871.994, también al reabrir. Las cuatro medidas conservan proporción; error relativo máximo0.00000220 frente tolerancia0.01.
- Ratioszoom CSS ancho1.101311/alto1.101011. Scroll wrapperclient549x329, contenido698x896: extremos horizontal149 yvertical567 alcanzados; restauración de scroll confirmada.
- Cerrar lector/reabrir conserva página2/110%; no se presenta como reinicio de proceso completo.
- Cierre de app: CloseMainWindow true, WaitForExit true43.6201ms, ExitCode0 leído antes de DELETEsession. Cleanup completo, sin errores, procesos propios ausentes y puertos33816/33817 libres. Sin errores JS registrados.

Root leyó el JSON, verificó hashes reales y vio la captura de página1 con texto sin compresión. Capturas archivadas: `native-fix5-page1.png` SHA `5CB96001CFE299B424264FE621A179F3E9BD0BA310C85AED7EF1FD70B8B39C91`; `native-fix5-page2-zoom110.png` SHA `69B75DBB381AEE0F43A977120DBA4ACC94DCC13A308FE5DC424FE099EEC5038D`. Reapertura05 conservada en run, SHA `716A87A647B1CB4A532A408CDCD419464235E2762EE26DE5F618631025728A7C`.

## Límite

Se cierra el defecto UTC y la deformación visual de este recorrido sintético nativo. Esto no prueba instalación, máquina limpia/offline, reinicio de la app conservando posición ni todos los tipos posibles de PDF. El siguiente candidato NSIS se construye desde el commit integrado limpio6ad3aa4, sin modificar/sustituir el setup histórico03682f9d.
