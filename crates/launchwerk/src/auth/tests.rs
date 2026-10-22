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
