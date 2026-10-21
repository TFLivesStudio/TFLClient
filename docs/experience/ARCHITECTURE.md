# Experience & Settings Rework

Base inicial: `v0.13.1-beta.4`. Se integró `origin/main` (`cf049e7`, publicación de `0.13.1` estable durante la misión). Versión preparada: **0.13.2-beta.1**. Rama independiente `feat/experience-settings-rework`; sin tag ni release.

## Ajustes por tarea

`settings/catalog.ts` define las siete páginas y los grupos canónicos. `SettingGroup` obtiene de ese mismo catálogo su título e ID. El buscador compara cada término de la consulta con etiquetas y sinónimos ES/EN, ignorando mayúsculas y tildes; cada resultado abre, desplaza, enfoca y destaca el grupo real. Conserva la consulta para volver a los resultados. `openSettingsAt` permite accesos desde Home, instancia y los propios ajustes sin crear rutas de configuración paralelas.

La [matriz de doce columnas](SETTINGS_MAP.md) documenta cada opción. Apariencia reúne estilo, tema/superficie, acento, material, tipografía, densidad, calidad, movimiento y Home, en ese orden. Fondos y escenas tiene página propia. No hay modos Simple/Avanzado. Java, argumentos y variables usan los servicios de lanzamiento existentes; las rutas vacías conservan la selección automática. Las validaciones y errores de guardado permanecen visibles. No se ejecuta un shell para aplicar argumentos ni variables.

Un campo heredado, `discord_presence`, no tiene consumidor RPC en el repositorio: se conserva en los datos sin ofrecer un control que simule una integración activa. Las credenciales y tokens siguen en sus servicios; no entran en búsqueda, preferencias visuales ni demos.

## Escenas y vídeo

`scenes/model.ts` valida una estructura versionada (`tfl-scenes-v1`) separada de los datos de juego. Un valor global y overrides completos por UUID permiten heredar/restablecer por instancia. Se conservan `tfl-wallpaper`, su imagen personalizada y la ambientación anteriores: la escena predeterminada es «Tu fondo actual».

Siete escenas originales usan tres SVG por composición: cielo, horizonte y primer plano. Miniaturas y Lite son imágenes, sin motor. La hora local ajusta la luz sin ubicación ni llamadas meteorológicas. Lluvia, niebla y nieve son decorativas. El movimiento de cámara responde al puntero mediante transformaciones; no mantiene un bucle de sondeo.

Los presets Aurora Drift y Emberlands incluyen bucles originales MP4 H.264 y WebM VP8 de seis segundos, sin audio, 480×270, 24 fps. La biblioteca admite vídeos MP4/WebM locales con 128 MB máximos y 12 entradas. El backend valida extensión, cabecera y tamaño, serializa las mutaciones, copia primero a un temporal y guarda un identificador administrado. Las URLs de vídeo no se aceptan desde preferencias. La reproducción usa el decodificador disponible del WebView; no instala códecs ni ejecuta procesos externos. Si falla, queda la imagen de respaldo. Se puede elegir también una imagen existente como respaldo.

`media-src` y el asset protocol solo amplían acceso a la carpeta administrada `shared/appearance/scenes`. Los cuatro comandos nuevos se registran mediante el generador ACL existente; las pruebas verifican que la ventana de consola no los recibe. Los vídeos en uso no se eliminan desde el selector.

## Política de coste y ciclo de vida

`sceneCapabilities` combina calidad, accesibilidad, actividad y política durante el juego. Lite nunca crea vídeo ni WebGL de fondo. Balanced admite vídeo de forma explícita, sujeto a los controles globales de movimiento. «Detener efectos continuos» tiene prioridad, con explicación y acceso a su opción. Experience admite luz procedural opcional. Las previews empiezan quietas y requieren Reproducir; la escena de fondo se pausa detrás de overlays para evitar dos reproductores activos.

Una escena se detiene al ocultarse o perder foco, al reducir movimiento o según la política de Minecraft abierto. Su vídeo se desmonta, el shader pierde su contexto al limpiarse y los listeners de puntero se liberan. Una promesa `play()` interrumpida no convierte la próxima reproducción en un error permanente. Los importes y las operaciones de juego no esperan animaciones.

`frameClock.ts` mantiene una sola fuente de requestAnimationFrame para shaders y personaje. Cada suscriptor tiene un límite de FPS; sin suscriptores no queda RAF activo. El visor conserva `renderPaused=true` en skinview3d y avanza animación/render desde el reloj compartido. Mantiene carga diferida, observación de intersección, disposición al cubrirse y DPR limitado a la densidad real y al perfil. Lite conserva la activación manual del estudio. Los FPS configurados son un máximo de efectos procedurales/personaje; no se promete cambiar los FPS codificados del vídeo ni los del juego.

