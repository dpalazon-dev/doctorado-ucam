# T02 — revisión independiente de conformidad y calidad

## Spec Compliance

**❌ Issues found.** No aprobar T02 en HEAD `45c4c8c48269495b74241004f92f68acf4770708`: hay cinco hallazgos Important. BASE `94f1796513383ae75486f0b429ab25bae0fb63c1`. No se exige Reader/UI T03, instalación, ni observación GUI del selector como gate de este corte.

**Calidad: Needs fixes.** Los ocho comandos están conectados y el commit final del import es atómico, pero la autoridad de tokens, el diagnóstico de recuperación, una frontera de limpieza y la composición transaccional para T04 incumplen los requisitos vinculantes. La lectura al parser tampoco conserva el límite explícito del buffer.

## Strengths

- `src-tauri/src/transport/commands/mod.rs:147` conecta los ocho comandos Library a adaptadores reales; `transport/commands/library.rs:6` mantiene adaptadores delgados. El diff conserva el registro cerrado y las capacidades Reader/Workflow deshabilitadas (`modules/settings.rs:13`).
- `src-tauri/src/adapters/sqlite/library_repository.rs:350` confirma Paper/autores/Document/COMMITTED bajo el `with_receipt` de `:524`; no comunica éxito antes del commit. El payload original se separa de la normalización (`:198`, `:319`) y la comprobación final DOI/hash se repite dentro del commit (`:386`).
- `src-tauri/src/modules/library/service.rs:82` serializa confirm/cancel por token sin mantener una transacción durante filesystem; `adapters/documents/store.rs:27` ejecuta copia, hash e inspección fuera de UI y actor. `desktop/maintenance.rs:61` y `desktop/lifecycle.rs:184` extienden el drenaje a permisos de operaciones y mantenimiento.
- `src-tauri/src/adapters/documents/pdf_probe.rs:4` valida estructura/primera página/caja y rechaza cifrado mediante trailer, sin buscar `/Encrypt` como texto. `tests/library_integration.rs:413` usa cifrado real, y `:243` cubre xref/object stream e incremental.
- `src-tauri/tests/library_integration.rs:909` inyecta fallo SQL, verifica ausencia de Paper prematuro y recupera después de reabrir actor; `:1110` verifica inventario íntegro y rechazo de mutaciones para esquema futuro. La prueba de autores conserva orden tras reopen (`:1084`).

## Issues

### Critical (Must Fix)

Ninguno identificado en esta revisión de SPEC/calidad. La revisión de seguridad/Rust es un gate separado.

### Important (Should Fix)

1. **El replay de selección vuelve a autorizar un token anterior.** `src-tauri/src/modules/library/service.rs:124` llama `remember_token` cuando recupera un receipt; `:104` le asigna la sesión actual y otras 24 horas. Tras reiniciar, repetir el requestId de una selección STAGING no confirmada permite confirmar/cancelar con una autoridad nueva, aunque el ruling final de TASK02_PORTS, punto 4, exige devolver el resultado original sin renovar autoridad de una sesión anterior. Devolver el preview sin insertar/renovar TokenAuth; solo una selección nativa nueva crea autoridad. Añadir prueba con servicio nuevo: replay conserva preview y confirm/cancel nuevo sigue rechazado. No es plan-mandated: contradice una resolución explícita.

2. **Los issues reales de recovery se pierden antes de Settings.** `src-tauri/src/lib.rs:49` descarta tanto `RecoveryReport` como el error de `reconcile_imports`. `src-tauri/src/adapters/sqlite/settings.rs:34` infiere recoveryRequired únicamente de estados no COMMITTED; una importación COMMITTED cuyo archivo falta, cambió de hash o tiene staged/destino ambiguos genera issue en `modules/library/service.rs:313`, pero Settings informa false si no hay otros pendientes. Esto incumple la conservación/publicación del diagnóstico exigida por el ruling de composición y DATA. Conservar resultado/error seguro del arranque en un estado consultable por Settings y combinarlo con pendientes durables; los papers ajenos pueden seguir operativos. Verificar COMMITTED faltante/alterado y fallo global de recovery a través del estado real consumido por Settings.

3. **Faltan los helpers de aplicación componibles bajo la UoW del consumidor.** `src-tauri/src/modules/library/repository.rs:8` solo ofrece `require_current_revision`; `prepare_import` (`adapters/sqlite/library_repository.rs:212`), `commit_confirmation` (`:350`), `update_metadata` (`:713`) y `archive` (`:766`) son privados dentro del adaptador. Los puertos públicos (`application/library_ports.rs:145`, `:180`) retornan futures que abren/confirmarán su propio receipt/UoW (`adapters/sqlite/library_repository.rs:524`, `:671`), y no aceptan la Transaction de Workflow. T04 no puede archivar Paper con el avance de fase en un único commit consumiendo esta ABI; el reporte que sugiere usar LibraryService preservando una única UoW es incorrecto. Extraer los helpers `*_in_tx` y el repositorio transaccional declarados en el brief/TASK02_PORTS, con reglas de negocio en aplicación y SQL en adaptador. Mantener los wrappers públicos y una única transacción del llamador. Este incumplimiento también explica el nuevo adaptador de 930 líneas y la concentración de reglas de transición/duplicados en infraestructura; no es una objeción meramente estilística ni un defecto plan-mandated.

