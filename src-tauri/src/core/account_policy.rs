//! El launcher permite intentar multijugador con cualquier tipo de cuenta.
//! El servidor conserva su autenticación y decide si acepta esa identidad.
use launchwerk::auth::AccountType;

pub fn allows_multiplayer(_user_type: AccountType) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_identity_can_attempt_a_server_connection() {
        for kind in [
            AccountType::Cracked,
            AccountType::Microsoft,
            AccountType::Yggdrasil,
        ] {
            assert!(allows_multiplayer(kind));
        }
    }
}
