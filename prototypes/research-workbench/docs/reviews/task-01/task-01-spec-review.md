Spec Compliance: Issues found — desviaciones confirmadas en frontera de aplicación, contratos congelados y propagación del gate.
Task quality: Needs fixes — 4 Important/HIGH, 0 Critical, 0 Minor en este ámbito. Los dos hallazgos de contratos coinciden con la revisión TS y deben consolidarse, no contarse dos veces.

## Alcance y autoridad

Revisión independiente de T01, BASE `dd31de021d9d1ee60c0dd3d60c6347525d0e5903`, HEAD `13f985f212b29008e699fbde1ee00d5a556dd408`. Autoridad: brief central, instrucciones de revisión y clarificación aprobada de PhaseDefinition. Revisión del paquete suministrado por secciones acotadas; recuperación de segmentos cortados por el límite de salida. Lockfiles, inventario de licencias y permisos generados se inspeccionaron estructuralmente. No ejecuté Git, suite ni scripts de producto; checkout, index, HEAD y ramas permanecieron fuera de mi ámbito de escritura.

El juicio cubre cumplimiento global, fronteras y seguridad transversal, configuración y scripts. La revisión profunda de Rust y TS tiene sus propios dictámenes. La ausencia de importador, lector, PRE/P1/P2, búsqueda, exportación, restore o QA de instalación no se considera un defecto de T01.

## Strengths

- Registry cerrado con 63 comandos únicos y exactamente tres implementados: `settings_get_app_info`, `settings_get_library_info` y `settings_get_library_status` (`contracts/manifest.json:295`, `contracts/manifest.json:300`, `contracts/manifest.json:315`). Las otras capacidades no simulan éxito: dispatcher devuelve `UnsupportedCapability` y el cliente corta antes de invoke (`src-tauri/src/transport/commands/mod.rs:153`, `src/shared/adapters/tauri/client.ts:13`). Biblioteca y Conocimiento aparecen deshabilitados (`src/app/App.tsx:9`). Los diez puertos están declarados (`src/shared/contracts/ports.ts:4`, `src/shared/contracts/ports.ts:22`, `src/shared/contracts/ports.ts:30`, `src/shared/contracts/ports.ts:51`, `src/shared/contracts/ports.ts:65`, `src/shared/contracts/ports.ts:81`, `src/shared/contracts/ports.ts:95`, `src/shared/contracts/ports.ts:104`, `src/shared/contracts/ports.ts:109`, `src/shared/contracts/ports.ts:121`).
- Identidad estable, ventana `main`, capability limitada a esa ventana y grupos propios sin APIs genéricas de filesystem, SQL o shell (`src-tauri/tauri.conf.json:5`, `src-tauri/tauri.conf.json:15`, `src-tauri/capabilities/main-local.json:5`). CSP limita scripts/workers al origen empaquetado y conexiones a IPC/protocolo local (`src-tauri/tauri.conf.json:27`). El handler conserva autoridad ACL y etiqueta de ventana (`src-tauri/src/lib.rs:45`); los tests ejercitan autoridad real de Tauri mediante MockRuntime, además de comparar registry y permisos (`src-tauri/tests/desktop_bootstrap.rs:159`, `src-tauri/tests/scaffold_contracts.rs:33`). No encontré una ampliación de privilegios frontend en el diff.
- Actor dedicado con cola 64; PRAGMAs requeridos; UoW `BEGIN IMMEDIATE`; receipts durables y hash canónico (`src-tauri/src/adapters/sqlite/actor.rs:141`, `src-tauri/src/adapters/sqlite/actor.rs:54`, `src-tauri/src/application/unit_of_work.rs:6`, `src-tauri/src/adapters/sqlite/receipts.rs:23`, `src-tauri/src/adapters/sqlite/receipts.rs:28`). La firma SQLite de UoW está explícitamente prescrita por el brief y no se incluye en el hallazgo de fronteras.
- Snapshot utiliza SQLite Backup API, exige autocommit para evitar copia con transacción abierta, incluye metadata/recursos y verifica hashes de documentos referidos (`src-tauri/src/adapters/sqlite/migrations.rs:123`, `src-tauri/src/adapters/sqlite/migrations.rs:139`, `src-tauri/src/adapters/sqlite/migrations.rs:144`, `src-tauri/src/adapters/sqlite/migrations.rs:169`). La ruta de datos se deriva de LOCALAPPDATA, con override de prueba limitado a debug (`src-tauri/src/adapters/windows/paths.rs:35`).
- Generación desde Rust con ts-rs y comprobación de drift/export de declaraciones (`src-tauri/src/bin/generate_contracts.rs:1`, `scripts/generate-contracts.ps1:12`, `scripts/generate-contracts.ps1:14`). La clarificación de fases está representada con campos y enum cerrado (`src-tauri/src/transport/dto.rs:1563`, `src-tauri/src/transport/dto.rs:1589`, `src/shared/contracts/wire.ts:104`). No se interpreta esa declaración como implementación de gates.
- Cambios de `.gitignore` se limitan a tsbuildinfo y schemas, preservando las excepciones archivadas (`.gitignore:23`, `.gitignore:26`). Toolchain exacto fijado y helper con entorno de proceso/cache local, sin comandos de instalación global (`rust-toolchain.toml:2`, `scripts/development-env.ps1:15`, `scripts/development-env.ps1:21`). Los lockfiles usan fuentes npm/crates oficiales con integridad/checksum para dependencias. Licencias se presentan como inventario, con redistribución final diferida (`THIRD_PARTY_NOTICES.md:3`).

