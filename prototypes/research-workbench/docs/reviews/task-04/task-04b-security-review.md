# T04b — revisión independiente de seguridad e integridad

**Estado: BLOCKED para integración del corte.** Revisión estática completada el 2026-10-03. **Cero hallazgos nuevos** de seguridad/integridad; los hallazgos abiertos S3–S9 del informe SPEC conservan su autoridad y no se duplican aquí. El resultado favorable de los focos siguientes no autoriza integrar mientras esos bloqueos y la validación independiente sigan pendientes.

## Corte y método

- Repo principal: `C:/Users/david/Projects/Research-Workbench`.
- Autor: `.worktrees/task-04b-workflow-backend`.
- BASE: `ce15c95dc6404aae1cb158ca6c4a1fe576b70505`.
- HEAD de producto: `a051d88ca4c8127fe4749293b0b03cecc2923a14`.
- HEAD de informe: `7aa120e46ad180d85c638f39bbcbc9355bab79ca`; su delta respecto al producto sólo contiene `docs/reports/task-04b-report.md`.
- Producto inspeccionado exclusivamente mediante `git show` y `git diff` de estos cortes; no se leyó WIP posterior ni se modificó producto.
- Fuentes locales actuales: AGENTS, INTENT, STATUS, briefs centrales T04b/review, informe SPEC y reporte del autor; TASK04_PORTS, WORKFLOW_GATES, CONTRACTS y ADR-020. Skill utilizada: `security-review`, adaptada al producto local y al alcance exclusivamente estático del despacho.
- Se revisaron la frontera del dispatcher/DTO, servicio, puertos, composición/lifecycle, receipts/UoW, repositorio y migración SQL, dominio, integración Library e invalidación. Se leyeron las pruebas relevantes de IPC, concurrency, migration, Library y el delta del test TypeScript.
- No se ejecutaron tests, builds, auditoría de dependencias, GUI, instalador, nuevas herramientas/scripts ni operaciones sobre bibliotecas personales. No se crearon subagentes, commits o merges. No se consultó ni escribió OpenViking: peer no verificado; las fuentes locales gobiernan.

## Hallazgos

No se añade una vulnerabilidad nueva: **CRITICAL 0 / HIGH 0 / MEDIUM 0 / LOW 0**. Por ello no hay filas nuevas de severidad/path/línea/escenario. S3–S9 se remiten a `task-04b-spec-review.md`, sin recodificar sus defectos de pin, precedencia, historial, evidencia, límites o arquitectura como hallazgos adicionales.

S1/S2 del corte SPEC original sí tienen cambios visibles en este HEAD: `workflow_repository.rs:766` inicia/reactiva P1 y mueve contexto en la TX de PRE, preservando datos de una P1 previa; `domain/workflow.rs:55` exige concordancia con Paper.reviewType también para UNKNOWN. Las regresiones añadidas consultan la activación durable y conservan los datos de P1. Esto es comprobación estática del delta, no ejecución independiente ni cierre de toda la aceptación SPEC.

## Comprobaciones estáticas favorables y límites

| Foco | Evidencia del corte | Conclusión y límite |
|---|---|---|
| IPC y entrada | `transport/commands/mod.rs:74` y ramas Workflow; Args existentes de `transport/dto.rs`; `domain/workflow.rs:228`; `workflow_repository.rs:935` | Registry cerrado y ventana main; UUID canónico, campos desconocidos rechazados, enums cerrados. PRE.review_type y P1.relevance_decision validan estructura cerrada; las salidas textuales no aceptan structuredValue. Respuestas/explanation se normalizan y acotan sin truncar. El límite de expectativas goBack/touch ya está en S8. |
| SQL e información privada | `workflow_repository.rs`, `receipts.rs`, `transport/error.rs` | Consultas/mutaciones de inputs parametrizadas; nombres de columnas/tablas y ramas de fase son constantes. No se añadió shell, fetch/red, SQL genérico, filesystem libre para UI, ni credenciales detectables en el diff. Los errores nuevos conservan códigos y mensajes seguros; el snapshot documental contiene id/status/hash/available, sin ruta ni bytes PDF. No es una auditoría completa de historia/secrets/dependencias del repo. |
| Propiedad del trabajo | `modules/workflow/service.rs:81`, `:113`, `:139`, `:188`, `:240`; `workflow_repository.rs:298`, `:394`, `:447` | Mutaciones y evaluación usan spawn/oneshot: cancelar IPC abandona el receptor, mientras el trabajo admitido conserva permisos. Advance/touch/evaluate trasladan permisos y proof a la closure final, fuera de action y alrededor de toda TX/with_receipt. Save/goBack conservan permisos en el trabajo propietario mientras esperan al actor. No se identificó una liberación anticipada en el recorrido normal o de error leído. |
| Replay y colisión | `receipts.rs:8`, `workflow_repository.rs:251`, `:371`, `:410` | Lookup compartido verifica requestId/comando/hash y deserializa errores corruptos como IntegrityFailure. Prepare advance/touch resuelve replay antes del proof; with_receipt repite lookup bajo IMMEDIATE. Las excepciones de capability conocidas pertenecen a S4. No se añade receipt exitoso ante un error de action. |
| Proof y TOCTOU | `workflow_repository.rs:500`, `:324`, `:415`, `:460`; `modules/workflow/service.rs` | Referencia desde SQLite sin filtro ACTIVE-only, asociación Paper/Document y ruta administrada comprobadas. Apertura fuera del actor; Available debe concordar con RegisteredDocument. La TX/lectura final vuelve a comparar Paper, activeDocumentId y RegisteredDocument completo, tanto para Available como para Unavailable. Una asociación rota falla; un cambio válido devuelve Conflict. Err(AppError) se propaga mediante `?`, sin convertirlo en available=false. No se llama read_all ni se hashea el PDF bajo el actor. |
| Atomicidad y clocks | `application/unit_of_work.rs:2`, `workflow_repository.rs:673`, `:935`; `application/workflow.rs:73` | Estado/snapshot/contexto/clocks y receipt se escriben en una sola TX IMMEDIATE. Archive P1 compone el helper Library con la TX prestada, sin receipt ni actor anidados. Cambio de metadata título/tipo e import llaman al helper Workflow en su TX. D1 une/deduplica inputs, distingue dependientes y preserva snapshots/completedAt. Overflow de clock/answer revision retorna error dentro de la TX. La suficiencia del historial ya está en S5. |
| Lifecycle, recovery y esquema futuro | `modules/workflow/service.rs:295`, `domain/workflow.rs:157`, `workflow_repository.rs:970`, `adapters/sqlite/actor.rs` | Nuevas mutaciones verifican recovery y writable antes de admitir; guardas durables rechazan ARCHIVED/COMPLETED/TRASHED para nuevas transiciones. Lecturas de fases/respuestas/definiciones no restauran ni aceptan. Replay no repite archive. Se reutiliza el actor existente, cuyo modo futuro es query_only; Workflow no cambia su mecanismo. No se ejecutó schema_diagnostic ni se comprobó una biblioteca futura en runtime. |
| Migración/composición | `0002_workflow.sql`, delta `migrations.rs`, `lib.rs:67` | 0001 no cambia; 0002 es SQL-only dentro del runner existente, con PK/FK/CHECK, seed y backfill. Un contexto previo incoherente fuerza fallo del CHECK en vez de resetearlo. Import/recovery usan la misma confirmación. Workflow comparte actor, store, maintenance, recovery y RequestRegistry con Library/Reader; no introduce conexión/registry/harness nuevos. La amplitud de la prueba de upgrade/reopen ya está en S7. |

