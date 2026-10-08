# T03 fix3 — revisión focal TypeScript

**DONE — PASS del delta TypeScript.** Los tres Important de la revisión fix2 quedan cerrados. No se identifican nuevos Critical/Important en el cambio revisado.

## Corte y alcance

- Reviewer: `/root/task03_typescript_review`.
- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-03-reader`.
- BASE: `30d532714d053c09d4021bc89bafbb0168b1602a`.
- HEAD: `5a512a7bba610e90ecb10d657ed1b36095df4078`; producto `e268862f61f7896f2efe6baab65b25f01c4c9f18`.
- Fuentes: brief fix3, informe del autor, informe TS fix2 y delta completo del hook/pruebas de Reader. Revisión limitada a las correcciones y sus consecuencias; no se repiten auditorías de módulos cerrados ni revisión Rust.
- Revisión local, árbol limpio y HEAD verificado; sin metadata de PR/CI remoto verificable. Sin edición de producto, commits, merge, GUI, instalaciones ni subagentes. Única escritura: este informe.

## Cierre de hallazgos

### F2-TS-01 — Cerrado: resolución exitosa retira el conflicto y permite guardar de nuevo

`useReadingPosition.ts:328-340` espera la persistencia confirmada y limpia solo la recuperación capturada. La comparación por identidad de `entry.local` conserva una intención posterior en lugar de eliminarla junto a la que acaba de resolverse.

Probe independiente con hook real: initial revision0; durable página9/revision1; save página2 produce Conflict; save-local explícito confirma página2/revision2; la recuperación queda null. El siguiente save ordinario de página3 confirma revision3. Revisions enviadas: `[0,1,2]`; estado final saved y recovery=null.

La regresión añadida también difiere la resolución e introduce página4 mientras esa escritura está pendiente: al confirmar la resolución, la intención nueva se guarda sobre revision2, termina sin conflicto y no se pierde. Esta prueba focal pasó.

### F2-TS-02 — Cerrado: confirmar el replay anterior conserva la posición nueva como pendiente

`useReadingPosition.ts:219,239-259` conserva la distinción entre la intención cuyo receipt se recupera y una intención posterior. La continuación de replay utiliza estado de error/pendiente cuando todavía existe recuperación, en lugar de asignar saved incondicionalmente.

Probe independiente: petición incierta de página2; intención posterior de página3; replay de página2 confirmado. Resultado: durable página2/revision1, recovery conflict, status error. La posición local sigue pendiente y no se comunica como guardada.

La prueba añadida del Reader real verifica que la página visible más nueva permanece en el input, aparece «Cambios pendientes» con «Guardar mi posición» y no aparece «Guardado». Pasó junto con las pruebas del hook. El canvas/render de PDF se simula; no se atribuye a esta prueba evidencia visual nativa.

### F2-TS-03 — Cerrado: la vista reabierta recibe recuperación de trabajo anterior

`useReadingPosition.ts:95-112` registra suscriptores en la entrada correspondiente al documento, hidrata el valor actual al suscribirse y elimina el listener en cleanup. publishRecovery publica al conjunto vigente, sin depender de la instancia que inició el trabajo.

Probe independiente: save A pendiente → desmontar → reabrir A → recibir Conflict y posición durable. La nueva vista recibe recovery conflict. Elegir keep-saved desde ella devuelve página9/revision1 y termina con status saved/recovery=null, sin reiniciar otra vez. Las nuevas pruebas de hook y Reader verifican también el resultado incierto tardío y sus controles.

## Consecuencias e aislamiento

- Probe independiente A→B: se inicia save A, se cambia el hook a B y llega Conflict de A. B conserva status idle, recovery=null y error=null. La suscripción anterior se elimina; el resultado queda disponible en la entrada de A para una futura reapertura.
- La entrada y cola siguen asociadas al documento; los callbacks de notificación no sustituyen la serialización de persistencia. Desmontar una vista retira su suscripción, no cancela una escritura ya admitida.
- La limpieza condicional no borra una recuperación distinta ni una intención creada durante la resolución. La secuencia de notificaciones de la vista sigue usando identidad/generación/secuencia para el estado de guardado.
- No se modificaron IPC, DTO, schema, dependencias ni el consumidor Reader en este delta de producto; el cambio consiste en publicación de recuperación y transiciones terminales del hook.

## Validación propia

Desde el worktree indicado:

1. `git status --short`, `git rev-parse HEAD`: árbol limpio/HEAD indicado.
2. `git diff 30d532714d053c09d4021bc89bafbb0168b1602a 5a512a7bba610e90ecb10d657ed1b36095df4078 -- src/features/reader/useReadingPosition.ts` y diff de sus pruebas/Reader: inspeccionados completos.
3. `npm.cmd run typecheck`: exit0.
4. `npm.cmd test -- src/features/reader/useReadingPosition.test.tsx src/features/reader/reader.test.tsx`: exit0, 2 archivos/21 pruebas.
5. Cuatro probes inline mediante here-string PowerShell a `node --input-type=module`, importando el hook real, React renderHook/act y JSDOM: exit0. Escenarios y resultados descritos arriba; cleanup por probe, sin crear archivos de test.
6. `git diff --check`: exit0; `git status --short`: limpio. ESLint sigue ausente del bin local y del manifest; no se declara ejecutado ni verde. Una consulta auxiliar rg tuvo un error de comillas PowerShell antes de ejecutarse; la consulta corregida y comprobaciones posteriores terminaron correctamente, sin cambiar producto.

## Límites y veredicto

**PASS TypeScript para este delta**, sin hallazgos Critical/Important abiertos de esta especialidad. Gate completo, 67 UI, entradas Cargo y build son evidencia del autor/orquestador, no reejecutados aquí. Las revisiones Rust/conformidad y el gate de integración conservan su propia responsabilidad.

No se lanzó el binario congelado SHA-256 `60130E7090F12D5C145D1D4E966C9A7FE3688278CE6A398080D7F14DAAD168BE`. Este PASS no acredita WebView2 real, selector nativo, layout, instalador ni equipo limpio.
