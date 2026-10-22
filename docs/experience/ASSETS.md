# Recursos originales

## Imágenes de escenas y fondos de pantalla

`static/scenes/cinematic/*.webp` (siete escenas) y `static/wallpapers/pack-*.jpg` (fondos adicionales) se generan a partir del paquete de imágenes aportado por el equipo (capturas con shaders, arte oficial de Minecraft y obras de la comunidad). **No son recursos originales de TFL ni están cubiertos por la GPL-3.0 del repositorio**: antes de distribuir una versión pública hay que confirmar su licencia o permiso de uso (arte de Mojang/Microsoft y créditos de los autores de la comunidad). Una imagen usada como escena no se repite en la lista de fondos de pantalla, y viceversa.

`static/scenes/*.svg`: geometría vectorial original de TFL creada para esta misión; siete escenarios con capas separadas y composición estática. Sin imágenes extraídas de Minecraft ni otros launchers. Misma licencia GPL-3.0-only del repositorio.

Los vídeos de aurora y brasas se generan desde fórmulas procedurales locales con FFmpeg (sin fotos ni vídeo de terceros), a 480×270 / 24 fps. Incluyen WebM VP8 y MP4 H.264 para ofrecer alternativas al decodificador de cada WebView. No se descargan codecs. Un fallo de reproducción mantiene la composición SVG.

## Instance emblems

36 original SVG silhouettes in `src-tauri/resources/instance-icons`: nature, biome, artifact, creature, sigil and gear. Hand-defined vector paths, no raster embeds, scripts, external references or copied third-party artwork. Existing legacy PNG presets and their licenses remain intact.
