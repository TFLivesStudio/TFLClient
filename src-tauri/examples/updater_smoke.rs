//! Native AppImage check for the release workflow. Mirrors only signed test bundles.

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::Path;
    use std::time::Duration;
    use tauri_plugin_updater::{Error, UpdaterExt};

    let args: Vec<String> = std::env::args().skip(1).collect();
    let [mode, endpoint, current, expected, profile] = args.as_slice() else {
        return Err(
            "Expected mode, endpoint, current version, expected version, and profile".into(),
        );
    };
    let endpoint = url::Url::parse(endpoint)?;
    if endpoint.scheme() != "http"
        || endpoint.host_str() != Some("127.0.0.1")
        || endpoint.port().is_none()
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
    {
        return Err("The smoke probe accepts only its loopback mirror".into());
    }
    let profile = Path::new(profile).canonicalize()?;
    let installed = profile.join("installed.AppImage");
    if !profile.join("TEST_PROFILE").is_file()
        || installed.symlink_metadata()?.file_type().is_symlink()
        || !installed.is_file()
    {
        return Err(
            "The smoke probe needs an isolated profile containing its AppImage copy".into(),
        );
    }
    if !matches!(mode.as_str(), "verify" | "reject" | "install") {
        return Err("Unknown smoke probe mode".into());
    }

    let mut context = tauri::generate_context!();
    context.config_mut().app.windows.clear();
    context.package_info_mut().version = current.parse()?;
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .build(context)?;
    let updater = app
        .updater_builder()
        .endpoints(vec![endpoint])?
        .executable_path(&installed)
        .no_proxy()
        .timeout(Duration::from_secs(60))
        .build()?;

    tauri::async_runtime::block_on(async {
        let update = updater
            .check()
            .await?
            .ok_or("The expected newer version was not offered")?;
        assert_eq!(&update.current_version, current);
        assert_eq!(&update.version, expected);
        let contents = update.download(|_, _| {}, || {}).await;
        if mode == "reject" {
            assert!(
                matches!(contents, Err(Error::Minisign(_))),
                "Modified bundle must fail signature verification"
            );
            println!("Tauri rejected the modified bundle for {}", update.version);
        } else {
            let contents = contents?;
            if mode == "install" {
                update.install(&contents)?;
                println!("Tauri installed {} over {}", update.version, current);
            } else {
                assert_eq!(contents, std::fs::read(&installed)?);
                println!(
                    "Tauri verified the published fixture for {}",
                    update.version
                );
            }
        }
        Ok(())
    })
}

#[cfg(not(target_os = "linux"))]
fn main() {}
