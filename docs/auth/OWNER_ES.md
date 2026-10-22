# Microsoft Authentication 2.0 — guía del propietario

Base: `v0.14.0-beta.6`. Entrega en PR borrador, **sin release ni cambio de Client ID
de producción**. El Client ID identifica una aplicación pública: no es un secreto.
Los access/refresh tokens, códigos de autorización y verificadores PKCE sí lo son.
Nunca se necesita un Client Secret.

## Estado y configuración

El punto de configuración es `src-tauri/src/core/microsoft_identity.rs`.
`PRODUCTION_CLIENT_ID` sigue apuntando al registro existente de CubicLauncher.
No se suministró el GUID real de TFL ni evidencia de aprobación de Minecraft;
no se inventó ninguno. El modo automático conserva código de dispositivo para
ese registro conocido. Los otros métodos se pueden elegir explícitamente, pero
no se afirma que un tercero tenga registradas las nuevas URI.

Las variables de ejecución, ajustes, archivos remotos y herramientas MCP **no**
cambian la identidad OAuth. Las variables siguientes son de **compilación** y
se ignoran sin la feature `auth-test-client`:

```sh
# Sustituir el marcador por el GUID real del registro propio, no por un secret.
TFL_AUTH_TEST_CLIENT_ID='<Application (client) ID real>' \
TFL_AUTH_TEST_ACK='development-only-not-for-release' \
bunx tauri dev --features auth-test-client
```

PowerShell:

```powershell
$env:TFL_AUTH_TEST_CLIENT_ID = '<Application (client) ID real>'
$env:TFL_AUTH_TEST_ACK = 'development-only-not-for-release'
bunx tauri dev --features auth-test-client
```

Un valor ausente, inválido o sin reconocimiento explícito bloquea el login de esa
build de prueba. No cae silenciosamente al registro anterior. La interfaz muestra
que es una build de prueba. No distribuir estos binarios por el canal oficial.
El build oficial existente no habilita esa feature.

## Registro Microsoft y prueba antes de activación

En el registro de aplicación pública de escritorio, permitir cuentas Microsoft
personales y flujo de cliente público. Sin secreto. Comprobar las URI según
el flujo, sin reemplazarlas unas por otras:

- Ventana integrada: `https://login.microsoftonline.com/common/oauth2/nativeclient`.
- Navegador del sistema: `http://localhost`. El receptor usa un puerto efímero y
  ruta `/`; Microsoft documenta la excepción de puerto para loopback. El registro
  heredado conserva `/callback` para pruebas explícitas con su configuración previa.
- Código de dispositivo: no utiliza callback.

Se usan los endpoints v2.0 `consumers` y los scopes `XboxLive.signin offline_access`.
La sesión termina en Xbox Live → XSTS → Minecraft Services → perfil real.
No hay formulario Microsoft recreado, iframe, servidor OAuth propio, Graph scopes
innecesarios ni intercambio de tokens en Svelte.

Probar el ID propio únicamente en una build de prueba. Una respuesta de Minecraft
`403` con `Invalid app registration` en `error` o `errorMessage` se muestra como
rechazo del registro. No demuestra un problema de contraseña/licencia, y no activa
otro Client ID. Tampoco se afirma que todo 403 sea falta de aprobación.
Comprobar además MFA, permisos familiares, licencia/perfil, varias cuentas y los
almacenes del sistema en cada SO. No pedir al usuario contraseñas ni tokens.

## Activar producción después de la aprobación

1. Conservar evidencia de aprobación del ID exacto y autorización del propietario.
2. Verificar una cuenta real en build de prueba, con navegador y código, y repetir
   la matriz manual documentada. Guardar resultados sin credenciales.
3. En otro PR explícito, cambiar solo la configuración de producción al GUID
   aprobado. Revisar el comportamiento automático asociado a `custom_identity`:
   un ID propio selecciona ventana integrada con alternativas visibles.
4. Avisar que las cuentas Microsoft anteriores necesitarán autorizar de nuevo.
   No borrar perfiles, instancias, ajustes ni onboarding ya completado.
5. Solo tras revisión, preparar la versión y la publicación que corresponda.
   Esta misión no crea tags, no integra el PR y no publica una release.

Las credenciales protegidas guardan el Client ID que las emitió. La renovación
compara ese emisor con la identidad de la build antes de enviar un refresh token.
Un emisor distinto requiere **Ajustes → Cuentas → Renovar sesión**. La renovación
de una cuenta exige el mismo UUID; otra cuenta devuelta por Microsoft se rechaza
sin reemplazar el perfil original.

## Almacenamiento y migración

`keyring-core = 1.0.1` y los backends nativos fijados (Windows 1.1.0, Apple
1.0.2, Secret Service 1.0.1) utilizan Windows Credential Manager, macOS Keychain o
Linux Secret Service. Se elige el backend del SO explícitamente y se reinicializa
en cada operación para que Reintentar pueda recuperar un servicio que vuelva a estar disponible. Una clave AES-256 aleatoria por cuenta/proveedor se guarda allí, con
lectura de verificación. Los tokens se cifran con AES-GCM y nonces aleatorios en
`dirs::data_dir()/TFLClient/sessions-v2/<hash>.enc`. Esto evita el límite de tamaño
de credenciales genéricas de Windows sin almacenar tokens o claves en texto plano.
En Unix se crean directorios 0700 y temporales 0600; el archivo se reemplaza por
rename después de escribir y sincronizar. El nombre es un hash del proveedor/UUID.

