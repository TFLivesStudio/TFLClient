# Prueba manual antes de publicar

1. Instala el build de revisión de tu sistema; conserva tu instalación actual y sus
   datos. Abre Ajustes → Integraciones: MCP debe estar apagado inicialmente.
2. Activa MCP, copia la configuración del cliente y conéctalo. Antes de aprobarlo,
   pídele listar instancias: debe rechazarlo. Autoriza solo Lectura y repite.
3. Intenta instalar un mod: debe denegarlo con Lectura. Revoca y reconecta; autoriza
   Operación únicamente para una instancia desechable.
4. Crea o usa esa instancia, cambia RAM e instala una versión compatible de un mod.
   Comprueba dependencias, progreso normal del launcher y actualización de la vista.
5. Cambia estilo o escena mediante MCP: comprueba el cambio visible y reinicia el
   launcher. La preferencia debe persistir. Comprueba también la consola.
6. Concede Críticas solo para la prueba. Solicita un plan de borrado: no debe borrar
   nada. Recházalo; debe seguir existiendo. Solicita otro, revisa UUID/nombre y
   aprueba esa acción exacta. No pruebes con mundos personales.
7. Revoca el cliente, desactiva MCP y cierra el launcher. Las consultas deben fallar
   claramente. Una nueva conexión requiere una aprobación nueva.
8. Prueba forest/embers/void/aurora/pixel en Modern y Pixel, Dark/Light/OLED y ventanas
   800×600, 1280×720 y 1920×1080. Previsualiza, cancela, aplica y reinicia.
9. En Lite no debe haber vídeo ni WebGL automático. En Balanced prueba vídeo opt-in;
   en Experience prueba shader, pérdida de foco, minimizar, overlays y Minecraft.
10. En Windows/macOS/Linux verifica el auxiliar empaquetado, reproducción MP4/WebM,
    fallback estático y actualización beta/estable sin alterar la firma del updater.

Estos pasos nativos complementan las pruebas automáticas; no se consideran aprobados
por el mero hecho de que el instalador haya compilado.
