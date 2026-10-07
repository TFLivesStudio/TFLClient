pub mod account_policy;
pub mod bundled_resources;
pub mod errors;
pub mod event_bus;
pub mod http_client;
pub mod path_manager;
pub mod perf;

#[allow(unused_imports)]
pub use account_policy::allows_multiplayer;
#[allow(unused_imports)]
pub use bundled_resources::bundled_resource_dir;
#[allow(unused_imports)]
pub use errors::AppError;
#[allow(unused_imports)]
pub use event_bus::{AppEvent, emit};
#[allow(unused_imports)]
pub use http_client::{
    HTTP, get_bytes_retrying, get_json_retrying, get_text_retrying, post_json_retrying,
};
pub use path_manager::PathManager;

/// Azure AD (Entra ID) app registration usada para el login "Iniciar sesión
/// con Microsoft" (device code + Xbox Live + XSTS + Minecraft Services).
/// Hoy sigue siendo la de CubicLauncher — el proyecto del que TFL Client
/// viene (ver `launchwerk::auth::microsoft::DEFAULT_CLIENT_ID`) — por eso la
/// pantalla de "sesión iniciada" de Microsoft dice "CubicLauncher" en vez de
/// "TFL Client", y por qué depende de un registro ajeno (si ellos lo
/// revocan o cambian, el login de acá se rompe sin aviso).
///
/// Para que sea propio: registrar una app en portal.azure.com → Entra ID →
/// App registrations → New registration. Tipo "Public client/native",
/// sin necesidad de un plan pago (el nivel gratis de Entra ID alcanza).
/// Habilitar el flujo "Allow public client flows" (para device code) y
/// agregar el permiso delegado `XboxLive.signin` (API "Xbox Live",
/// requiere buscarla por App ID `00000000-0000-0000-0000-000000000000` o
/// agregarla manual si no aparece en la lista). Con eso ya alcanza — no
/// hace falta client secret (es un public client) ni verificación de
/// dominio. Una vez creada, reemplazar el valor de acá por el
/// "Application (client) ID" que Azure le asigna.
pub const TFL_MICROSOFT_CLIENT_ID: &str = launchwerk::auth::microsoft::DEFAULT_CLIENT_ID;
