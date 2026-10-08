# T04c — primer intento nativo del runner QA3

2026-10-05. Orquestador root. Resultado **FAIL**, no aceptación nativa de T04c.

Runner QA3 `31631fd3641bc52a6403144a0a05558262cbc6a1`, SHA256 `1B95DC08050BAEE82FD134939EC37A10952693F85437CF0CA49D9E05A9501089`, revisado estáticamente por root y revisor independiente. Candidato debug `32249B7A168D8B8AA53E65D82BE11A55FA6EE0B2294DE94682A424CC0921BDEC`; versión WebView2/EdgeDriver154.0.4258.53 releída antes de lanzar. Los cinco hashes fijados, tres fixtures y AST fueron comprobados de nuevo a las04:00:49UTC.

Comando desde primary, con PowerShell7.6.5 incluido en Codex:

```powershell
& 'C:/Users/david/.cache/codex-runtimes/codex-primary-runtime/dependencies/native/powershell/pwsh.exe' -NoProfile -ExecutionPolicy Bypass -File '.worktrees/task-04c-native-runner/work/qa/t04-native/native-qa.ps1' -ExecuteAfterReview -ApprovedRunnerSha256 '1B95DC08050BAEE82FD134939EC37A10952693F85437CF0CA49D9E05A9501089'
```

Stdout/stderr y exit inmediato conservados en `work/task-04c-integration/native-qa3.*`; proceso terminó con **exit1**. Run propio: `.worktrees/task-04c-native-runner/work/qa/t04-native/native-run-b4f1fe3c9f0f4fe8b4c83230f75bd526/`. Hash de native-result.json: `4D87E5255F514398066CCE1A469CA22485915803165EE08A100BE5350A4B20E3`.

Se inició la aplicación real con perfil y biblioteca sintéticos; el ciclo1 llegó a abrir Reader tras las importaciones. La aserción de línea843 falló: `Reader page 1 raster does not contain the synthetic fixture rule`. No llegaron a ejecutarse PRE/P1 ni reinicios. No hay capturas ni métricas de raster guardadas antes de esta excepción, por lo que el resultado no permite atribuir el fallo al producto.

El registro muestra click de Reader04:01:31.0057UTC y consultas04:01:31.0274/0342UTC. La condición de espera lee canal rojo sin alpha; puede aceptar un canvas transparente antes de renderizar. Hipótesis enviada a investigación focal QA4, sin modificar producto ni reducir requisitos. Una prueba sintética puede demostrar el defecto del predicado, pero no reconstruir los píxeles históricos ausentes.

Cleanup registró DELETE session, app propia ausente, driver/EdgeDriver/WebView propios ausentes, puertos libres y cero errores. Identidad de perfil correlacionada. No se observó cierre normal: CloseMainWindow=false, WaitForExit=false, ExitCode=null; la limpieza tras fallo no se presenta como aceptación de cierre.

El merge de producto sigue preparado sin commit, árbol `a85fc499718fe3f75c629ac4b04da3d901233989`; los gates previos de producto no cambian. Se conserva evidencia del fallo y se asigna QA4 a un autor Sol nuevo conforme a la cuarta ronda del procedimiento. Instalador/equipo limpio siguen sin probar.
