# Experience & Settings Rework — auditoría inicial

Base verificada: `v0.13.1-beta.4`, commit `1428f551cf5a1228c7bd761e787a39ff70a2912d`. Rama nueva: `feat/experience-settings-rework`. Las ramas anteriores y main quedan intactas. No publicar tags ni releases.

## Hallazgos comprobados

- SettingsPanel mezcla controlador y una vista extensa. La búsqueda usa palabras por pestaña: no localiza ni enfoca opciones.
- Apariencia tiene tema, VisualSettings con presets/material/acento/Home, otra paleta separada, fondos, superficie, densidad, fuente y calidad. El color personalizado y los colores oficiales son controles separados para el mismo propósito.
- La regla Pixel da borde/fondo/sombra tanto a inputs como a `.search-box`; los campos dentro de contenedores ya decorados reciben un segundo marco. Es una causa compartida, no un fallo de una barra concreta.
- Sidebar compacta pone un avatar de 32px dentro de un botón de 44px con otro marco. Los iconos de instancia también añaden un slot propio. Hay que distinguir los botones de navegación de los assets.
- Consola beta.4 dispone de líneas/anillos y muestreo real. Se conservará su persistencia y sondeo, separando render/layout del editor de demostración.
- Los títulos/iconos de resultados abren fichas al primer click. El doble click requiere coordinación de ambos eventos para no desmontar la fila.
- Fondos personalizados actuales son imágenes. Se necesita importación de vídeo al almacenamiento administrado, media-src limitado, validación de tamaño/formato y fallback.
- Los diálogos macOS se corrigieron en la beta más reciente: mantener comandos async y el modo manual existente.
- Java/JVM/env están persistidos; algunos controles globales no están expuestos. Se ofrecerán con validación y alcance explícito, sin modos separados.

## Arquitectura de implementación

1. Catálogo tipado de opciones compartido entre rótulos, grupos y búsqueda profunda. Cada resultado apunta a un control real y conserva el contexto de búsqueda.
2. Siete páginas por tarea: General, Apariencia, Fondos y escenas, Minecraft y Java, Cuentas, Consola y monitoreo, Actualizaciones.
3. Escenas versionadas aparte de preferencias existentes; ninguna migración destructiva de fondos. Capas 2.5D originales, vídeo local administrado, política central de actividad/calidad y render de miniaturas estático.
4. Un layout y tarjetas de métrica compartidos por consola real/editor; datos demo explícitos sin sondeo. Aplicar/cancelar/deshacer con esquemas válidos y orden accesible.
5. Biblioteca de SVG originales con el guardado por instancia existente, manteniendo los PNG y las importaciones admitidas.
6. Perfil Rust de revisión separado; no tocar la firma, updater ni el perfil oficial sin evidencias.

## Límites que se comprobarán

Los codecs dependen del WebView/OS. Sin descargar codecs: detectar fallo y mantener imagen estática. Los materiales nativos y la autenticación/juego real requieren prueba manual por plataforma. La alternativa 2.5D evita añadir un motor 3D completo. No se declararán pruebas con usuarios ni métricas de GPU inexistentes.
