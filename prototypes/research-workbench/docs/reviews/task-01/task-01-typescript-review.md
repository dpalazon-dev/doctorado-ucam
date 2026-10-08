Spec Compliance: ❌ Issues found. Task Quality: Needs fixes.
Ámbito: contratos TypeScript/React de T01, transporte Rust conectado con esos contratos, fixtures, shell inicial y configuración npm/TS/PDF.js. BASE `dd31de021d9d1ee60c0dd3d60c6347525d0e5903`; HEAD `13f985f212b29008e699fbde1ee00d5a556dd408`.

## Spec Compliance

La base cumple el alcance funcional revisado: diez puertos y 63 comandos declarados; únicamente las tres lecturas Settings están habilitadas; capacidades futuras visibles como no disponibles. No corresponde exigir Biblioteca, Reader ni APIs futuras implementadas en esta tarea. Sin embargo, dos contratos actuales no son exactos: opcionales generados admiten null indebidamente y BackupDto hereda un máximo reservado a PDFs individuales. Deben corregirse antes de congelar esta base para sus consumidores.

## Strengths

- `src/shared/contracts/ports.ts:4`, `:19`, `:27`, `:49`, `:62`, `:79`, `:91`, `:100`, `:105`, `:121`: los diez puertos conservan métodos, revisiones optimistas, requestId y envelopes; las capacidades futuras siguen siendo contratos tipados.
- `src/shared/adapters/tauri/client.ts:11`: entrada validada antes del IPC, rechazo de extras, payload/envelope comprobados al recibir y comparación del requestId. `:14` evita invoke para comandos no implementados. `client.test.ts:5` a `:9` prueban invoke simulado, bloqueo de capacidades, entrada inválida, correlación y normalización segura sin filtrar el texto privado de la excepción.
- `src/shared/contracts/wire.ts:3`, `:6`, `:177`: UUID canónico, JSON finito con claves de prototipo rechazadas y sobre discriminado estricto con contractVersion literal. `src-tauri/src/transport/dto.rs:14` y `:42` dan contrapartes Rust; los nullable obligatorios usan deserialización explícita.
- `src-tauri/src/bin/generate_contracts.rs:8` y `scripts/generate-contracts.ps1:11`: Rust genera los DTOs TS; el check detecta drift y declaraciones Rust sin exportar. El problema de opcionales señalado abajo demuestra por qué ese check necesita además fixtures de semántica, pues generar fielmente un DTO errado no valida CONTRACTS.
- `src/shared/contracts/fixtures.test.ts:6`, `wire.test.ts:8`, `src-tauri/tests/scaffold_contracts.rs:3`: fixtures compartidas ejercitan Unicode, autor largo y tamaño límite wire de un PDF; el test Rust hace round-trip del envelope. El probe enfocado confirmó que Zod instalado acepta 1000 caracteres suplementarios y rechaza 1001, consistente con el contador Rust de caracteres.
- `src/app/ShellView.ts:1` conserva exactamente la unión solicitada. `src/app/App.tsx:8` carga Settings en paralelo y evita aplicar respuestas tras desmontaje; `:9` ofrece estados de carga/error/diagnóstico, confirma biblioteca local y copia administrada, y deja Biblioteca/Conocimiento deshabilitados.
- `src/shared/adapters/tauri/pdf-assets.ts:1` y `src/main.tsx:5` incorporan el worker local; `work/final-check-1.log:27` registra su salida empaquetada. `tsconfig.app.json:17` activa strict y `package.json:9` declara el check canónico. No se presenta la disponibilidad de paquetes como un lector implementado.

## Findings

### Critical

Ninguno identificado en este ámbito.

### Important

1. **Opcionales TS/Rust aceptan null aunque CONTRACTS y Zod lo rechazan.** `src/shared/contracts/generated/contracts.ts:35` declara `PageDto.total?: number | null`; `:122` declara `WorkflowGetPhaseDefinitionArgs.version?: number | null`. Sus fuentes `src-tauri/src/transport/dto.rs:427` y `:1733` usan `Option` con `#[ts(type = "number | null")]`, aceptando null al deserializar. El contrato exige `total?: number` y `version?: number`, y `src/shared/contracts/wire.ts:8`, `:119` efectivamente rechazan null; `ports.ts:30` también restringe version a number. Por ello un valor válido para el DTO generado no puede atravesar su validador, y un mismo comando tiene dos definiciones incompatibles de entrada. No es ausencia de un servicio futuro: los tipos canónicos exactos son entrega de T01. Corregir la fuente Rust para distinguir omisión de null explícito, generar opcionales number y conservar rechazo de null. Añadir fixtures positiva de omisión/entero y negativa de null en ambas capas; comprobar compatibilidad exacta entre DTO, puerto y esquema.

