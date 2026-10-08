# T04c fix2 — revisión del orquestador

2026-10-05. Producto congelado5d35f03b13c4e85a0944296bac20e593cf494b5c; BASE fix2 1f0fbdf7d4a422bdf14b9aafdcd74dbffdcfe8db; HEAD67446412f9c2f82cab63ccb5bf02d3dff695bb84 sólo añade informe41líneas. Autor limpio verificado.

Leído TODO delta3archivos: hook39líneas modificadas, vista8 y tests221 añadidas. F1: montaje usa reload completo; selectedPhaseRef mantiene callback estable y consulta elegida, estado sólo limpia refreshRequired tras Paper/fase/activa pertinentes y versión de confirmación coincidente. Todas cuatro ramas de confirmación piden refresh mediante helper. Aviso/retry existe sin depender del error borrado al consultar fase. Generaciones previas se conservan; no replay confirmado ni nuevo canonstore. F2: GateBlocked borra gate, conserva resolución conocida y requiere reevaluación explícita. No hallazgos importantes adicionales por lectura.

Pruebas nuevas afirman cinco regresiones originales, fallo fase activa y lectura anterior a confirmación; tokens vigentes y no reenvío. Root leyó logs y sidecars RED5fallos/49PASS exit1 y GREEN56PASS exit0, tsc-b exit0. No es ejecución propia ni QA nativa. Prueba concurrente usa dos consumidores sintéticos para aislar protección; App real normalmente monta uno. Archive/lifecycle cubierto por lecturas canónicas y pruebas previas, sin atribuir esta evidencia a SQLite.

Informe autor y copia central hashes iguales64A0A5726A3ADF4FADE7F6CA267E90D82D527FB2112A4F2F2633DBFA1179B5C4. Diff-check producto exit0. Minor fixtureReader de fix1 sigue diferido, no afecta F1/F2. Revisiones independientes SPEC/TS en curso; sólo después se prepara merge y gate/build, antes de confirmar integración se exige QA nativa pertinente. No T04c integrada ni T05 activada.
