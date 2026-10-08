# Instalación del piloto 0.0.1

Esta guía corresponde al instalador local NSIS por usuario de Windows x64. El piloto no tiene actualizador integrado ni firma Authenticode. La disponibilidad de instalación sin conexión y el contenido WebView2 solo quedan confirmados cuando se inspecciona y prueba el instalador congelado.

## Instalar

1. Descarga o copia `Research-Workbench_0.0.1_x64-setup.exe` junto con su `manifest.json`.
2. Comprueba que el SHA-256 del instalador coincide con `artifact.sha256` en el manifiesto.
3. Ejecuta el instalador y sigue el asistente de Windows. La configuración del producto instala para el usuario actual y crea la entrada del menú Inicio `Research Workbench`.
4. Inicia la aplicación desde Inicio. No hace falta Node.js, Rust, Cargo, Git ni un checkout del código fuente para el uso instalado.

El payload de WebView2 se configura para instalarse desde el paquete. Hasta completar la prueba limpia sin red, mantén disponible el instalador y trata esa capacidad como pendiente.

## Datos locales

La biblioteca release se guarda en `%LOCALAPPDATA%\ResearchWorkbench\library`. Las copias de seguridad y logs quedan en las carpetas hermanas `backups` y `logs`. Conserva esas carpetas al actualizar o desinstalar. El override `RESEARCH_WORKBENCH_TEST_ROOT` solo afecta a builds de depuración y no redirige los datos de una versión release.

## Actualizar y desinstalar

Antes de instalar una versión posterior, cierra la aplicación y conserva una copia de la biblioteca y de `backups`. El piloto 0.0.1 no acredita migración desde una versión anterior. Instala una versión posterior compatible solo cuando sus notas indiquen la versión de origen y su prueba de migración.

Para desinstalar, usa **Configuración de Windows → Aplicaciones → Aplicaciones instaladas** y selecciona Research Workbench. La política esperada es retirar binarios y accesos conservando biblioteca, copias de seguridad y logs. La conservación debe confirmarse mediante la prueba instalada antes de confiar en ella.

## Verificación

El estado observado del instalador y de las pruebas está en [`docs/verification/installed-pilot.md`](verification/installed-pilot.md). No se debe interpretar la generación de un setup como verificación de instalación limpia, funcionamiento offline, actualización ni conservación de datos.
