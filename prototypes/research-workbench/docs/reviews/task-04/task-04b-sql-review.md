# T04b — revisión final SQL-only del corte a051d88

Estado de entrega: **DONE_WITH_CONCERNS** por límites de ejecución. Dictamen: **PASS para DDL y registro de migración en este corte**, sin hallazgos críticos o importantes. No es aprobación de T04b completo ni del merge.

## Corte y alcance

- BASE: `ce15c95dc6404aae1cb158ca6c4a1fe576b70505`.
- HEAD revisado: `a051d88ca4c8127fe4749293b0b03cecc2923a14`.
- `0002_workflow.sql`: blob `0394c1db6056b2ebf2697aa3da9c12ba8442a5f0`.
- `migrations.rs`: blob `68047c068baa1fcdd8567a64bf904c916c9640fa`.
- `0001_library.sql`: blob idéntico en BASE y HEAD, `57e2822dc1e5b2f1acb57930ffbbfd2019d4889d`.

Producto leído sólo desde objetos Git mediante `git show`, `git diff`, `git ls-tree` y `git rev-parse`; no se leyó producto WIP. Alcance: DDL/seed/backfill de 0002, registro en migrations.rs y compatibilidad estática con 0001, runner, unidad transaccional y clasificación de esquema futuro. La lectura puntual de pruebas/definitions.rs sólo identifica verificaciones previstas; no amplía el dictamen al adaptador Workflow.

Fuentes locales: AGENTS, INTENT, STATUS, brief central T04b (migración/ownership), precheck de migración, DATA §§2/3/7/8, TASK04_PORTS §1 y WORKFLOW_GATES (pin, clock, inicialización y lifecycle). Karpathy aplicado a alcance/supuestos/evidencia; using-superpowers contiene SUBAGENT-STOP y excluye al revisor delegado. Sin consulta/escritura OpenViking: peer no verificado; las fuentes locales bastan.

## Resultado de la inspección

1. **Registro y continuidad — PASS.** `src-tauri/src/adapters/sqlite/migrations.rs:9–23` mantiene `Migration { version, sql }`, conserva 0001 y añade únicamente 0002/version2. El diff completo de ese archivo no cambia checksum, ledger, backup, runner o apertura read-only. La igualdad de blobs confirma que 0001 no se reescribió.

2. **DDL y pertenencia — PASS.** `src-tauri/src/adapters/sqlite/migrations/0002_workflow.sql:1–38` crea exclusivamente definiciones PRE/P1/P2, fases y respuestas. PK compuesta de definición permite versiones posteriores; FK `(phase_code,definition_version)` obliga al pin exacto existente. PK `(paper_id,phase_code)` y FK compuesta de respuestas conservan pertenencia al Paper/fase, sin cascadas destructivas. NOT NULL, enums, revisión no negativa de fase/positiva de respuesta, límites de texto, JSON válido y pareja snapshot/hash cubren las restricciones básicas previstas. La PK de fases indexa su FK a Paper; la PK de respuestas indexa su FK a fase. No hay tablas futuras, triggers, commits o pragmas en 0002.

3. **Seed autocontenido — PASS estático.** `0002_workflow.sql:8–11` inserta los tres payloads v1 y sus hashes con INSERT normal, sin reemplazo/ignore ni seed tardío. La lectura contra los tres JSON canónicos del mismo HEAD no muestra discrepancias en contenido, salidas, reglas o definitionHash. No se recalculó SHA-256 ni se ejecutó la comparación automática seed/JSON: corresponde al gate del autor. SQLite sólo valida JSON/longitud; validez semántica e inmutabilidad de definiciones siguen siendo obligaciones de aplicación.

4. **Backfill — PASS estático.** `0002_workflow.sql:39–47` cambia únicamente `processing_initialized/current_phase`. Para todos los Papers piloto válidos `(0,NULL)`, fija processing1/active PRE y crea PRE IN_PROGRESS/revision1, P1/P2 NOT_STARTED/revision0, pin v1, sin respuestas ni completed_at/snapshots. No filtra lifecycle: preserva NEW, ARCHIVED, COMPLETED reservado y archived_from_lifecycle; tampoco escribe UUID/documento/metadatos/revision/updated_at/receipts. Las expresiones del UPDATE leen el contexto anterior. El ELSE2 viola deliberadamente el CHECK de 0001 `processing_initialized IN(0,1)` ante contexto inesperado y aborta; no normaliza ni ignora incoherencia. Una tabla preexistente también produce error por CREATE TABLE, sin IF NOT EXISTS.

