# T04c corrección1 — revisión estática del orquestador

2026-10-04. Corte37169112abd10d7639265af589b76904e734b290, BASE fix1c14c97dd8cfa2209e5596ef5ef8c39540973ad99. Root leyó delta completo de los cinco archivos, incluidos todos los tests. Árbol limpio observado antes del gate. No se ejecutaron tests/build propios; el autor está ejecutando check.ps1. Se difiere el build Tauri adicional mientras se contrasta F1; no se edita el corte congelado.

## F1 HIGH — refreshRequired sobrevive a una carga correcta sin ofrecer recuperación

useWorkflow.ts:223–250: sólo reload limpia state.refreshRequired; el efecto de montaje llama loadPaper/loadPhase directamente. Escenario: save A se confirma mientras A está desmontado y no hay listener; fija refreshRequired=true. La recarga del hook viejo retorna false por mounted=false y finalmente libera busy. Al volver después a A, el montaje carga Paper y fase correctamente, pero no limpia el flag. Guardar/evaluar/transiciones quedan bloqueados, error=null y Reintentar carga no existe. Variante: confirmación+refresh fallido, salir y volver después.

Segunda vía al mismo callejón: refresh getPaper falla con flagtrue; usuario consulta otra fase; loadPhase borra error, pero no actualiza Paper ni limpia refreshRequired. Se pierde el botón de recuperar sin resolver la necesidad de relectura completa. La sesión no tiene forma explícita de salir de ese estado.

Corrección mínima a decidir con autor después de revisión: toda hidratación completa y vigente debe reconciliar el flag de esa sesión; cuando siga requerido, conservar acción explícita de recuperación aunque se consulte fase y se borre un error visual. No limpiar flag sólo por loadPhase parcial ni sin las guardas de generación actuales. Pruebas: resolver guardado mientras A ausente y volver después; refresh fallido→salir/volver; refresh fallido→consultar otra fase→recuperar Paper sin reenviar mutación. Revisor independiente debe reproducir/contrastar antes de cerrar.

## F2 MEDIUM — preview completa queda vigente después de GateBlocked

useWorkflow.ts:409–412 marca gateRejected y limpia pendingAction, pero no invalida el gate completo que habilitó el avance. PhaseWorkspace conserva canAdvance porque no mira gateRejected. Resultado: UI muestra evaluación completa y ofrece de nuevo completar justo después de que backend rechazó su gate, hasta que el usuario reevalúe. No hay pérdida durable: backend vuelve a comprobar. Problema de coherencia de preview y guía operativa.

Revisar si debe invalidarse la preview tras el rechazo confirmado para exigir reevaluación explícita antes de otro advance. Mantener fase/datos y no fabricar GateEvaluationDto. Prueba: complete preview→GateBlocked→no avance habilitado ni preview completa vigente; reevaluación explícita satisfactoria puede volver a habilitarlo. No se solicita añadir retry automático.

## Cierres y límites observados

S1 admite NEEDS_REVIEW y hay tests PRE/P1. S3 guarda global/error; S4 contador no vuelve a cero; S5 empty distinto de saved/PENDING; S6 consulta P2 renderiza datos; S7 comparación de cuatro campos y lecturas de contexto; S8 elimina lectura automática en catch de mutación; TS9 reload completo en botón; TS10 permite retry archivado+writable y conserva lifecycle leído. Los cambios apoyan esos cierres por lectura, sujetos a revisión independiente; S2 queda abierto por F1.

R1 ahora entrega envelope de éxito tipado. Fixture usa pageIndex0 y file:///synthetic.pdf, fuera del contrato real de posición/protocolo aunque son innecesarios en la rama que la guarda debe descartar; ajustar a valores plausibles como mejora menor si se vuelve a tocar fixture. No se atribuye prueba de renderer PDF a ese caso. Los nuevos tests de remount resuelven la operación cuando A ya está montado de nuevo; falta la ventana F1 con cero listeners.

No GUI/candidato corregido/instalación observados. El runner QA sigue en preparación independiente. Root conserva AGENTS externo, producto congelado y límites del autor. Un intento de leer brief central desde cwd del worktree falló por ruta; los hallazgos se obtuvieron del diff/git y rg correctos, no de ese error.
