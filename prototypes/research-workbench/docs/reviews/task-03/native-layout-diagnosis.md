# T03 — diagnóstico de deformación visual del lector

Estado: DONE. Diagnóstico read-only; ninguna edición de producto, ejecución GUI, mutación SQLite o cambio de configuración.

## Evidencia del corte

Código: `.worktrees/task-03-reader`, HEAD `cb66da1124e553ddd5958f1d8619dc7a3686324d`.
Run: `work/qa/t03-native/native-run-55958ac426eb44d3b3088ff62d65b9b6/native-result.json`; log `work/evidence/task03-native-fix4-executed.log`.
Binario SHA256 `0C4BE713C8D2DD9618B8ACD6AF0A3C6E334D957DDA49B1530A36544A81CC7537`; runner8297 según asignación del orquestador.

Se leyó el resultado selectivamente, sin volcar screenshots base64. Se inspeccionó visualmente `03-reader-page1.png`: texto del PDF aplastado verticalmente. La UI circundante no presenta esa misma deformación. El éxito funcional/exit0 de la ejecución permanece válido dentro de su alcance; no prueba geometría correcta.

| Observación | Raster canvas | Rectángulo CSS |
|---|---|---|
| Página1, 100% | 612 × 792 | 611.99408 × 304.58334 |
| Página2, 100% | 612 × 792 | 611.99408 × 304.58334 |
| Página2, 110% | 674 × 872 | 673.99561 × 304.58334 |

El ancho CSS escala 1.10131; la altura CSS escala 1.0. La escala vertical aplicada al bitmap es aproximadamente 0.3846 al100% y 0.3493 al110%, mientras el ancho conserva aproximadamente un CSS pixel por pixel de raster. Esta diferencia explica la deformación. El redondeo de612×1.1 a674 y792×1.1 a872 es normal y mucho menor que el defecto observado.

## Causa concreta

`src/features/reader/pdfLoader.ts:90-92` calcula viewport con scale=zoom y asigna ambos atributos canvas.width/canvas.height con Math.ceil. La evidencia confirma que ambos cambian correctamente. `PaperReader.tsx:275` coloca ese canvas directamente en `.pdf-page-wrap`, sin estilo que determine una geometría alternativa.

En `src/app/tokens.css:359`:

```css
.pdf-page-wrap {
  display: flex;
  justify-content: flex-start;
  overflow: auto;
  max-height: calc(100vh - 200px);
  padding: 12px;
}
.pdf-page-wrap canvas {
  display: block;
  flex: none;
  max-width: none;
  height: auto;
  margin-inline: auto;
}
```

El contenedor usa flex-direction:row y flex-wrap:nowrap por defecto. El eje principal es horizontal y el transversal vertical. La alineación transversal efectiva es stretch porque no se declara align-items/align-self. El canvas tiene height:auto y márgenes verticales no automáticos.

La única línea flex queda limitada por max-height del contenedor. Stretch asigna al canvas esa altura de línea; no conserva su altura intrínseca. `flex:none` impide crecer/encoger en el eje principal, pero no desactiva stretch en el transversal. `margin-inline:auto` centra horizontalmente si sobra espacio y tampoco cambia la alineación vertical. El resultado es exactamente el patrón medido: ancho intrínseco variable y altura limitada constante.

La regla coincide con el algoritmo normativo: la línea única se limita por min/max del contenedor; un item stretch con tamaño transversal auto adopta la altura de línea. Ese paso no ajusta el eje principal aunque exista una proporción preferida. Fuente primaria consultada: [CSS Flexbox §9.4, Cross Size Determination](https://www.w3.org/TR/css-flexbox-1/#cross-sizing), y [§8.3, align-items/align-self](https://www.w3.org/TR/css-flexbox-1/#align-items-property).

El número exacto304.58334 procede de la medición nativa. El JSON no incluye innerHeight, dimensiones completas del wrapper y barra de desplazamiento suficientes para reconstruir cada término de ese límite; no se atribuye una altura de viewport inventada. La relación causal se sustenta en las reglas activas y el algoritmo, y la confirmación experimental de la corrección corresponde al siguiente corte QA.

## Corrección mínima propuesta

Añadir una sola declaración al contenedor:

```css
.pdf-page-wrap {
  align-items: flex-start;
}
```

Así el canvas mantiene su altura intrínseca y desborda verticalmente el contenedor limitado; overflow:auto ofrece el desplazamiento. Se conserva flex:none, max-width:none, height:auto y el centrado horizontal actual. Una alternativa equivalente para este único hijo es `align-self:flex-start` en `.pdf-page-wrap canvas`; basta una de las dos. No se propone fijar altura de página, distorsionar el raster, eliminar el límite del lector ni cambiar PDF.js.

## Verificación que debe hacer el autor/orquestador

En un motor de layout real, con una página más alta y ancha que el área disponible:

1. Al100%, el rectángulo CSS del canvas debe aproximarse a612×792, con tolerancia por redondeo/subpixel, y conservar la razón ancho/alto del raster.
2. Al110%, ambos ejes CSS deben aumentar aproximadamente1.1 y aproximarse a674×872; comprobar también que ambas escalas CSS/raster son iguales.
3. El wrapper debe tener scrollHeight > clientHeight y permitir llegar al pie de la página. Si el ancho excede el área, el desplazamiento horizontal debe permitir alcanzar ambos bordes.
4. Captura nativa legible y sin aplastamiento; repetir página2/zoom y reapertura usando el recorrido ya disponible.

Una prueba de atributos canvas.width/height o un test jsdom no verifica Flexbox ni este defecto. Debe evaluarse la geometría real, además del estado funcional ya comprobado. No se ha aplicado ni ejecutado aquí la corrección propuesta.

## Autorrevísión y límites

Se inspeccionaron únicamente PaperReader, pdfLoader, reglas CSS pertinentes y el resultado/captura del run asignado, más especificación primaria CSS para sustentar el mecanismo. Sin reabrir diagnóstico UTC o protocolo. No se afirma que el parche funcione hasta observar el corte corregido. Único archivo creado: este informe.
