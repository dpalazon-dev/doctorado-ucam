# T03 — evidencia nativa al 3 de octubre

La integración de código T03 está aprobada; este checkpoint no declara validado el recorrido PDF nativo ni una aplicación instalada.

## Selector observado

El diagnóstico `picker-diagnostic-2d07b20403aa43888475c5140eafef18` observó el diálogo propio, Edit/1148 y Button/1. UI Automation los exponía sin los patrones requeridos por el runner anterior. Se cerró el diálogo y la aplicación terminó con código 0 antes de eliminar la sesión; limpieza completa. No seleccionó un archivo.

Su resultado está en `work/qa/t03-native/picker-diagnostic-2d07b20403aa43888475c5140eafef18/picker-diagnostic-result.json`, SHA256 `597F480DBA0DBBF0334995CB393E36E89BB00A38FDEDF2F456554DFB2BE8B941`. El resultado conserva el hash de script ejecutado `50AD0ADB3AA9E4718562283E4D92B1A3D051D8D29023E532038CFFA199416B82`, pero la fuente exacta se sobrescribió durante la preparación posterior y no se recuperó. No se presenta la copia posterior 7FF como el script ejecutado. Los resultados originales y el informe de revisión permanecen conservados.

## Intento sobre T03 integrado

Se revisó el selector Win32 del runner 47379FFF y se cerró su único ajuste preventivo: BM_CLICK usa flags 0x02, manteniendo timeout y comprobaciones posteriores. El runner ejecutado tiene SHA256 `39C2102EE00A9ED4F8E2C1A93221AE5A73ABACBE236A4D3B1D85A6D52D772616`; el candidato integrado tiene SHA256 `69FD578D817849DFB60F2B0D15A82DE0A66686EF6D1F9D79F946D5978931D938`.

El intento `native-run-79e56c22f2c444c78e53bb800b35a056` terminó con exit 1 en el selector. Encontró controles únicos del proceso propio; WM_SETTEXT devolvió éxito y BM_CLICK completó el transporte. Después contó dos diálogos Open/Abrir y abortó por ambigüedad. No registró la identidad de ambos ni la visibilidad del original, por lo que no puede determinarse aún si fue una ventana de validación o una condición del runner.

Resultado: `work/qa/t03-native/native-run-79e56c22f2c444c78e53bb800b35a056/native-result.json`, SHA256 `20A1E8DA5FB8C410297A189AC4355F51A45171C647DE34F5FE5D0875165BAB37`. Limpieza completa, sin procesos propios ni listeners en los puertos 64777/64778. No se acreditan importación, protocolo PDF, render, posición persistida ni cierre normal anterior al cleanup en este intento.

El próximo diagnóstico conservará el snapshot del runner y añadirá inventario acotado de las ventanas del proceso propio. No se modifica el producto a partir de una causa no demostrada. La prueba usa únicamente PDF y biblioteca sintéticos.

## Diagnóstico y avance posterior

El runner 189CFC5A observó dos HWND propios: el original visible y una ventana hija oculta. Leer el campo devolvió exactamente la ruta sintética (94 caracteres). El inventario explica por qué el conteo global anterior era ambiguo; no acreditó importación por sí mismo. Resultado `native-run-c1c2f06292964250b2c76a8808008880`, SHA256 `8BF9738755E051E3C9A9AA1AA0275554079415DF679BFF372C3F733A80FD3834`.

Tras revisión independiente, el runner `30A4538128A320CF7FB8C0A08D541DA182B83028BD7E9B46602883EEA667046D` adquiere un diálogo visible y espera la desaparición/ocultación del HWND original, conservando las comprobaciones posteriores. El intento `native-run-447da86b3bf44d14ba72c0d18555ee9f` confirmó importación real: tarjeta en Biblioteca y copia administrada de 968 bytes con hash idéntico a la fixture.

El mismo intento terminó con exit 1 en la espera del visor. Las últimas respuestas DOM siguen mostrando Biblioteca con «No se pudo completar la operación. Revisa la biblioteca.» después de pulsar Leer; no llegó a PaperReader. Por tanto, la causa pendiente corresponde a la apertura previa al visor, no a un fallo PDF.js demostrado. No hay protocolo, render, página/zoom/reapertura ni cierre normal acreditados. Resultado SHA256 `976C753550069248B55DB98CDB4596D95661A84B07C1B7D3AD3BEE583481D251`; cleanup completo y puertos 37171/37172 libres. Se conserva la biblioteca sintética para diagnóstico de solo lectura.
