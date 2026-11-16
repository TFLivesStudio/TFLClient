# Addons declarativos de TFL Client

API de paquetes **1.0.0**, disponible desde **0.15.4-beta.3**. Los Addons del
launcher son distintos de los plugins de servidores Minecraft. Esta entrega
permite temas y escenas locales. No tiene marketplace ni descarga remota.

## Instalar y recuperar

En Ajustes → Addons, selecciona un `.tfl`, inspecciona sus contribuciones,
compatibilidad y autor **declarado**, e instala. La instalación inicial permanece
desactivada hasta que pulses Activar. Después puedes elegir su tema o escena en
los selectores de apariencia existentes. No hace falta conexión a Internet.

Una actualización requiere un paquete con el mismo ID, una versión SemVer
superior y confirmación expresa. Conserva la activación anterior. Restaurar
recupera la generación anterior; desactivar o desinstalar revoca sus
contribuciones y el frontend vuelve a una alternativa integrada si estaba activa.
El modo seguro suspende todos los Addons sin borrar los paquetes ni sus estados
de activación. Para recuperar una interfaz ilegible antes de abrirla, inicia el
launcher con `TFL_ADDONS_SAFE_MODE=1`; mientras exista esa variable no se pueden
reactivar contribuciones. El modo seguro también se activa cuando se recupera un
registro dañado.

No se ejecuta código de Addons. El nombre de autor es una declaración, y el hash
identifica los bytes inspeccionados: ninguno acredita la identidad de un editor.
Revisa también un paquete de actualización antes de sustituir otro del mismo ID.

## Formato `.tfl`

Es un ZIP pequeño con `manifest.json` en la raíz y los PNG declarados. No añadas
entradas de directorio. El [ejemplo real](../../examples/addons/orange-dunes/manifest.json)
incluye un tema y tres capas de paisaje; el
[paquete instalable](../../examples/addons/orange-dunes.tfl) se reconstruye con:

```sh
node scripts/build-example-addon.mjs
```

El script genera PNG y un ZIP reproducible sin dependencias adicionales. Arte y
código originales de esta contribución, bajo la licencia GPL-3.0-only del
repositorio; no se incorporan recursos de terceros. Las siluetas usan funciones
periódicas para que los extremos conecten al repetir horizontalmente.

Campos requeridos del manifest:

| Campo                           | Contrato                                                                                                                                                                                       |
| ------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `schemaVersion`                 | `1`                                                                                                                                                                                            |
| `id`                            | Namespace estable, por ejemplo `tfl.orange-dunes`; 3–80 caracteres ASCII en minúsculas, segmentos separados por puntos. Cada segmento empieza por letra y sólo admite letras, números y guion. |
| `name`, `author`, `description` | Texto sin controles; límites de 80, 120 y 1200 bytes respectivamente. No contienen HTML interpretado.                                                                                          |
| `version`                       | SemVer completa, por ejemplo `1.0.0`.                                                                                                                                                          |
| `apiVersion`                    | Rango SemVer que incluya `1.0.0`, por ejemplo `^1.0.0`.                                                                                                                                        |
| `launcherVersion`               | Rango SemVer compatible con el ejecutable; para esta beta, por ejemplo `>=0.15.4-beta.3`. Los prereleases requieren comparador explícito conforme a SemVer.                                    |
| `permissions`                   | `[]`. No hay permisos de ejecución disponibles.                                                                                                                                                |
| `themes`, `scenes`              | Arrays opcionales; al menos una contribución en conjunto, hasta ocho por clase.                                                                                                                |

No se admiten campos desconocidos, scripts ni capacidades futuras en el esquema
actual. IDs de contribuciones: letra inicial, minúsculas/números/guion, hasta 48
caracteres. No se repiten dentro de su clase. Los IDs efectivos se califican como
`addon:<package-id>:<contribution-id>`; dos paquetes no sobrescriben contribuciones
entre sí ni las integradas. El propietario del launcher sigue controlando todos
los componentes y puntos de extensión.

### Temas

Cada tema tiene `id`, `name` y `tokens`. Los valores sólo admiten colores opacos
`#RRGGBB`. Claves permitidas:

```json
{
	"id": "example",
	"name": "Example",
	"tokens": {
		"--bg-main": "#17131B",
		"--bg-sidebar": "#201A24",
		"--bg-card": "#251E29",
		"--bg-input": "#17131B",
		"--bg-item-active": "#3A2935",
		"--text-primary": "#FFF7ED",
		"--text-secondary": "#E8D2C3",
		"--text-muted": "#C4ADAA",
		"--border": "#81666E",
		"--accent": "#FB923C",
		"--accent-text": "#21120A"
	}
}
```

No se aceptan CSS, URLs, selectores, fuentes externas ni HTML. Las claves omitidas
heredan el tema del anfitrión. El autor debe comprobar contraste en Modern y
Pixel, claro y oscuro, y con distintos fondos; la validación de sintaxis no es
una certificación de accesibilidad de un tema arbitrario.

### Escenas

Cada escena declara `id`, `name`, `background` (`#RRGGBB`, por defecto `#111827`)
y de una a seis `layers`, ordenadas desde el fondo al frente. Cada capa tiene
`path`, `speed` (0–20 píxeles lógicos por segundo hacia la derecha) y `opacity`
(0–1, por defecto 1). Usa PNG estáticos con transparencia; diseña cada imagen para
repetición horizontal sin costuras. No se admiten SVG, APNG, vídeo de Addons,
shaders ni código de renderizado externo. Las políticas de perfil, movimiento
reducido, pausa y FPS pertenecen al anfitrión y no pueden ser anuladas por un
paquete. El vídeo propio/importado del usuario continúa en su sistema existente.

## Límites y validación

- Paquete comprimido: 12 MiB; total declarado descomprimido: 16 MiB.
- Hasta 25 archivos, incluido un único `manifest.json` de hasta 64 KiB.
- PNG: hasta 4 MiB cada uno, entre 1 y 2048 píxeles por dimensión. Se verifican
  estructura, CRC de todos los chunks, dimensiones y decodificación real.
- Ratio ZIP por entrada: como máximo `comprimido × 200 + 4096` bytes.
- Sólo archivos regulares; sin enlaces, rutas absolutas, `..`, barras inversas,
  nombres reservados de Windows, colisiones de mayúsculas, entradas duplicadas,
  archivos no declarados, cifrado, ZIP64, multidisco o comentarios ZIP.
- Hasta 20 Addons instalados. Las actualizaciones conservan como máximo la
  generación previa más las referencias necesarias para recuperar el registro.

Los límites se comprueban antes y durante la lectura. Si un archivo cambia entre
la inspección y la instalación, el hash SHA-256 ya revisado deja de coincidir y
se exige una nueva inspección. Ninguna extensión por sí misma constituye
autenticidad o confianza.

## Almacenamiento y transacciones

El directorio administrado es `<perfil>/.tflclient/settings/addons/`, dentro del
perfil existente del launcher; `TFL_DATA_DIR` mantiene el aislamiento de pruebas.
`state.json` contiene activaciones y referencias a generaciones; su copia
`state.previous.json` permite recuperación en modo seguro. Cada generación de
`packages/<sha256>/` es inmutable e incluye el paquete original y recursos
extraídos. La API de lectura vuelve a verificar el archivo original y su hash;
no entrega rutas arbitrarias ni confía en los PNG extraídos si fueron alterados.

La instalación valida primero todo el ZIP acotado en memoria, escribe en una
carpeta temporal del mismo directorio, sincroniza archivos y publica la
generación mediante rename. El reemplazo atómico del registro es el punto de
commit. Un cierre antes del commit deja la versión anterior activa; un staging
sin referencia nunca activa contribuciones y se limpia en una mutación posterior.
Una actualización fallida conserva estado y paquete anteriores. La eliminación
revoca primero el registro; el recolector sólo elimina generaciones no
referenciadas con nombres administrados y rechaza enlaces. Una copia histórica
puede conservar temporalmente bytes de un paquete eliminado para la recuperación
del registro. No se borran instancias, mundos, preferencias ajenas ni archivos
desconocidos. Esta API no ofrece almacenamiento privado de código de Addons.

## Seguridad y SDK futuro

Consulta [la frontera de confianza y el diseño de extensiones](security-and-sdk.md).
Los tests del módulo Rust `commands::addons::tests` cubren archivos maliciosos,
compatibilidad, transacciones y recuperación. Los tests de ACL comprueban que la
ventana de log no recibe estos comandos. La evidencia de navegador simulado no
prueba por sí sola el instalador nativo ni la durabilidad ante un corte eléctrico.