5. **Atomicidad/backup — PASS de compatibilidad estática.** `migrations.rs:196–226` verifica ledger antes, calcula pendientes y hace snapshot antes del upgrade de versión positiva (`207–208`). DDL, seed, backfill, ledger/checksum y user_version se ejecutan dentro de una sola `with_transaction` (`210–225`); ésta usa IMMEDIATE y confirma al final (`src-tauri/src/application/unit_of_work.rs:6–9`). quick_check/foreign_key_check anteceden al commit. Una migración fallida se mapea a MigrationFailed y la Transaction se descarta sin commit; el backup previo queda fuera de esa TX. `snapshot` usa API SQLite backup, valida integridad e inventario; 0002 no altera ese procedimiento. Estas conclusiones son lectura de código, no una prueba nueva de rollback/reapertura o resistencia a fallos físicos.

6. **Esquema futuro y fallo — PASS de compatibilidad estática.** `src-tauri/src/adapters/sqlite/actor.rs:55–96` compara con SCHEMA_VERSION; una versión >2 conserva el recorrido diagnóstico/probe y query_only, sin llamar a migrate ni establecer WAL/FULL sobre esa raíz. Para esquema soportado activa foreign_keys y mantiene la adquisición previa del lock. MigrationFailed/IntegrityFailure/BackupFailed deshabilitan escritura. Subir el límite a2 cambia la versión soportada, sin quitar la guarda. No se ejecutó schema_diagnostic ni se validó inventario/filesystem read-only en esta revisión.

## Observaciones menores, no bloqueantes

- `0002_workflow.sql:38`: `phase_answers_paper_phase` duplica el prefijo `(paper_id,phase_code)` de la PK de respuestas. Añade mantenimiento/espacio, sin defecto de integridad. No requiere alterar este corte para la aceptación.
- `0002_workflow.sql:22`: la FK a definición no tiene índice hijo que comience por `(phase_code,definition_version)`. Un intento de borrar/cambiar la clave de una definición podría recorrer paper_phases para comprobar referencias. Las definiciones son inmutables y no hay tal operación autorizada en v0.1; no se demuestra un problema de rendimiento de producto. Conviene indexar esa referencia si se añade un acceso por definición o una operación administrativa pertinente. No se exige un cambio especulativo de migración.
- Los CHECK no prueban por sí solos hash real, reglas/keys de definición, estados/contexto coherentes, tipos estructurados por pregunta o límite seguro de contadores JSON. Tampoco todos los TEXT temporales son fechas válidas por DDL. El contrato deposita esas invariantes semánticas en dominio/aplicación/decoder; esta revisión no afirma que los adaptadores ya las cumplan.

## Hallazgos anteriores y límites

Los dos importantes del precheck —clock de inputs directos e idempotencia/coherencia de initialize_processing— pertenecen a `application/workflow.rs`. **No se consideran cerrados por este PASS SQL** y requieren dictamen SPEC/Rust final sobre el corte corregido. Tampoco se revisan casos de uso, WorkflowRepository en extracción, proof/guardas, IPC, replay, import/recovery o invalidación de aplicación.

No se ejecutó SQL, test, cargo, build, script de proyecto, GUI o instalador. No hubo cambios de producto/normativa/configuración global, commits, ramas, merges o subagentes. No se abrió ninguna biblioteca personal. Se leyeron tests de migración/diagnóstico para reconocer sus objetivos, sin atribuirles exit0 ni cierre de cobertura. El autor/orquestador debe aportar upgrade0001→0002 poblado, checksum rechazado, constraints, fallo tardío con rollback integral y backup reabrible, seed/hash, reapertura e integridad FK, además del gate pertinente. No se mide rendimiento ni se extrapola esta aprobación a instalación nativa.

## Registro y autorrevisión

Comandos de inspección: `git diff BASE HEAD -- <0001/0002/migrations.rs>`, `git diff --raw BASE HEAD -- <esas rutas>`, `git show HEAD:<ruta>`, `git ls-tree -r --name-only HEAD src-tauri`, `git rev-parse HEAD:<ruta> BASE:<0001>`; lecturas centrales con Get-Content/rg. Las inspecciones pertinentes terminaron exit0. Dos búsquedas iniciales usaron rutas inexistentes (`src-tauri` en árbol central y nombre db_actor.rs) y una lectura probó un nombre incorrecto `0001_initial.sql`; se corrigieron con inventario Git hacia actor.rs/0001_library.sql. Esos errores de localización no son fallos de producto ni pruebas ejecutadas. Salidas truncadas se completaron con lecturas específicas.

Única escritura: este informe central, conservando cambios ajenos. La consulta auxiliar de memoria sólo recordó la pauta histórica de leer fuentes locales; no aportó evidencia de producto ni resultados de pruebas.

**Validez del dictamen:** el orquestador comprobará que 0002 y migrations.rs conservan estos blobs en el HEAD final. Si cambian, revisar el delta antes de integrar. Aunque permanezcan iguales, el PASS sigue limitado a DDL/registro; el merge espera revisiones finales de aplicación y gates verificados.