2. **El validador BackupDto impone un máximo de 500 MiB al backup completo.** `src/shared/contracts/wire.ts:87` aplica `.max(524288000)` a `BackupDto.sizeBytes`, mientras `ImportPreviewDto.sizeBytes` en `:28` es el campo al que pertenece el máximo por PDF. CONTRACTS no establece ese máximo para backups de SQLite y todos los PDFs referenciados; dos PDFs admitidos pueden superarlo. `src-tauri/src/transport/dto.rs:1426` y el tipo TS de BackupDto tampoco expresan esa restricción. Al habilitar Portability, una respuesta correcta sería rechazada y el cliente comunicaría un fallo de almacenamiento aunque el backup estuviera verificado. El probe actual confirma que un BackupDto completo con sizeBytes=524288001 falla. Quitar el máximo por PDF del esquema de backup, conservar entero no negativo y añadir fixture de backup agregado superior al límite individual, manteniendo la fixture que rechaza un PDF sobredimensionado. Corregir este contrato ahora no exige implementar Backup.

### Minor

1. **Una respuesta IPC con payload/envelope inválido se clasifica como fallo de almacenamiento reintentable.** `src/shared/adapters/tauri/client.ts:15` usa parse dentro del mismo try de invoke; `:16` transforma cualquier ZodError en `StorageUnavailable`, y `:9` lo marca retryable. El requestId incorrecto ya recibe `IntegrityFailure`, pero otros defectos de contrato (versión, enum, campos o payload) generan una explicación y política de reintento equivocadas. Separar la excepción del transporte de la validación de la respuesta y devolver IntegrityFailure no reintentable para respuestas recibidas que incumplen el contrato. Añadir caso simulado de envelope o payload inválido; preservar la normalización de failures estructurados y la protección frente a contenido privado.

## Checks y evidencia

- Se siguió la plantilla task-scoped de Superpowers y las instrucciones centrales de revisión, leyendo los hunks asignados por secciones acotadas. No se ejecutaron comandos Git, no se volvió a ejecutar ninguna suite ni se modificaron producto, índice, HEAD, ramas u otros informes.
- Se inspeccionó `work/final-check-1.log`: registra el comando canónico `npm run typecheck` / `tsc -b`, nueve tests en tres archivos, build Vite con worker local y tests Rust. Esta es evidencia existente del implementador, no una ejecución nueva del revisor. package.json no declara ESLint ni script de lint.
- Único focused check: riesgo de desalineación entre límites/nullable de los esquemas wire y contratos. En el worktree se ejecutó un probe Node in-memory que importó `wire.ts` y llamó safeParse de los campos concretos; sin instalación, archivos temporales ni writes. Comando: here-string JavaScript canalizado a `node --input-type=module -`. Exit 0; resultado exacto: `{"unicode1000":true,"unicode1001":false,"backupOverSinglePdfCap":false,"pageNullableTotal":false,"nullableDefinitionVersion":false}`. Resuelve la duda Unicode sin inferir erróneamente semántica UTF-16 de .max en esta versión de Zod, y confirma los rechazos descritos.
- El registry leído contiene 63 entradas y solo settings_get_app_info/settings_get_library_info/settings_get_library_status marcados implemented=true. `src-tauri/tests/scaffold_contracts.rs:36` comprueba lista normativa y permisos Rust; la autoridad Tauri/ACL completa corresponde al revisor general.

## Cannot Verify

- Es una revisión local de tarea; no se suministró PR ni metadata de merge readiness/CI. Los resultados existentes y este dictamen no sustituyen comprobaciones de integración.
- Los nueve tests TS del paquete ejercitan contratos/adaptador con invoke simulado; no montan App ni prueban interacción React/WebView2. La revisión estática permite valorar el guard de desmontaje y las ramas visuales, pero no confirma comportamiento visible en el ejecutable entregado.
- La captura nativa descrita en el informe precede a los últimos ajustes Rust. La compilación nativa posterior no demuestra por sí sola esa interacción, instalación limpia, NSIS, release sin consola ni WebView2 offline. Son límites reconocidos; no se exige repetirlos aquí.
- La fixture de 500 MiB es un valor wire. No demuestra copia/importación de un PDF de ese tamaño ni lector operativo, que pertenecen a las tareas siguientes.
