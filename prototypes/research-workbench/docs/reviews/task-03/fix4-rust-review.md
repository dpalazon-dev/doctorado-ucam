# T03 fix4 — revisión Rust de timestamps UTC

Estado: **DONE**. Veredicto: **PASS**. Sin hallazgos Critical/Important/Minor en el delta Rust revisado.

- BASE: `5a512a7bba610e90ecb10d657ed1b36095df4078`.
- HEAD final: `cb66da1124e553ddd5958f1d8619dc7a3686324d`, árbol limpio.
- HEAD de producto: `55407d12d6b7ca403b2ab2c10390e524f43ffdbd`.
- Worktree: `.worktrees/task-03-reader`.
- Revisor: `/root/task03_rust_review`.

## Alcance y resultado

Leídos los briefs fix4/revisión, informe del autor, ADR-019 y CONTRACTS del repositorio principal, diagnóstico `docs/reviews/task-03/native-open-diagnosis.md` y diff completo de los dos archivos Rust modificados. No se reaudita T03 previo ni se reabren sus hallazgos cerrados.

Los únicos cambios Rust de producto están en `src-tauri/src/adapters/sqlite/reader_repository.rs:166` y `:219`: apertura y guardado generan `Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)`. Eso fija los dos escritores a UTC con tres decimales y `Z`, conforme a ADR-019 y al formato existente de Library/receipts.

El cambio permanece dentro de las acciones transaccionales ya existentes. No modifica payload/hash del request, identidad, CAS, incremento de revisión, referencia documental, actor, captura del handle o permisos. No cambia los lectores ni introduce migración o normalización de resultados históricos. En replay, `with_receipt` sigue devolviendo el resultado durable sin ejecutar el escritor.

## Pruebas revisadas

- Apertura real de la fixture: exige `lastOpenedAt` terminado en Z, conserva revisión/updatedAt bibliográficos y comprueba igualdad entre DTO, columna DB y timestamp del receipt. El replay completo vuelve a verificar acceso y devuelve el mismo resultado.
- Guardado real: exige `updatedAt` terminado en Z, getter igual al resultado guardado y replay idéntico. Conserva la revisión y posición esperadas.
- `legacy_open_utc_timestamp_replay_preserves_the_receipt_and_database_value`: parte de un resultado real, siembra una representación histórica de nueve decimales con `+00:00` y verifica getter/replay, receipt byte a byte, columna intacta y auditoría sin incremento.
- `legacy_position_utc_timestamp_get_and_receipt_replay_preserve_stored_strings`: verifica getter/replay con la misma representación histórica, columna intacta y receipt byte a byte. Este caso no cuenta auditoría por separado; esa afirmación está probada directamente por el caso de apertura y respaldada por la ruta de replay compartida sin cambios.

Los asserts nuevos de formato comprueban Z; la precisión exacta de milisegundos se verifica además por la llamada explícita `SecondsFormat::Millis` de ambos escritores. Las fixtures históricas se siembran únicamente en DB temporales de test; no son una migración del producto ni una reparación de la biblioteca de QA.

## Comprobaciones propias

Después de cargar `. .\scripts\development-env.ps1`:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml --test reader_integration
```

Todos terminaron con exit 0. Reader: 19 entradas pasaron, de las cuales una es el helper de proceso hijo. `git diff --check BASE..HEAD` pasó y el árbol permaneció limpio.

El gate completo con 69 pruebas UI y el build nativo exit 0 son evidencia del autor verificada por el orquestador; esta revisión no repitió esos comandos. Se preservaron producto, configuración y ejecutable de QA; no se abrió la app, biblioteca personal ni instalador.

## Límites de aprobación

PASS para los dos escritores y compatibilidad Rust de lectura/replay. La validación completa del decoder TypeScript pertenece a su revisión especializada. Estos tests no acreditan renderer, protocolo observado en WebView2 ni instalación. La integración sigue condicionada a las demás revisiones aprobadas y a comprobar el merge preparado sin conflictos con su gate pertinente; no se hizo merge.
