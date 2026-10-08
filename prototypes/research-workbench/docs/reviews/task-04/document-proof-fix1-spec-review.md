# T04a corrección1 — revisión focal SPEC/calidad

Fecha: 2026-10-03. Revisor: `/root/task04a_spec_review`.

Dictamen: **PASS — DONE**. El hallazgo HIGH de la revisión anterior queda cerrado; no hay nuevos hallazgos críticos o importantes.

## Corte y alcance

- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04a-document-proof`.
- Corte anterior: `20c7da9f9c9cb1757bd9755aceb00906d6f56239`.
- HEAD revisado: `9ad0bc269909e8013f3add0dc9ac389290fb0b81`, verificado con árbol limpio.
- BASE original de T04a: `9188ac5dc4f1cc5ad22347ef1422a2c51d795b1b`.
- Leídos: delta completo de corrección1, brief central fix1, informe versionado y copia central. Se reutiliza la revisión completa del corte anterior para el código que no cambió.
- Sólo lectura de producto; única escritura: este informe. Sin compilaciones, pruebas nuevas, cambios de producto, merge ni subagentes.

## Cierre del HIGH anterior

`LocalDocumentStore` incorpora un campo privado `cfg(test)` por instancia. Sus clones comparten la inyección de esa instancia, sin estado global entre fixtures. El fallo se consume con `take()` dentro de la closure real de `spawn_blocking`, después de validar RegisteredDocument y justo antes de la llamada de apertura (`store.rs:211`). El guard del mutex se libera antes de ejecutar la inyección; el panic no lo envenena. La compilación normal mantiene la llamada directa a `managed_file_open`, las guardas y la traducción originales.

La nueva prueba `public_proof_and_reader_ports_preserve_injected_open_failures` (`store.rs:793`) invoca realmente ambos puertos y verifica:

| Caso | `prove_access` | `open_verified` |
|---|---|---|
| AccessDenied inyectado | `Ok(Unavailable(AccessDenied))` | `Err(StorageUnavailable)` |
| Fatal(StorageUnavailable) | `Err(StorageUnavailable)` | `Err(StorageUnavailable)` |
| Fatal(NotFound) | `Err(NotFound)` | `Err(NotFound)` |
| Panic dentro del trabajo real | `Err(StorageUnavailable)` | `Err(StorageUnavailable)` |

El control positivo final abre el archivo sintético por `prove_access` sin inyección y recibe `Available`. Esto verifica consumo de la inyección y continuidad del recorrido real después de los panic. Las pruebas previas de Missing real, sharing NTFS, tamaño y retención siguen intactas. La cobertura nueva satisface la parte pendiente de `TASK04_PORTS:162` y los puntos 1–2 del brief fix1.

## Semántica y calidad

El delta no cambia las firmas ni la semántica de producción. La inyección se limita a los tests y no añade un framework, rutas, permisos, dependencias ni estado global. El rustdoc explica propiedad del handle/pins, límites de Unavailable, errores fatales y retención por el consumidor. Los helpers existentes se renombran para que su nivel de evidencia quede claro; el informe separa esas comprobaciones de los recorridos públicos y no atribuye una prueba ACL real a la inyección.

La copia central del informe coincide con la versionada: SHA256 `CAD343B381109B303AA1424275D2679E064946BD2C9E090FCE5D2D85BA0DE22D`. La eliminación de la línea vacía extra al EOF está incluida en el delta.

## Evidencia comprobada

Sin repetir suites, se leyeron los logs existentes `work/task-04a/fix1-check.log` y `fix1-build-debug.log`, y sus respectivos `.exit`: ambos contienen `0`. El log de check incluye la nueva prueba pública con resultado `ok`, 69 pruebas UI aprobadas y 138 entradas Rust aprobadas. Esto verifica evidencia archivada de la ejecución del autor, no una ejecución independiente de esta revisión.

Se ejecutó `git diff --check 9188ac5dc4f1cc5ad22347ef1422a2c51d795b1b..9ad0bc269909e8013f3add0dc9ac389290fb0b81`: sin diagnósticos. HEAD y estado limpio se comprobaron al comenzar la revisión. No se afirma instalación, ACL real, ejecución no Windows ni Workflow PRE/P1. El gate de integración preparado corresponde al orquestador.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: **APPROVE**. Gate SPEC/calidad T04a: **PASS** para el HEAD revisado. El HIGH histórico de `task-04a-spec-review.md` queda cerrado por esta corrección.
