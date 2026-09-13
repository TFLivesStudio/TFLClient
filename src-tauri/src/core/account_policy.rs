//! Política de acceso multijugador por tipo de cuenta. Cuentas offline/cracked
//! solo pueden jugar en singleplayer; Microsoft y Yggdrasil tienen acceso
//! completo. Única fuente de verdad de esta regla.
use launchwerk::auth::AccountType;

pub fn allows_multiplayer(user_type: AccountType) -> bool {
    !matches!(user_type, AccountType::Cracked)
}
