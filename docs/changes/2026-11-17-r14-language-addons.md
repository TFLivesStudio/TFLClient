# R14 — idiomas declarativos y configuración

Implementación de trabajo de 0.15.4. Este informe distingue los contratos y tests
añadidos de la validación nativa/visual que debe registrar la matriz final.

## Configuración y compatibilidad

`Settings.locale` es la preferencia persistente. Si está ausente, se migra la
selección anterior `tfl-locale` del WebView al primer guardado satisfactorio. `es`
se normaliza como `es-419` y `en` como `en-US`; un error conserva el último valor
válido. Las escrituras usan la cola central y el control de revisión de Settings.
Las ventanas consultan esa misma configuración. El almacenamiento web anterior se
lee para migrar, sin convertirlo en una segunda fuente de preferencias nuevas.

Los paquetes `.tfl` de Beta 3 sin `languages` siguen siendo esquema/API 1. La
extensión opcional `languages` es declarativa, con su propio `schemaVersion: 1`.
Las generaciones inmutables, validación ZIP, inspección, consentimiento, actualización,
rollback, activación y desinstalación existentes se conservan. No se conceden
permisos de ejecución, red, cuentas, IPC, CSS ni acceso arbitrario a archivos.

## Contrato de un idioma

Cada contribución contiene locale BCP47 canónico, nombre, bandera PNG declarada,
dirección `ltr`/`rtl`, fallback incorporado `es-419` o `en-US` y mensajes. Se admite
un subconjunto documentado: lengua de dos/tres letras, script opcional, región de
dos letras/tres dígitos y variantes de cinco a ocho caracteres. Extensiones privadas
o locales fuera de ese subconjunto se rechazan; no se afirma soporte universal.
El código macroregional 419 identifica Latinoamérica, no un país ficticio «LATAM».

El catálogo versionado `src/lib/i18n/catalog.v1.json` define las claves y placeholders.
Se regenera con `node scripts/i18n-catalog.mjs`; `--check` detecta divergencias.
El generador inventaría diccionarios, `w`/`vr` y frases literales de `tr`/`authText`.
Las frases heredadas tienen un ID estable derivado del inglés; el runtime también
compara la fuente para evitar aceptar colisiones del hash. Las frases construidas
dinámicamente que no tienen entrada en el catálogo conservan su fallback incorporado;
no se promete traducirlas mediante un hash de texto cambiante.

Se rechazan claves desconocidas, cambios en placeholders, plurales sin `other`,
categorías desconocidas, etiquetas HTML, sintaxis activa y controles de dirección
invisibles. Los mensajes llegan a nodos de texto, nunca a `{@html}`. Las formas
plurales usan `Intl.PluralRules` del locale activo; variables ausentes permanecen
visibles como placeholders en lugar de desaparecer silenciosamente.

La activación, actualización y rollback rechazan locales duplicados entre paquetes
activos. El frontend además descarta todos los idiomas en conflicto de un snapshot
inválido, independientemente del orden de entrada. Al quitar/desactivar el paquete
activo o activar perfil seguro, la UI cambia inmediatamente al fallback y solicita
persistirlo por la cola de Settings. No se modifican cuentas, mundos ni otros Addons.

## Selector, recursos y ejemplo

`LanguagePicker.svelte` muestra bandera, nombre y locale; la bandera es decorativa
para accesibilidad. La bandera regional de Latinoamérica es un símbolo propio de
TFL en `static/flags/latin-america.svg`, no una bandera oficial ni una aprobación de
marca. Los recursos incorporados son locales; los Addons solo admiten PNG validados.
El atributo `lang` y `dir` del documento sigue el idioma. La infraestructura admite
RTL, pero esta entrega no añade una traducción RTL ni certifica todos sus layouts.

El archivo separado `examples/addons/portuguese-br.tfl` es un paquete real de idioma
con traducción **parcial**, bandera rasterizada, controles y pluralización. Su
descripción declara el fallback inglés y no lo presenta como traducción completa.
Puede reproducirse con `node scripts/build-language-example.mjs`. Sus fuentes
están en `examples/addons/portuguese-br/` y su instalación funciona mediante Addons.

Simple conserva permisos efectivos y autor declarado visibles. Los detalles de
esquema, API, hash y ausencia de firma verificable quedan en Avanzado. La instalación
sigue requiriendo consentimiento concreto; el selector de idioma guarda al elegir.

## Validación y límites

Los tests de `tests/unit/languages.test.mjs` ejecutan el módulo real compilado de
Svelte con Settings/IPC aislados: cambio en vivo, fallback de claves, interpolación,
plurales, actualización, desactivación, conflictos, contenido malicioso y fallo de
guardado. Son tests unitarios; no equivalen a una instalación nativa ni a una prueba
real con dos ventanas. También se conserva la suite de modelos Addons existente.

Los tests Rust añadidos en `commands/addons.rs` cubren el paquete real, compatibilidad
Beta 3, locales, placeholders, plurales, rutas, contenido activo, ciclo de
instalación/actualización/desactivación y rechazo de locales duplicados. Su resultado
nativo debe figurar en la matriz final; escribirlos no implica que se ejecutaron.
Quedan para validación integral la instalación por diálogo nativo, reinicio y dos
ventanas, lectores accesibles, expansión de textos y plataformas macOS/Linux.
