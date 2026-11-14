# Personaje interactivo

El visor sigue usando `skinview3d` 3.4.2, cargado bajo demanda, y el reloj compartido
de `scenes/frameClock.ts`. La dependencia se crea con `renderPaused: true`, sin
animación interna ni auto-rotación. Los controles orbitales se deshabilitan:
un único controlador de puntero gobierna la orientación y conserva la vista
elegida hasta pulsar Vista frontal o Inicio. Flechas giran; Enter/Espacio saludan.

`visual/character.ts` contiene un controlador numérico independiente de Three.js.
Sus poses originales interpolan cabeza, torso, brazos y piernas sin cambiar clips
abruptamente. No gira el cuerpo por tiempo. El seguimiento limita cabeza a ±20°
horizontal y ±10° vertical. Hover reconoce al usuario como máximo cada doce
segundos; el saludo explícito sigue disponible. Preparación usa tareas reales de
descarga; lanzamiento, éxito y error consumen los eventos existentes del launcher.

## Capa: nivel 1

Se inspeccionaron `node_modules/skinview3d/libs/model.js`, `animation.js` y
`viewer.js`. La capa original es una caja rígida `BoxGeometry(10,16,1)` con pivote
en los hombros, no una malla de tela articulada. Se conserva su geometría, UV y
textura. Dos springs con velocidad, rigidez y amortiguación gobiernan inclinación
y balanceo. Reciben fuerza de la velocidad de giro manual, aceleración vertical y
pose del torso/piernas. Los pasos se subdividen hasta 120 Hz, con delta máximo de
100 ms y límites que evitan inversión y oscilación descontrolada.

Esto es inercia de una capa rígida, **no** simulación de tela, deformación por
segmentos ni garantía de ausencia de intersecciones en cualquier pose. El nivel 2
requeriría otra geometría/UV y validación visual adicional; no se modifica la
dependencia para simular compatibilidad inexistente.

Sin capa visible no se integra ningún spring. Balanced reduce la fuerza y limita
el reloj a 24 FPS; Experience admite el límite elegido hasta 60. Lite conserva la
skin estática y permite inspección 3D manual sin bucle continuo. Reduced motion,
ventana oculta, overlays sobre Home y política de juego suspenden las actualizaciones.
Las rotaciones explícitas del usuario pueden dibujar un único cuadro estático.

## Validación

`tests/unit/character.test.mjs` comprueba orientación estable, límites e
interpolación, retorno de saludo/cooldown, cero integración sin capa/reduced y
convergencia de springs a 15/24/60 Hz. `tests/visual/character-interaction.mjs`
utiliza texturas sintéticas de skin/capa y el IPC aislado con el WebGL real de
Chromium: controles de teclado, saludo, drag, restauración frontal, limpieza de
overlay, juego ejecutándose y fallback tras pérdida de contexto. Cuenta llamadas
de dibujo/RAF en Lite y reduced para detectar trabajo continuo. Las capturas se
guardan en `artifacts/character-interaction/`. Estos resultados no certifican GPU
nativa, inicio de sesión real ni la ejecución de Minecraft.
