# T10-piloto — revisión focal de corrección 1

Fecha: 2026-10-03. Revisor: `/root/task10_pilot_review`.

**Resultado: PASS para preparación/build del candidato.** Los dos HIGH de la revisión original quedan cerrados. Una observación menor sobre conservar una regresión no bloquea este corte.

## Corte

- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-10-pilot`.
- BASE de tarea: `f2d39a25feb81141b51ca6cf028777b38503818b`.
- BASE de corrección: `fe6af93a23bf1fb54f03e4c24df4cada675ad090`.
- HEAD revisado: `510edb1aba4234e74812e668f0c9ee736a9c68b7`.
- Árbol limpio antes/después; sin diferencias staged/unstaged.
- Revisados el delta completo de cinco archivos, el script y sus pruebas completos. Alcance limitado a host, arquitectura app/stub, manifiesto y documentación afectados. Sin reabrir T03.

## Cierre de hallazgos importantes

1. **HIGH host CRLF — CERRADO.** `Assert-RustcHost` analiza exactamente una línea de host, admite CRLF/LF y compara el target normalizado. `Invoke-ReleaseBuild` usa este helper. La suite ejecutada cubre LF, CRLF, host incorrecto y la salida real del `rustc` instalado usando `Get-ToolOutput`; exit 0.
2. **HIGH arquitectura setup/payload — CERRADO.** Se valida PE `0x8664` en `research-workbench.exe` del target aislado; la ausencia del archivo falla explícitamente. El setup usa un validador separado que admite i386/x64 y registra la máquina observada. El manifiesto separa hash/bytes/máquina del setup y de la aplicación, y los resultados distinguen arquitectura de aplicación y stub. Checklist, informe y verificación instalada se corrigieron. La suite confirma aplicación x64 y stub i386, rechaza un stub ARM64 y comprueba el texto de arquitectura registrado.

## Observación no bloqueante

### [LOW] Conservar el caso de rechazo de aplicación no x64

**Archivo:** `scripts/tests/build-release.tests.ps1:51`.

La corrección eliminó el `Assert-Throws { Assert-PeX64 ... }` anterior al convertir el fixture en stub i386. La función conserva correctamente el rechazo, pero esa regresión ya no está en la suite versionada. Se recomienda restaurar la aserción sobre el fixture i386, además de aceptarlo con `Assert-NsisSetupMachine`.

La revisión ejecutó un probe sintético adicional sobre las funciones vigentes: aplicación i386 rechazada con el diagnóstico esperado; el mismo PE aceptado como stub i386; aplicación x64 aceptada. Los tres casos pasaron, por lo que no queda un fallo de comportamiento abierto.

## Evidencia ejecutada

```powershell
git status --short
git diff --staged
git diff
git diff fe6af93a23bf1fb54f03e4c24df4cada675ad090 510edb1aba4234e74812e668f0c9ee736a9c68b7
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/tests/build-release.tests.ps1
git diff --check fe6af93a23bf1fb54f03e4c24df4cada675ad090 510edb1aba4234e74812e668f0c9ee736a9c68b7
git status --porcelain=v1
git rev-parse HEAD
```

Resultados: suite focal exit 0 (`build-release behavior checks passed`); diff check exit 0; HEAD exacto confirmado y status vacío. El probe PE adicional cargó por dot-source el script, escribió exclusivamente un PE sintético de 256 bytes bajo `%TEMP%` con nombre GUID, comprobó los tres casos descritos y eliminó únicamente ese archivo en `finally`; exit 0.

No se ejecutaron build, instalador ni aplicación release. No se cambiaron código de producto, configuración global o bibliotecas. La inspección del `.nsi` generado, el payload empaquetado y todos los gates instalados siguen pendientes y pertenecen al trabajo posterior del orquestador. Esta revisión autoriza técnicamente continuar con el candidato; no acredita instalación ni funcionamiento offline.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 1 | note |

Verdict: **PASS** — ambos HIGH cerrados y ninguna regresión funcional detectada en el alcance focal.