El shader opcional es una capa de iluminación mínima, sin dependencias nuevas, limitada a 1600×900 y DPR 1.5. No hay una escena voxel 3D completa: las composiciones 2.5D proporcionan profundidad sin un segundo motor pesado.

## Respuesta a operaciones reales

Las reacciones de luz y mascota/personaje consumen lanzamiento real, resultados del sistema de descargas y tareas reales en curso. El personaje puede caminar al lanzar, saludar al completar o dar un gesto de error. Son adornos acotados por perfil, no progreso simulado; los errores y notificaciones existentes siguen siendo la información operativa. No se cambia autenticación.

La navegación Biblioteca → detalle explora View Transitions compartiendo el icono existente. Se detecta la API, se limita el nombre a un elemento y se limpia al terminar; las webviews sin soporte conservan el cambio normal y su reveal. La navegación no depende del éxito del efecto.

## Consola compartida

`MetricCards`, `LogBlock` y `ConsoleLayout` son renderers compartidos entre consola real y diseñador. `ProcessCharts` conserva el muestreo real: una petición pendiente, intervalos de un segundo, pausas por visibilidad, huecos al faltar datos y limpieza al salir. No se calcula CPU ficticia; por núcleo puede superar 100%. RAM es memoria residente; disco utiliza bytes y tiempo realmente muestreados.

El diseñador importa un dataset fijo rotulado como demostración y nunca instancia el sondeador. Cinco moldes se traducen a posiciones válidas existentes, sin coordenadas arbitrarias. Asas de puntero, drag/drop y flechas mantienen el orden dentro de los moldes. Tiene selección de bloque, inspector, Aplicar, Cancelar, Restablecer y una pila de 20 cambios para Deshacer. Los cambios del editor no se guardan antes de Aplicar. Un borrador sin aplicar se conserva ante cambios de otra ventana; Cancelar toma la preferencia actual.

Los valores globales de líneas/anillos y ventana temporal de beta.4 migran a mapas por métrica. Colores, orden y visibilidad permanecen. El texto del registro comparte niveles, bordes y colores con la vista previa; Warn/Error se distinguen también por su etiqueta. La sincronización usa el almacenamiento y evento entre ventanas ya existentes, sin ampliar privilegios de la consola.

## Iconos, búsquedas y contenido

36 SVG originales se añaden a los presets empaquetados existentes, con búsqueda, categorías y preview antes de aplicar. El backend acepta SVG solo de esos recursos de confianza; las subidas propias conservan los formatos raster anteriores. Guardar copia el nuevo archivo antes de retirar extensiones antiguas. Los PNG históricos y sus licencias se mantienen.

En Pixel el contenedor de búsqueda posee el marco; su input interno queda sin borde/sombra/fondo adicionales. La corrección abarca sidebar, Biblioteca, Ajustes, explorador y selector de iconos, sin modificar búsquedas Modern. Los assets de cabeza/avatar se muestran sin un segundo slot decorativo dentro del botón compacto.

El doble clic de título/icono coordina el primer clic antes de abrir una ficha. El botón de información y el acceso por teclado mantienen apertura inmediata. La instalación llama al dispatcher existente, con sus compatibilidades y dependencias. La coalescencia por instancia/tipo/proyecto y el refresco esperado de instalados evitan duplicados pendientes/completados. Los modpacks no reciben instalación rápida y conservan su flujo de confirmación.

## Materiales nativos y builds

La base sigue siendo CSS matte/soft glass/glass, con degradación sólida en Lite y sin blur. Se revisaron las opciones nativas de Tauri: Mica/Acrylic dependen de la versión de Windows y del compositor; Vibrancy requiere cambios de ventana/material en macOS. La ventana actual es opaca y compartida por plataformas. No se activan cambios nativos de transparencia sin validación real, especialmente con el historial de macOS. Los materiales CSS son la alternativa funcional disponible, también en Linux.

El perfil oficial `release` permanece idéntico. El perfil opt-in `review` usa opt-level 2, ThinLTO y 8 unidades de generación, con incremental apagado. `.github/workflows/review.yml` construye instaladores manuales con caché existente y sin secretos ni artefactos del updater. El overlay de configuración de review no modifica el flujo oficial de firma/publicación. No se promete una mejora de segundos nativos hasta tener mediciones comparables en runners; sccache y linkers alternativos quedan documentados como opciones posteriores, para no mezclar caches, toolchains y estabilidad en esta misión.

## Revisión pendiente por plataforma

La evidencia automática es Chromium con IPC simulado, pruebas Rust de archivos reales/ACL/telemetría y observaciones CDP. No equivale a probar inicio de sesión Microsoft, Minecraft con una cuenta real, códecs de cada OS, notarización, GPU física, autonomía energética ni todos los controladores. Tampoco se afirma haber hecho un estudio con usuarios: las tareas de descubrimiento se verifican mediante búsquedas y destinos reales. Esas comprobaciones manuales se enumeran en [TESTING.md](TESTING.md).
