//! The OAuth identity is build configuration, never a remote/user preference.
use launchwerk::auth::microsoft::DEFAULT_CLIENT_ID;
use serde::Serialize;

/// Approved Microsoft Entra registration owned by TFL Client. This is a public
/// OAuth client ID, not a credential or a secret.
pub const TFL_CLIENT_OWN_CLIENT_ID: &str = "40fffa46-b69c-4ae1-9410-16f74d7847fd";

/// The owner confirmed Minecraft approval on 2026-11-10. Keep the selection at
/// build time: preferences, environment variables and remote configuration must
/// never be able to choose an OAuth identity.
pub const PRODUCTION_CLIENT_ID: &str = TFL_CLIENT_OWN_CLIENT_ID;

#[derive(Clone, Serialize)]
pub struct MicrosoftIdentity {
    pub client_id: String,
    pub test_build: bool,
    pub custom_identity: bool,
}
pub fn configured() -> Result<MicrosoftIdentity, String> {
    resolve(
        cfg!(feature = "auth-test-client"),
        option_env!("TFL_AUTH_TEST_CLIENT_ID"),
        option_env!("TFL_AUTH_TEST_ACK"),
    )
}
fn resolve(
    test: bool,
    supplied: Option<&str>,
    ack: Option<&str>,
) -> Result<MicrosoftIdentity, String> {
    let client_id = if test {
        if ack != Some("development-only-not-for-release") {
            return Err("AUTH_TEST_CONFIGURATION_REQUIRED".into());
        }
        let id = supplied.ok_or("AUTH_TEST_CONFIGURATION_REQUIRED")?;
        let parsed = uuid::Uuid::parse_str(id).map_err(|_| "AUTH_CLIENT_ID_INVALID")?;
        if parsed.is_nil() || id != parsed.hyphenated().to_string() {
            return Err("AUTH_CLIENT_ID_INVALID".into());
        }
        id
    } else {
        PRODUCTION_CLIENT_ID
    };
    Ok(MicrosoftIdentity {
        client_id: client_id.into(),
        test_build: test,
        custom_identity: client_id != DEFAULT_CLIENT_ID,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_ignores_environment_and_requires_no_secret() {
        assert_eq!(
            resolve(false, Some("untrusted"), None).unwrap().client_id,
            PRODUCTION_CLIENT_ID
        );
        assert!(resolve(true, None, None).is_err());
        assert!(
            resolve(
                true,
                Some("not-a-guid"),
                Some("development-only-not-for-release")
            )
            .is_err()
        );
        // Reuse an existing public ID in tests, never invent a supposed TFL registration.
        let test = resolve(
            true,
            Some(DEFAULT_CLIENT_ID),
            Some("development-only-not-for-release"),
        )
        .unwrap();
        assert!(test.test_build);
    }

    /// Production uses the approved TFL Client registration. The test lane stays
    /// isolated and requires its explicit development acknowledgement.
    #[test]
    fn production_uses_the_approved_tfl_client_registration() {
        assert_ne!(TFL_CLIENT_OWN_CLIENT_ID, DEFAULT_CLIENT_ID);

        let production = resolve(false, Some(TFL_CLIENT_OWN_CLIENT_ID), None).unwrap();
        assert_eq!(production.client_id, TFL_CLIENT_OWN_CLIENT_ID);
        assert!(!production.test_build);
        assert!(production.custom_identity);

        let test = resolve(
            true,
            Some(TFL_CLIENT_OWN_CLIENT_ID),
            Some("development-only-not-for-release"),
        )
        .unwrap();
        assert_eq!(test.client_id, TFL_CLIENT_OWN_CLIENT_ID);
        assert!(test.test_build);
        assert!(test.custom_identity);

        // Missing/wrong acknowledgement still blocks it even with the real ID supplied.
        assert!(resolve(true, Some(TFL_CLIENT_OWN_CLIENT_ID), None).is_err());
        assert!(resolve(true, Some(TFL_CLIENT_OWN_CLIENT_ID), Some("yes")).is_err());
    }
}

/// Never trust a frontend/profile value to select the credential lane.
pub(crate) fn scope_user(user: &mut launchwerk::auth::MinecraftUser) {
    user.test_session = cfg!(feature = "auth-test-client");
}

#[cfg(any(feature = "auth-test-client", test))]
pub(crate) fn isolate_test_config(
    config: &mut tauri::utils::config::Config,
    root: &std::path::Path,
    client: &str,
) {
    config.identifier = format!(
        "{}.auth-test-{}",
        config.identifier,
        client.replace('-', "")
    );
    for window in &mut config.app.windows {
        window.incognito = true;
        window.data_directory = Some(root.join("webview"));
        window.title = "TFL Client — Microsoft Auth TEST".into();
    }
    config.app.security.asset_protocol.scope = serde_json::from_value(serde_json::json!({
        "allow": [root.join("instances/*/icon.*"), root.join("instances/*/screenshots/*"), root.join("shared/appearance/*"), root.join("shared/appearance/scenes/*")],
        "deny": [root.join("settings/**")]
    })).expect("static isolated test asset scope");
}
#[cfg(test)]
mod isolation_tests {
    use super::*;
    #[test]
    fn test_window_uses_a_private_store_and_separate_application_identity() {
        let mut config: tauri::utils::config::Config = serde_json::from_value(serde_json::json!({"identifier":"com.tflives.tflclient","app":{"windows":[{"label":"main"}]}})).unwrap();
        let root = std::env::temp_dir().join("isolated-auth-profile");
        isolate_test_config(&mut config, &root, DEFAULT_CLIENT_ID);
        assert_ne!(config.identifier, "com.tflives.tflclient");
        assert!(config.app.windows[0].incognito);
        assert_eq!(
            config.app.windows[0].data_directory.as_ref(),
            Some(&root.join("webview"))
        );
    }

    #[test]
    fn own_registration_gets_its_own_isolated_identifier_and_profile_root() {
        let mut config: tauri::utils::config::Config = serde_json::from_value(
            serde_json::json!({"identifier":"com.tflives.tflclient","app":{"windows":[{"label":"main"}]}}),
        )
        .unwrap();
        let root = std::env::temp_dir().join("isolated-auth-profile-own-id");
        isolate_test_config(&mut config, &root, TFL_CLIENT_OWN_CLIENT_ID);
        assert_ne!(config.identifier, "com.tflives.tflclient");
        assert!(
            config
                .identifier
                .contains("40fffa46b69c4ae1941016f74d7847fd")
        );
        assert!(config.app.windows[0].incognito);
        assert_eq!(
            config.app.windows[0].data_directory.as_ref(),
            Some(&root.join("webview"))
        );
    }
}
