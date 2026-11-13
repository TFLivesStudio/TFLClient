# Nombres del ecosistema TFL

## Regla editorial

- **TFL Client:** launcher.
- **TFL Selection:** biblioteca donde TFLives publica sus clientes modificados.
- **TFLClient:** cliente final de Minecraft.
- **TFL Core:** componente que impulsa funciones dentro de TFLClient.

Referencia: [nombres y términos](../PRODUCT_NAMES.md). La web incorpora el
[glosario bilingüe](https://launcher.tflives.com/ayuda/glosario).

## Alcance

Se actualizaron interfaz, textos ES/EN, nombres sugeridos de nuevas instancias,
notas visibles y documentación en TFLClient, TFLClientWEB, TFLClientMOD y
TFLClientSelection. Las ramas de desarrollo activas del launcher y del cliente
reciben correcciones equivalentes sin mezclar sus implementaciones con main.
La revisión de los otros repositorios de la organización no encontró estos
nombres en sus README. No se cambiaron repositorios ajenos al producto.

La normalización de nombres antiguos ocurre al mostrar el catálogo. No renombra
instancias existentes ni cambia IDs, rutas, namespaces, claves guardadas, tags,
assets o bytes firmados. El publicador usará TFLClient en nuevas generaciones.
No se crearon releases binarias: las interfaces de aplicaciones distribuidas
recibirán los textos cuando se compile y publique su próxima versión.

Se corrigió la nomenclatura en 25 releases ya publicadas: 4 del launcher, 20 del
cliente/Core y una de la biblioteca. Se preservaron sus assets y metadatos de
publicación; las notas no atribuyen funciones nuevas a versiones anteriores.
Los borradores y la evidencia histórica original conservan su contenido.

Durante la edición inicial de un borrador del cliente RC1, GitHub regeneró su
etiqueta temporal y las URL provisionales `untagged-*` de seis assets. Se
restauraron el tag configurado, nombre, cuerpo, estado draft/prerelease y commit
originales. Los IDs, nombres, tamaños y hashes de sus assets siguen iguales;
las URL temporales de ese borrador fueron regeneradas por GitHub. No se publicó
el borrador ni se cambiaron las URL de ninguna release publicada. La revisión
posterior se limitó a releases publicadas.

## Validación

- Launcher main: comprobación Svelte/TypeScript, ESLint/Prettier y build correctos;
  25 pruebas unitarias correctas. Svelte informa siete advertencias existentes.
- Web: 19 pruebas unitarias, 27 pruebas Playwright, TypeScript, ESLint y build
  correctos; cuatro vistas adicionales en escritorio/móvil y ambos temas.
- Biblioteca: cinco JSON validados y 45 pruebas Python correctas; catálogo
  publicado, firmas y manifiestos publicados intactos.
- Cliente: JSON de traducciones válidos. En cuatro ramas candidatas se compiló
  y ejecutó la clase Product real para verificar TFLClient en ambas versiones de
  Minecraft sin cambiar ID ni carpeta de configuración. No se ejecutó un build
  completo de Minecraft: el entorno no dispone del JDK 25 requerido y Gradle
  no pudo descargar su distribución con la conexión Java disponible.
- Las tres ramas del launcher pasan compilación sintáctica de los componentes
  Svelte y traducciones modificados; no sustituye su matriz de pruebas nativas.

## Descripciones de GitHub pendientes

La integración permite cambios de archivos y releases, pero rechaza con HTTP 403
(Resource not accessible by integration) la edición de la descripción y homepage
del repositorio. No se modificaron esos ajustes. Para mantener About alineado,
estos son los textos preparados para su edición desde GitHub:

| Repositorio        | Descripción                                                                                                                       |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------- |
| TFLClient          | TFL Client: launcher de TFLives para instalar y abrir Minecraft y TFLClient desde la biblioteca TFL Selection.                    |
| TFLClientMOD       | TFLClient: cliente modificado de Minecraft de TFLives, impulsado por TFL Core y publicado en la biblioteca TFL Selection.         |
| TFLClientSelection | TFL Selection: biblioteca oficial de clientes modificados de TFLives, incluido TFLClient. Se accede desde el launcher TFL Client. |
| TFLClientWEB       | Web oficial del launcher TFL Client, la biblioteca TFL Selection y el cliente modificado TFLClient.                               |

Homepage oficial: https://launcher.tflives.com.