## Issues

### Critical

Ninguno confirmado en este ámbito.

### Important

**I1 — [HIGH] El gate puede ocultar el fallo del generador.**

Archivo: `scripts/check.ps1:11`; origen de salida: `scripts/generate-contracts.ps1:9`.

`check.ps1` comprueba el código de npm/Cargo en los pasos anteriores, pero llama al último `.ps1` sin comprobar su resultado. Si `cargo run` del generador termina con un código no cero, el generador ejecuta `exit $LASTEXITCODE`, devuelve control al script llamador y este termina con `Pop-Location`, sin propagar ese código. Ejecutado con `powershell.exe -File`, el gate puede devolver cero aunque no haya verificado generación ni drift. `$ErrorActionPreference='Stop'` no convierte ese retorno en excepción. El log verde existente no cubre esta rama de fallo.

La semántica está documentada por Microsoft: un script llamado con `&` puede actualizar `$LASTEXITCODE`, mientras el proceso que ejecuta un script con `-File` devuelve cero cuando el script termina normalmente. Fuentes: [about_Automatic_Variables](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_automatic_variables?view=powershell-5.1#lastexitcode), [about_PowerShell_exe](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_powershell_exe?view=powershell-5.1#-file----filepath-args).

Fix: comprobar y propagar `$LASTEXITCODE` inmediatamente después de la llamada al generador, o hacer que el generador lance una excepción al fallar su comando hijo. Añadir una comprobación focalizada que inyecte un fallo del generador y observe salida no cero del gate completo, sin repetir toda la suite.

**I2 — [HIGH] Settings consume infraestructura concreta en lugar de un puerto de aplicación.**

Archivo: `src-tauri/src/modules/settings.rs:39`; dependencia directa: `src-tauri/src/modules/settings.rs:42`. El mismo acoplamiento aparece en las firmas de `get_app_info` y `get_library_info` (`src-tauri/src/modules/settings.rs:3`, `src-tauri/src/modules/settings.rs:34`).

El caso de uso requiere `adapters::sqlite::actor::DbActor` y llama directamente a `adapters::sqlite::settings::recovery_required`. Trasladar la query fuera del transporte evita SQL en comandos, pero deja el servicio de aplicación acoplado al adaptador SQLite. `application/ports.rs:2` solo declara reloj e identidad; no existe una interfaz consumida por este servicio para consultar el estado de biblioteca. Contradice las reglas explícitas de que módulos llaman interfaces de aplicación y la aplicación depende de dominio/puertos, con concretos compuestos por `lib.rs` (`docs/architecture/ARCHITECTURE.md:68`, `docs/architecture/ARCHITECTURE.md:98`). Sustituir persistencia o probar el caso de uso exige arrastrar infraestructura y no se congela la frontera prometida a tareas consumidoras.

Fix: declarar un puerto mínimo de consulta del estado de Settings/biblioteca, implementarlo sobre el actor en infraestructura y suministrarlo desde composición. Mantener DTO/envelope en la frontera de transporte o en tipos compartidos explícitos. No hace falta añadir repositorios ni servicios futuros.

**I3 — [HIGH] Opcionales no nullable se ensanchan en los contratos generados.**

Archivo: `src-tauri/src/transport/dto.rs:429` y `src-tauri/src/transport/dto.rs:1735`; resultado: `src/shared/contracts/generated/contracts.ts:35` y `src/shared/contracts/generated/contracts.ts:122`.

`PageDto.total` y `WorkflowGetPhaseDefinitionArgs.version` se generan como `?: number | null`. CONTRACTS v1 exige `total?: number` y `version?: number` (brief central:146 y :224). Sus validators Zod rechazan null (`src/shared/contracts/wire.ts:8`, `src/shared/contracts/wire.ts:119`) y el puerto Workflow mantiene `version?: number` (`src/shared/contracts/ports.ts:33`). El contrato Rust/generado admite una forma que el adaptador rechaza: la generación sin drift puede pasar mientras el artefacto canónico incumple la firma congelada. Estos DTOs completos sí pertenecen a T01 aunque las operaciones futuras estén deshabilitadas.

Fix: representar opcionales no nullable coherentemente en Rust, generación y validators; quitar la ampliación explícita a null y probar omisión/número/null contra las tres capas. Consolidar este hallazgo con el correspondiente informe TS.

**I4 — [HIGH] El validator de backups aplica indebidamente el máximo por PDF al tamaño total.**

Archivo: `src/shared/contracts/wire.ts:87`.

`backupDtoSchema.sizeBytes` usa `.max(524288000)`. El límite de 500 MiB del brief central:150 corresponde a cada PDF; `BackupDto.sizeBytes` no tiene ese máximo (brief central:406). Una biblioteca con dos PDFs válidos de 300 MiB produce un backup válido superior al límite del validator. El adaptador rechazaría su resultado, también a través de `LongOperationDto` (`src/shared/contracts/wire.ts:90`, `src/shared/contracts/wire.ts:93`). Es una restricción incorrecta en el validator asignado a T01; no se pide implementar backups T08.

Fix: conservar entero finito no negativo para el tamaño total, sin el máximo por archivo. Mantener el máximo de PDF en `ImportPreviewDto` y entradas documentales. Añadir fixture de BackupDto superior a 500 MiB. Consolidar con el informe TS.

### Minor

Ninguno añadido en este ámbito; no se hace barrido de estilo ni se repiten sugerencias de los revisores de lenguaje.

## Evidence and Cannot Verify

- Inspeccionados, sin rerun: `work/final-check-1.log:14` muestra 9 tests TS; :58, :71 y :89 muestran 7+12+4 tests Rust. `work/final-permissions-check.log:5` y :23 muestran la comprobación posterior de 16 tests. `work/tauri-build-debug-delivery.log:24` muestra el ejecutable debug nativo final. Son logs de ejecuciones previas; el hallazgo I1 no afirma que esas ejecuciones fallaran, sino que el gate omite una ruta de propagación necesaria.
- Los tests TS ejercitan contratos/invoke simulado, sin montaje de App ni WebView2 real (`src/shared/adapters/tauri/client.test.ts:5`, `src/shared/contracts/fixtures.test.ts:6`). MockRuntime prueba ACL real del framework, no visualización nativa (`src-tauri/tests/desktop_bootstrap.rs:159`). No se atribuye instalación al resultado de ninguno de esos tests.
- La captura de la ventana anterior precede a dos cambios finales y no demuestra el diálogo de segunda instancia ni la ejecución visible del HEAD actual; el implementador lo declara correctamente (`docs/reports/task-01-report.md:108`). No hice una nueva ejecución nativa.
- No verificables por este diff: instalador NSIS en VM limpia/offline, consola de release, uninstall/reinstall/conservación de datos, redistribución final WebView2 y CSP resuelta del ejecutable release. Configuración declarada en `src-tauri/tauri.conf.json:30` y `src-tauri/src/main.rs:1`; pendientes reconocidos como T10 en `docs/reports/task-01-report.md:114`. No bloquean T01 por su ausencia.
- Primera pasada adicional fuera del diff, por riesgo concreto: `ARCHITECTURE.md` para confirmar la regla infringida por Settings; documentación primaria Microsoft para confirmar propagación de exitcode; logs nombrados para contrastar conteos y build. `INTENT.md`, `docs/STATUS.md` y AGENTS se leyeron como autoridad de alcance, sin tratar el estado previo a integración como defecto del worker. El estado limpio y la identidad Git de integración corresponden al controlador; no ejecuté Git para verificarlos.

## Assessment

Task quality: **Needs fixes**. El scaffold mantiene el alcance funcional y las restricciones de seguridad visibles, y aporta pruebas reales de persistencia/ACL. Deben corregirse las cuatro desviaciones confirmadas antes de integrar, consolidando los dos hallazgos de contratos con TS y manteniendo la corrección acotada a T01.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 4 | warn |
| MEDIUM | 0 | info |
| LOW | 0 | note |

Verdict: WARNING — 4 HIGH issues deben resolverse antes de integrar; los hallazgos de nulabilidad y tamaño de backup son duplicados de la revisión TS a efectos de consolidación.
