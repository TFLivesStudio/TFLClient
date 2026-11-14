# Contrato visual Pixel

Pixel conserva las primitivas Svelte y sus eventos. `data-visual="pixel"` cambia
la presentación; no selecciona un escenario ni modifica las operaciones del launcher.
Las reglas compartidas viven en `src/styles/refresh.css`.

## Superficies y estados

| Token                                    | Responsabilidad                                                 |
| ---------------------------------------- | --------------------------------------------------------------- |
| `--pixel-stone`                          | Superficie de botones y controles elevados, derivada del tema   |
| `--pixel-slot`                           | Campo editable y ranura de contenido                            |
| `--pixel-rim-light`, `--pixel-rim-dark`  | Bisel iluminado y contorno exterior                             |
| `--pixel-action`, `--pixel-action-hover` | Acento configurado para acciones primarias                      |
| `--pixel-action-text`                    | Primer plano del acento, con la política de contraste existente |
| `--pixel-focus`                          | Anillo de teclado separado del bisel                            |
| `--pixel-outline`                        | Grosor del marco compuesto, 1 px                                |
| `--pixel-control-height`                 | Altura mínima de interacción, 36 px                             |

Pixel 2.0 conserva un contorno exterior de 1 px y un relieve de uno o dos píxeles
interiores. El peso visual procede del bisel y de la jerarquía, sin aumentar
los marcos de tarjetas y diálogos. La escala de espaciado es 4/8/12/16/24 px;
los controles compactos, normales y prominentes mantienen variantes propias.

Los botones conservan un solo relieve. Hover modifica la superficie, pressed
invierte el bisel y el foco conserva un contorno de 2 px separado del borde.
Los controles deshabilitados conservan su atributo nativo y reducen su opacidad.
Las acciones destructivas tienen borde semántico y fondo teñido; la variante ghost
reserva la superficie para hover. No se fuerza verde sobre un acento personalizado.

Las primitivas existentes (`.tfl-btn`, `.vr-control`, etc.) comparten ese contrato.
Los controles locales optan explícitamente mediante `data-pixel-control`; el valor
`primary` o `danger` identifica su intención. Las opciones conservan sus clases
`active`/`selected` y `aria-pressed`. No se decoran indiscriminadamente todos los
botones: iconos, muestras de color y áreas de contenido mantienen su función visual.

## Búsquedas compuestas

`data-search-control` pertenece al contenedor de icono, input y acciones opcionales.
El contenedor posee fondo, marco, bevel y foco. El input posee únicamente texto,
caret y edición: borde cero, sombra ninguna y fondo transparente. Las reglas
genéricas de inputs y de diálogos excluyen ese campo en Pixel para evitar que una
mayor especificidad vuelva a crear el segundo marco o una altura interior excesiva.

El contrato cubre Biblioteca, barra lateral, Ajustes, paleta de comandos, iconos,
explorador, gestor de contenido, modpacks y el diálogo de Modrinth. El filtro
de consola es un campo independiente y conserva un solo marco propio. La búsqueda
de la barra de título es un botón que abre la paleta, no un campo editable.

Cada buscador conserva filtros, debounce, atajos y navegación propios. Modern
conserva sus reglas locales; los atributos de contrato no modifican su apariencia.

## Tipografía y accesibilidad

Readable utiliza Monocraft en títulos y navegación distintiva. Classic amplía
Monocraft a controles y búsquedas. El texto técnico, logs y editores conserva
JetBrains Mono. El contenedor de búsqueda crece con su contenido y no impone una
altura fija que recorte fuentes grandes. Los inputs flexibles tienen `min-width: 0`.

Los colores de búsqueda derivan del tema. Los tooltips también respetan superficie,
texto y acento. Forced colors usa Canvas/CanvasText y Highlight en el foco; no depende
de una sombra decorativa para comunicar la selección de teclado.

## Verificación reproducible

`tests/visual/pixel-controls.mjs` utiliza el frontend real y el fixture IPC aislado.
Comprueba estilos calculados, marcos, sombras, fondo, foco, nombres accesibles y
geometría de seis buscadores principales; también preservación del filtro de
Biblioteca, filtro de comandos y Escape. La matriz recorre Readable/Classic,
Dark/Light/OLED, tres calidades y materiales, acentos personalizados, DPR 1/2 y
800×600, 1100×700, 1280×720, 1366×768 y 1920×1080. Verifica los colores renderizados
de la acción primaria, inglés, movimiento reducido y forced colors.

Las capturas y métricas se escriben en `artifacts/pixel-controls`. Estas pruebas
de Chromium no certifican lectores de pantalla ni WebView nativo en cada OS.
