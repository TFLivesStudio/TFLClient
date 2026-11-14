use super::*;

fn profile(loader: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tfl-selection-presentation-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("instance.tfl.json"),
        serde_json::to_vec(&serde_json::json!({"loader":loader})).unwrap(),
    )
    .unwrap();
    dir
}
fn lock(target: &str) -> SelectionLockfile {
    SelectionLockfile {
        schema_version: 1,
        pack_id: catalog::PACK_ID.into(),
        target_id: target.into(),
        channel: "beta".into(),
        release: "0.1.0-preview.1".into(),
        manifest_sha256: "a".repeat(64),
        installed_at: "1729000000".into(),
        managed_files: vec!["mods/official.jar".into()],
        seeded_files: vec!["options.txt".into()],
    }
}

#[test]
fn official_identity_is_loader_independent_and_normal_ornithe_stays_personal() {
    for loader in ["ornithe-gen2", "fabric", "forge", "neoforge"] {
        let dir = profile(loader);
        assert!(!is_selection_instance(&dir));
        assert!(identity_from_lock(&dir).is_none());
        let target = format!("mc-1.21.1-{loader}");
        write_lockfile(&dir, &lock(&target)).unwrap();
        assert!(is_selection_instance(&dir));
        assert_eq!(identity_from_lock(&dir).unwrap().target_id, target);
        assert!(ensure_personal_path(&dir, "mods/private.jar").is_err());
        assert!(ensure_personal_path(&dir, "config/../../mods/private.jar").is_err());
        for path in [
            "resourcepacks/personal.zip",
            "saves/My World/level.dat",
            "screenshots/a.png",
            "options.txt",
        ] {
            assert!(ensure_personal_path(&dir, path).is_ok());
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn corrupt_provenance_stays_protected_and_cannot_claim_valid_identity() {
    let dir = profile("fabric");
    std::fs::write(dir.join(LOCKFILE_NAME), b"broken").unwrap();
    assert!(is_selection_instance(&dir));
    assert!(identity_from_lock(&dir).is_none());
    assert!(ensure_personal_path(&dir, "mods/escape.jar").is_err());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn quarantine_preserves_personal_data_and_propagates_failure() {
    let dir = profile("fabric");
    std::fs::create_dir_all(dir.join("mods")).unwrap();
    std::fs::create_dir_all(dir.join("saves/My World")).unwrap();
    std::fs::write(dir.join("saves/My World/level.dat"), b"personal world").unwrap();
    std::fs::write(dir.join("options.txt"), b"personal options").unwrap();
    std::fs::write(dir.join("mods/official.jar"), b"pinned").unwrap();
    std::fs::write(dir.join("mods/personal.jar"), b"extra").unwrap();
    // A non-directory quarantine destination must fail rather than allow launch.
    std::fs::write(dir.join("quarantine"), b"blocked").unwrap();
    let managed = vec!["mods/official.jar".into()];
    assert!(quarantine_foreign_mods(&dir, &managed).is_err());
    assert!(dir.join("mods/personal.jar").is_file());
    std::fs::remove_file(dir.join("quarantine")).unwrap();
    assert_eq!(quarantine_foreign_mods(&dir, &managed).unwrap().len(), 1);
    std::fs::write(dir.join("mods/personal.jar"), b"second extra").unwrap();
    quarantine_foreign_mods(&dir, &managed).unwrap();
    assert_eq!(
        std::fs::read_dir(dir.join("quarantine")).unwrap().count(),
        2
    );
    assert_eq!(
        std::fs::read(dir.join("options.txt")).unwrap(),
        b"personal options"
    );
    assert_eq!(
        std::fs::read(dir.join("saves/My World/level.dat")).unwrap(),
        b"personal world"
    );
    assert_eq!(
        std::fs::read(dir.join("mods/official.jar")).unwrap(),
        b"pinned"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn optional_signed_icon_contract_is_bounded_and_legacy_manifest_remains_valid() {
    let catalog = catalog::parse_catalog(include_bytes!("testdata/catalog.json")).unwrap();
    let target = catalog
        .targets
        .iter()
        .find(|t| t.target_id == "mc-1.8.9-ornithe")
        .unwrap();
    let pointer = target.channels.get(Channel::Beta).unwrap();
    let expected = Expected { target, pointer };
    let mut m = manifest::load_manifest(
        include_bytes!("testdata/manifest-1.8.9.json"),
        include_bytes!("testdata/manifest-1.8.9.json.sig"),
        &expected,
        "0.15.4-beta.2",
        TRUSTED_KEYS,
    )
    .unwrap();
    assert!(m.icon_asset.is_none());
    assert!(m.capabilities.is_none());
    let hash = "b".repeat(64);
    m.icon_asset = Some(manifest::AssetRef {
        asset_name: format!("icon-{hash}.png"),
        sha256: hash,
        bytes: 1024,
    });
    m.capabilities = Some(manifest::ClientCapabilities { shaders: true });
    assert!(manifest::validate_manifest(&m, &expected, "0.15.4-beta.2").is_ok());
    m.icon_asset.as_mut().unwrap().bytes = 1024 * 1024 + 1;
    assert!(manifest::validate_manifest(&m, &expected, "0.15.4-beta.2").is_err());
    m.icon_asset.as_mut().unwrap().bytes = 1024;
    m.icon_asset.as_mut().unwrap().asset_name = "../icon.png".into();
    assert!(manifest::validate_manifest(&m, &expected, "0.15.4-beta.2").is_err());
    assert!(!valid_icon_png(b"not an image"));
}