4. **Recovery de reuseExisting confirma sin limpiar staging.** En `src-tauri/src/modules/library/service.rs:372` toda la ruta de archivos está dentro de `prepared.reuse_paper.is_none()`. Una intención durable reuseExisting interrumpida antes de cleanup llega directamente a commit (`:408`) aunque staging siga presente; la ruta normal sí lo limpia (`:202`). El commit reuseExisting deja COMMITTED con destination_path NULL (`adapters/sqlite/library_repository.rs:366`), y `recoverable_imports` (`:560`) excluye ese registro para siempre. Se acumula una copia duplicada de hasta 500 MiB sin diagnóstico ni limpieza futura. Aplicar también en recovery el cleanup verificado de la ruta normal antes del commit; conservar issue/intent si falla. Prueba focalizada: preparar resolución de duplicado, reconstruir servicio sin efectuar cleanup, reconciliar y comprobar ausencia de staging y un solo Paper/documento.

5. **La lectura al parser no está acotada como exige PDF_VALIDATION.** `src-tauri/src/adapters/documents/store.rs:186` usa `fs::read` sin límite después de terminar la copia y esperar el mutex de validación. El tope de `copy_bounded` (`:114`) restringe solo los bytes copiados antes: si el staging crece durante la espera/lectura, el buffer puede superar arbitrariamente 524288001 bytes. La prueba `growth_during_read_crosses_limit_deterministically` solo ejercita copia, no esta lectura. Usar lectura acotada del staging (máximo + 1 para detectar exceso), rechazar el crecimiento antes de publicar preview y mantener el hash coherente con los bytes validados. Esto es distinto del riesgo de expansión del parser, ya aceptado y documentado; aquí se incumple la cota explícita del buffer de entrada.

### Minor (Nice to Have)

1. **Fixtures de MediaBox introducen también xref inválido.** `src-tauri/tests/library_integration.rs:697` reemplaza/elimina textos de longitud distinta sin regenerar offsets/startxref. El rechazo observado puede provenir de xref antes de resolver MediaBox, por lo que no demuestra aisladamente las tres políticas de caja que atribuye el reporte. Construir PDFs con tabla xref válida para cada caja ausente/degenerada y comprobar una fixture de control. Similarmente, el caso bad_root de `:278` elimina EOF además de cambiar Root. La matriz general tiene buena cobertura, pero estas atribuciones deben afinarse.

2. **archive_restore_preserves_document_and_links no crea links.** `src-tauri/tests/library_integration.rs:1055` verifica UUID/documento/autores/lifecycle y archivo, pero no siembra ni relee relaciones/dependencias; el nombre y el reporte sobrestiman esta prueba. Añadir relaciones sintéticas cuando sus tablas/fixtures estén disponibles, o describir exactamente las invariantes hoy verificadas. El SQL de archive/restore no borra dependencias, por lo que esto es un hueco de prueba, no un borrado observado.

## Cannot Verify

- Selector GUI real y permisos del plugin registrado en un runtime nativo observado: build y callback bridge prueban compilación/puente, no diálogo visible. No se amplió este gate a UI T03.
- Instalación NSIS, equipo limpio, offline, render total y cuota total de RAM: fuera del corte/evidencia. El reporte declara estos límites correctamente (`docs/reports/task-02-report.md:87`).
- Logs rojos TDD individuales: el reporte declara que no se conservaron (`docs/reports/task-02-report.md:79`). La evidencia verde existe; no se confunde falta de log rojo con fallo de las pruebas.
- Semántica exacta del parser strict por defecto: el probe no fija ParseOptions explícitamente; no se inspeccionó la dependencia descargada en esta revisión. Corresponde al reviewer Rust confirmar este detalle antes de certificar política strict.
- La preservación de todos los invariantes en código T01 no modificado y FKs/quick_check reside en esa baseline y su evidencia; no se repitió su revisión ni una suite completa.

## Checks y alcance

Lectura del paquete central BASE..HEAD por porciones; un segmento de lockfile inicialmente truncado por la salida se recuperó sin relanzar Git. No se leyó un archivo de producto modificado completo fuera del diff; una búsqueda focalizada `rg -n` obtuvo localizadores para los hallazgos. No se inspeccionó código exterior al diff: la evaluación del estado Settings, fronteras de commit y recovery se sostiene en los hunks incluidos. Se consultaron AGENTS/INTENT/STATUS, brief/handoff/rulings, TASK02_PORTS, PDF_VALIDATION/NATIVE_PICKER, SPEC-002 y secciones relevantes DOMAIN/DATA/QUALITY. Se aplicó el template Superpowers task-reviewer-prompt proporcionado. No hubo subagentes, cambios de producto/Git, instalaciones ni tests ejecutados.

Evidencia leída en checkout: `work/evidence/check-final.log` y `.exitcode` = 0 (14 UI, 64 entradas Rust incluyendo helper); `work/evidence/tauri-build-debug.log` y `.exitcode` = 0 (Windows debug, sin bundle). Salida sin warnings detectados. Los verdes no cubren los cinco escenarios concretos descritos arriba. Única escritura: este informe central. Quick memory pass sin coincidencias relevantes; no se utilizó memoria como autoridad.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH / Important | 5 | warn |
| MEDIUM | 0 | info |
| LOW / Minor | 2 | note |

**SPEC: Issues found. Calidad: Needs fixes. Verdict: WARNING — resolver los cinco Important antes de integrar según el gate del proyecto.**
