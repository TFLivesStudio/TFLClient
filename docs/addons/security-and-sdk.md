# Frontera de confianza y futura API de extensiones

## Lo disponible en API 1.0.0

El paquete `.tfl` es entrada no confiable. Rust es la autoridad de instalación,
versiones y límites. Sólo entrega al frontend del anfitrión objetos validados y
PNG estáticos como data URLs. El anfitrión aplica un conjunto cerrado de colores
y renderiza capas con su código. Los datos nunca se convierten en componentes
Svelte, HTML, CSS libre, shaders, módulos, `eval`, DLLs ni procesos.

El IPC de administración pertenece a la ventana principal ya autorizada. El
addon carece de ventana y contexto JavaScript propios; una contribución no puede
invocar comandos ni obtener `app-commands-all`. No se amplían CSP, permisos de
ventanas o scopes de asset protocol. No se conceden cuentas, tokens, red, rutas
arbitrarias, portapapeles, ejecución ni acceso a otros paquetes. El perfil seguro
revoca todas las contribuciones activas y las lecturas de recursos, también si se
inicia con `TFL_ADDONS_SAFE_MODE=1`.

El modelo protege frente a un **archivo de addon malicioso**, archivos corruptos
y cierres durante instalación. Un usuario o proceso del sistema con acceso de
escritura al propio perfil sigue pudiendo modificar el programa o registro; no
se afirma aislamiento del perfil frente al mismo usuario del sistema. Los
symlinks observables en paquetes y generaciones se rechazan. El hash de paquete
permite detectar modificación accidental/maliciosa de ese archivo, pero no crea
un canal autenticado ni firma de editor.

| Amenaza                                            | Defensa implementada                                                                                                                            | Límite                                                                                                                    |
| -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| Zip Slip, enlaces, nombres ambiguos entre sistemas | Subconjunto ASCII cerrado, rutas declaradas, archivos regulares, colisiones insensibles a mayúsculas rechazadas                                 | No se importa un ZIP genérico; los autores deben seguir el formato.                                                       |
| ZIP bomb e imágenes enormes                        | Límites de paquete, índice previo a creación de ZipArchive, ratio/tamaño por archivo y total, lectura acotada, dimensiones antes de decodificar | No se promete tiempo constante para todos los PNG válidos.                                                                |
| JS/CSS/HTML privilegiado                           | Rechazo de campos, tipos y archivos fuera de esquema; colores puros y PNG decodificados                                                         | No hay extensiones de comportamiento en esta beta.                                                                        |
| Sustitución después de inspección                  | Relectura y comparación SHA-256 exacta                                                                                                          | El autor declarado no se verifica; actualizaciones requieren revisión humana.                                             |
| Instalación parcial o actualización fallida        | Generaciones inmutables, staging no activo, commit de registro atómico, backup y rollback                                                       | Durabilidad frente a pérdida física de energía depende del sistema de archivos; no se afirma haberla ensayado en tres SO. |
| Registro o paquete corrupto                        | Recuperación del registro en modo seguro; generación inválida omitida y error visible                                                           | No se recuperan mágicamente los bytes de un paquete perdido.                                                              |
| Desinstalación que borra datos ajenos              | IDs y digest cerrados, eliminación sólo en generaciones administradas, no seguimiento de enlaces                                                | Caches de recuperación pueden retener temporalmente paquetes desinstalados.                                               |

## Evolución del SDK: contrato, no funcionalidad disponible

La próxima API puede añadir contribuciones declarativas mediante una revisión de
esquema: localizaciones y slots concretos renderizados por el anfitrión, siempre
con texto y propiedades validadas. No se aceptarán nombres de componentes
arbitrarios ni acceso al árbol DOM del launcher. El versionado de API y de
esquema deberá separarse; una versión mayor cambiará contratos incompatibles y
exigirá una migración explícita con copia de estado.

Una extensión de comportamiento requiere un runtime con aislamiento real
demostrable en Windows, macOS y Linux. **Un proceso separado, una API de permisos,
un Web Worker o CSP no constituyen ese aislamiento.** Antes de activarlo habría
que elegir una tecnología soportada (por ejemplo un runtime WASM de capacidades
sin WASI ambiental), fijar el motor y probar escapes, imports, memoria, CPU,
interrupción y limpieza en las tres plataformas. No se incorpora ni anuncia ese
runtime aquí; no hay un permiso o botón de ejecutar código que aparente existir.

El host futuro deberá mediar cada capacidad a través de mensajes tipados y
versionados, con IDs de solicitud, cancelación, errores por extensión y cuotas.
Su ciclo de vida será instalar → inspeccionar → consentir → iniciar → suspender
→ revocar → detener → actualizar/restaurar. Fallar una extensión no podrá detener
el launcher. Revocar deberá cerrar handles, canales y trabajos pendientes, no
sólo ocultar su UI. Los permisos se pedirán por capacidad y ámbito, nunca como
acceso genérico al `invoke` del launcher.

Ejemplos de futuras capacidades **no disponibles**:

| Capacidad posible       | Mediación y consentimiento que necesitaría                                                                  |
| ----------------------- | ----------------------------------------------------------------------------------------------------------- |
| Datos privados acotados | Directorio virtual por ID; cuota; sin rutas del host; borrado explícito separado del paquete.               |
| Red                     | Destinos/protocolos concretos, sin credenciales ambientales; límites, cancelación y revocación por dominio. |
| Eventos de juego        | Eventos saneados y consentidos; sin cuentas, tokens ni classpath con credenciales.                          |
| Acciones de UI          | Slots y acciones enumerados por el host; sin DOM, script o IPC privilegiado.                                |

Firmas futuras deberán verificar una cadena de identidad de editor separada del
hash de contenido, manejar claves revocadas y rotación, y mostrar por separado
autor declarado, editor verificado y permisos consentidos. Los campos reservados
no se aceptan prematuramente en el esquema actual. Marketplace, cuentas,
publicación, pagos, distribución web y servicio de confianza quedan fuera de esta
entrega.

## Migración

Las preferencias existentes de tema, escenas, vídeos y plugins Minecraft
conservan su almacenamiento. El estado de Addons usa un registro nuevo separado.
Las selecciones calificadas de un addon ausente, desactivado, corrupto o
incompatible se resuelven con el fallback integrado del anfitrión. Un autor que
cambie su ID crea otro addon; para actualizar debe conservarlo e incrementar
SemVer. Los formatos que contengan código, SVG o CSS no se migran ejecutándolos:
deben convertirse en tokens y PNG compatibles o esperar una API futura revisada.
