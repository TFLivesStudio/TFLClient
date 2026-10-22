use crate::auth::MinecraftUser;

#[test]
fn test_token_serialization_skip() {
    let user = MinecraftUser::premium(
        "test_user".to_string(),
        "test_uuid".to_string(),
        "secret_token".to_string(),
        Some("refresh_token".to_string()),
    );

    let json = serde_json::to_string(&user).unwrap();

    // El JSON no debería contener los tokens
    assert!(!json.contains("secret_token"));
    assert!(!json.contains("refresh_token"));
    assert!(!json.contains("access_token"));

    // Al deserializar, los tokens deberían estar vacíos (o por defecto)
    let deserialized: MinecraftUser = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.access_token, "");
    assert_eq!(deserialized.refresh_token, None);
    assert_eq!(deserialized.username, "test_user");
    assert_eq!(deserialized.uuid, "test_uuid");
}

#[test]
fn tokens_are_redacted_in_debug_and_memory_only_is_explicit() {
    let mut user = MinecraftUser::premium(
        "name".into(),
        "id".into(),
        "secret-access".into(),
        Some("secret-refresh".into()),
    );
    assert!(!format!("{user:?}").contains("secret-"));
    user.session_only = true;
    user.save_tokens().unwrap();
    let mut reloaded: MinecraftUser =
        serde_json::from_str(&serde_json::to_string(&user).unwrap()).unwrap();
    assert!(reloaded.load_tokens().is_err());
    assert_eq!(reloaded.username, "name");
}

#[test]
fn changing_identity_requires_reauthorization_without_removing_profile() {
    let mut user = MinecraftUser::premium(
        "Original".into(),
        "id".into(),
        "fixture-access".into(),
        Some("fixture-refresh".into()),
    );
    user.microsoft_client_id = Some(crate::auth::microsoft::DEFAULT_CLIENT_ID.into());
    assert!(!user.requires_reauthorization(crate::auth::microsoft::DEFAULT_CLIENT_ID));
    assert!(user.requires_reauthorization("different-configured-registration"));
    assert_eq!(user.username, "Original");
    assert_eq!(user.uuid, "id");
    assert!(!MinecraftUser::cracked("Offline").requires_reauthorization("any-registration"));
}

#[test]
fn test_sessions_and_application_registrations_have_independent_credential_namespaces() {
    let mut user = MinecraftUser::premium(
        "Same account".into(),
        "same-uuid".into(),
        "fixture".into(),
        None,
    );
    let production = user.provider();
    assert!(user.may_load_legacy());
    user.test_session = true;
    assert_ne!(user.provider(), production);
    assert!(!user.may_load_legacy());
    assert!(
        !serde_json::to_string(&user)
            .unwrap()
            .contains("test_session")
    );
    user.test_session = false;
    user.microsoft_client_id = Some("another-registration-fixture".into());
    assert_ne!(user.provider(), production);
    assert!(!user.may_load_legacy());
}
