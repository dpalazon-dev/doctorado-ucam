# Verificación instalada — piloto 0.0.1

Fecha de preparación: 2026-10-03. Este documento distingue preparación del paquete de QA sobre una aplicación instalada.

## Artefacto y entorno

| Campo | Estado |
| --- | --- |
| Commit y SHA-256 del setup | Pendiente: Sol ejecutará el build tras revisión sobre el commit limpio que contiene el script. |
| Windows 11 x64 limpio / VM disponible | Pendiente; la disponibilidad de VM/Sandbox no está demostrada. |
| WebView2 previamente ausente | Pendiente. |
| Red desconectada | Pendiente. |
| Node/npm/Rust/Cargo/Git/Codex ausentes | Pendiente. |
| Checkout de fuente ausente y cwd de ejecución registrado | Pendiente. |
| Biblioteca sintética aislada | Pendiente; no se ha lanzado ni instalado el ejecutable release en el perfil de trabajo. |

## Resultados por separado

| Gate | Estado | Evidencia actual |
| --- | --- | --- |
| Comprobaciones de comportamiento del script | Comprobado: exit 0 el 2026-10-03. | `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/tests/build-release.tests.ps1`; salida `build-release behavior checks passed`. Cubre LF/CRLF, host incorrecto, probe del `rustc` instalado, app PE x64 y setup NSIS con stub i386. |
| Bundle NSIS generado | Pendiente; reservado al build de Sol después de la revisión. | No se ha ejecutado el comando de release. |
| Arquitectura del ejecutable de la aplicación | Pendiente hasta inspeccionar `research-workbench.exe` producido por Tauri. | El script exige PE `x64 (0x8664)` y registra su SHA-256/bytes por separado. |
| Máquina PE del setup NSIS | Pendiente hasta inspeccionar el artefacto real. | El script acepta y registra la máquina real del stub compatible (`i386` o `x64`); no la usa como prueba de arquitectura de la aplicación. |
| Payload WebView2 offline | Pendiente de inspección del NSIS generado y prueba limpia desconectada. | `offlineInstaller` es configuración, no evidencia del payload ni de instalación offline. |
| Recursos PDF.js dentro del producto | Pendiente de inspección/lectura instalada. | T03 configura worker, cmaps, fuentes estándar, ICC y WASM locales en el frontend. |
| Inicio desde menú / ejecución sin source ni servidor | Pendiente. | No se ha ejecutado la aplicación release. |
| Importar, mover original, página 2, cerrar/reabrir, archive/restore | Pendiente de QA instalada. | No se ha usado instalador ni datos personales. |
| Segunda instancia / escritor único | Pendiente de QA instalada. | No se ha comprobado el aviso de Busy en release. |
| Upgrade con migración real | Pendiente; no hay versión anterior compatible demostrada para este piloto. | Reinstalar 0.0.1 no contará como migración. |
| Desinstalar y reinstalar conservando biblioteca/backups | Pendiente de QA instalada. | No se ha ejecutado el instalador ni desinstalador. |

## Comandos preparados

El script, una vez incluido en un commit limpio, fija la plataforma explícitamente y guarda el candidato bajo `dist-release/0.0.1/`:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/build-release.ps1
```

Prueba focal del comportamiento del script:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/tests/build-release.tests.ps1
```

Este informe no declara aceptado el gate de instalación piloto. Solo la evidencia del setup exacto en un entorno limpio/offline puede cerrarlo.