Las líneas corresponden a HEAD de producto. Las referencias a archivos existentes fuera del delta sólo se usaron para interpretar el mecanismo reutilizado; no se expandió la revisión a una auditoría general del piloto.

## Pruebas leídas frente a pruebas ejecutadas

**Ejecutadas por este revisor: ninguna.** Los comandos de inspección fueron `git diff --stat BASE HEAD`, `git diff --name-only PRODUCT REPORT`, `git diff BASE HEAD -- <rutas>` y `git show HEAD:<ruta>`, con los SHA anteriores sustituidos en cada llamada. Las búsquedas/lecturas locales de instrucciones y normas fueron sólo lectura.

Pruebas leídas especialmente relevantes:

- `workflow_concurrency::cancelling_during_proof_keeps_shared_registry_and_operation_owner`: abort/join efectivo del caller; primeras polls de Library/Reader contra registry compartido antes de liberar proof; mantenimiento Busy y resultados Conflict posteriores.
- `workflow_concurrency::cancellation_keeps_request_and_maintenance_until_reference_conflict_rolls_back`: observa enqueue de advance con actor bloqueado, cancela realmente caller y verifica rollback. El Drop del handle lee estado durable y adquiere un escritor IMMEDIATE tras la TX; no se limita a contar drops después de idle.
- `workflow_concurrency::proof_handle_drop_observes_committed_advance`: Drop consulta lifecycle/fase/receipt confirmados y obtiene un escritor. Esto es una aserción adecuada de orden de destrucción, sin atribuir aquí que haya pasado.
- `workflow_concurrency::unavailable_proof_with_changed_reference_returns_conflict_without_transition` y `typed_proof_errors_propagate_without_becoming_gate_results`: comprueban referencia cambiada con indisponibilidad y errores fatales tipados, respectivamente. La matriz de cobertura restante es parte de S6; no se interpreta el nombre de un test como prueba más amplia que sus aserciones.
- `workflow_ipc::p1_archive_failure_rolls_back_phase_lifecycle_audit_and_receipt` y el rollback de inicialización Library inyectan fallo tardío en la operación transaccional. Se leyeron sus aserciones; no se ejecutaron.
- Las pruebas de migración leídas comparan un subconjunto del estado y comprueban backup/reopen; no acreditan por sí solas toda la aceptación ni un snapshot aceptado reabierto (S7).

El reporte en `7aa120e` declara gate completo y build debug exit0 observados por el autor. Reconoce expresamente que `*.summary.log` y `*.exit` son resúmenes/sidecars **transcritos manualmente**, sin raw stdout íntegro ni generación automática por proceso. Se toman como declaración del autor, nunca como logs independientes revisados. Este informe no añade PASS ejecutable, nativo, instalado o de equipo limpio.

## Entrega

Revisión estática terminada; nuevo reporte central escrito y ningún archivo de producto modificado. **BLOCKED para integración** por los hallazgos SPEC ya abiertos y las comprobaciones independientes pendientes. El foco de seguridad adicional no encontró nuevos bloqueos que deban añadirse al brief correctivo consolidado.
