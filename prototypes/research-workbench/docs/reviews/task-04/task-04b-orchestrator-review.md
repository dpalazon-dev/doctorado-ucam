# T04b — lectura del orquestador

2026-10-03. BASE ce15c95dc6404aae1cb158ca6c4a1fe576b70505. Corte producto5c35067f893aa52ec601cc7a02c18bb95b156145; delta posterior929c6e48cd3590cbad52804f32f06a21708929ea leído aparte. Resultado: BLOCKED para merge, pendiente corrección/revisiones/gate.

Root leyó el diff de los31 archivos, implementación completa de Workflow, puertos/helpers, cambios de composición/receipts/Library/migración y pruebas nuevas. La lectura usó git show/diff de cortes inmutables, nunca tomó WIP como aprobación. No ejecutó builds paralelos al autor.

## Hallazgos importantes
1. 5c35067, workflow_repository.rs::advance_in/StartOrientation: avanzar PRE devolvía nextPhase=P1 sin activar current_phase ni iniciar P1. Tests compensaban con touch manual. Autor reprodujo RED, corrigió en929c6e4 y añadió aserción directa de estado/contexto/clock persistidos. Cierre parcial: primera aceptación corregida; ver3.
2. 5c35067, domain/workflow.rs::evaluate_outputs: UNKNOWN de PRE.review_type sólo requería explicación y omitía igualdad con Paper.reviewType actual. Cambio unknown→survey podía conservar gate válido con confirmación obsoleta. Autor reprodujo RED y corrigió en929c6e4; root verificó delta y regresión. Pendiente gate final.
3. 929c6e4, StartOrientation nuevo: exigía P1 NOT_STARTED y devolvía IntegrityFailure al reconfirmar PRE con P1 ya iniciada/invalidadada. La norma exige preservar datos/state/snapshot históricos y renovar reloj/contexto. Autor confirmó RED con P1 completada previamente y está corrigiendo. Abierto al redactar.

## Observaciones remitidas a revisión independiente
- Evaluación/guardado usa definición sin versión fijada (None); DTO de advance/touch fija versión1 en vez de leer pin. Revisar comportamiento ante definición posterior/desconocida y coherencia con getPhase.
- La matriz de migración poblada omite ACTIVE y no compara íntegros todos los metadatos/UUID/hash declarados; las pruebas de IPC usan SQL directo para respuestas y no demuestran el journey import→save→advance completo. Revisar lagunas de aceptación de CAS/no-op/clock overflow y P2 no habilitada; no aprobar por conteo global.
- goBack/touch validan expectedRevision negativo pero no el máximo seguro publicado.
- Domain está separado de I/O, pero adapter mantiene coordinación de casos y carga/serialización de snapshots. Revisar conformidad con puertos/casos de uso publicados sin exigir framework adicional.

## Concurrencia
El primer corte7621da7 tenía falsos positivos: abort sin esperar caller y competidores Pending con actor bloqueado. El delta d959131 hasta5c35067 incorpora abort/join con is_cancelled y caso durante proof con actor libre; misma RequestRegistry, primer poll real y contador de operaciones, después ambos servicios alcanzan colisión de receipt. Root verificó ese delta. Drop observa estado durable en otra conexión y BEGIN IMMEDIATE/ROLLBACK con timeout cero, distinguiendo transacción terminada. Pruebas usan handles sintéticos para ownership; no son validación nativa de un PDF.

## Límites
No aprobación final de T04b, ni integración ni UI nueva. El gate de929c6e4 falló en Clippy según autor (collapsible_if del test); pendiente evidencia final. T04a/T03 y candidato NSIS previos no se atribuyen a este código. Los hallazgos volverán al mismo autor; revisión SPEC independiente en curso sobre5c35067.