El antiguo `claunch-rs/secrets` solo se lee para migración. Se descifra su formato
heredado, escribe y verifica el formato nuevo, y solo entonces elimina los archivos
anteriores. Si falla el almacén o la migración, se conserva la cuenta y sus archivos;
no se cae al cifrado antiguo. Una migración concurrente no debe reemplazar una
sesión recién autorizada. Las mutaciones, renovación y eliminación comparten bloqueo.
Las preferencias generales no pueden reemplazar el estado de cuentas en memoria.

Sin almacén disponible: desbloquear/configurar el llavero y reintentar, o elegir
explícitamente **Usar sesión temporal**. La sesión temporal vive en memoria y marca
el perfil para que al reiniciar exija autorización. La elección explícita descarta
los blobs de tokens de esa cuenta, incluso si un guardado anterior falló parcialmente;
si no pueden eliminarse, no se declara una sesión temporal correcta. Guardar otros ajustes no borra
sus tokens en memoria. No se presenta como guardada permanentemente.
Si falla el guardado local de perfiles, no se declara éxito; se permite reintentar.
Al eliminar una cuenta, un fallo al borrar credenciales es visible y conserva la
cuenta en vez de fingir que se desconectó.

En Linux se necesita una sesión D-Bus con Secret Service disponible y desbloqueado
(por ejemplo GNOME Keyring o un proveedor compatible). No se instala ni configura
un llavero silenciosamente. Los entornos sin escritorio pueden usar sesión temporal.
Las ubicaciones de credenciales son del usuario del SO, no contenido de instancias
ni datos que deba exportarse en backups. No copiar las antiguas carpetas de secretos
a informes, tickets o artifacts de CI.

## Navegador integrado y alternativas

La ventana `microsoft-auth-<operación>` usa la API estable `WebviewWindowBuilder` de
Tauri, título con dominio permitido, navegación HTTPS restringida, modo privado,
DevTools deshabilitados y ninguna capability local o remota. No hereda el conjunto
de comandos de `main` ni de `log-*`. No hay scripts propios inyectados en Microsoft,
lectura de cookies/contraseñas, opener arbitrario, iframe ni manipulación del formulario.
Se desactiva la inyección global de enlaces del plugin opener para que no intercepte
las páginas Microsoft. Las aperturas autorizadas se hacen desde Rust.

El panel local muestra carga/progreso, dominio, cancelación, alternativas y errores.
La ventana remota conserva sus controles nativos. Cerrar su ventana cancela la espera.
Si no puede crearse, no termina la carga inicial en 20 segundos, solicita un popup
no compatible o necesita un dominio fuera de la lista comprobada, se descarta ese
intento PKCE y se abre un intento nuevo en el navegador del sistema, con el mismo ID.
No se amplía la lista de dominios ni se autoriza navegación local para forzar MFA.
El navegador del sistema mantiene soporte de verificación y federación que una WebView
puede no tener. RFC 8252 recomienda el navegador del sistema para aplicaciones nativas.

Si el SO no puede abrir el navegador, se ofrece código de dispositivo. También se
puede elegir cualquier alternativa desde el panel. Esto es una alternativa de
transporte, nunca una sustitución silenciosa de la identidad OAuth.

El loopback solo escucha IPv4 127.0.0.1, con puerto temporal, tiempo y tamaños acotados.
Exige Host, ruta, método, estado y parámetros correctos. La respuesta es propia del
launcher, sin credenciales ni recursos remotos; un script fijo y autorizado por nonce
CSP únicamente sustituye la URL visible por `/complete` para quitar el código del
historial visible. No lee el código, cookies ni formularios. No es código inyectado
en la página de Microsoft. La página usa no-store y no-referrer.

## Cancelación y errores

Solo existe una operación OAuth activa por proceso. Cambiar de método cancela la
anterior; verificadores y estado no se reutilizan. Los intentos expiran en cinco
minutos; las solicitudes HTTP tienen timeout. Cancelar evita que una operación
pendiente agregue cuentas después. Al entrar en el guardado de credenciales comienza
una transacción no interrumpible: se muestra Guardando y el backend responde
AUTH_FINISHING a cancelaciones, evitando prometer una cancelación que luego guardaría.
Un prompt nativo del llavero debe resolverse en el sistema operativo.

Los errores distinguen Microsoft, Xbox Live, XSTS/familia, Minecraft/registro,
perfil y almacenamiento. No se devuelven cuerpos HTTP crudos ni URLs con parámetros.
La renovación no trata una sesión rechazada como si se hubiera refrescado. Solo
ante fallos transitorios de red/servicio puede usarse el token guardado si su
caducidad protegida demuestra que sigue vigente, con un aviso visible. No se usa
ese camino ante rechazo del registro, consentimiento, Xbox o sesión inválida.
Las sesiones heredadas sin caducidad conocida deben verificarse en línea. Las
cuentas offline siguen disponibles. Cerrar sesión elimina las credenciales locales
y su copia en memoria, conserva el perfil y selecciona una cuenta offline sin
repetir el onboarding. No revoca una sesión de Minecraft que ya esté ejecutándose.

## Reversión

Antes de activar el ID propio, la reversión es simplemente no habilitar la feature
de prueba; producción permanece igual. Después de una activación futura, restaurar
la configuración anterior en otro PR. Las sesiones emitidas para el ID nuevo no se
renuevan con el anterior: requieren nueva autorización. No borrar cuentas/instancias
para cambiar de identidad ni probar automáticamente diferentes registros.

El almacenamiento V2 no degrada al esquema antiguo. Revertir a un binario anterior
no le da acceso a los secretos V2; conserva perfiles/instancias, pero puede necesitar
nueva autorización. No regenerar el cifrado con UUID para simular compatibilidad.
