# Evidencia Pixel y Living Themes

Capturas del frontend real compilado con IPC simulado; no son una ejecución
nativa ni una sesión de Minecraft. La base es el merge `7dc7f25` de beta.7 y
main. `before-pixel.png` y `after-pixel.png` comparan Home a 1280×720 con Pixel
Balanced. Los tres archivos `*-balanced.png` muestran paisajes en Modern y
`pixel-window.png` comprueba el fondo de ventana con Pixel.

`settings.json` contiene comprobaciones de alcance y preferencias.
`landscapes.json` registra observaciones CDP y pruebas del renderizador. Las
mediciones son de Chromium y no deben usarse como cifras de CPU/RAM nativas.
El conjunto completo se genera en `artifacts/` y se adjunta en CI.
