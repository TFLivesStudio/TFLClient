//! MCP adapters reuse existing services; no JavaScript, shell or generic file API.
use super::{
    SettingsManager,
    content::{
        self,
        providers::{ContentProvider, ProjectType, modrinth::ModrinthProvider},
        types::ContentKind,
    },
    instance_manager::{self, InstanceData},
    mcp, mcp_preferences,
};
use crate::{commands, core::PathManager};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
fn text<'a>(p: &'a Value, k: &str) -> Result<&'a str, String> {
    p[k].as_str().ok_or_else(|| format!("MISSING_{k}"))
}
fn simple(s: &str) -> Result<(), String> {
    if s.is_empty()
        || s.len() > 200
        || s == "."
        || s == ".."
        || s.chars()
            .any(|c| c.is_control() || "/\\:*?\"<>|".contains(c))
    {
        Err("INVALID_IDENTIFIER".into())
    } else {
        Ok(())
    }
}
fn kind(p: &Value) -> Result<ContentKind, String> {
    match text(p, "kind")? {
        "mod" => Ok(ContentKind::Mod),
        "shader" => Ok(ContentKind::Shader),
        "resourcepack" => Ok(ContentKind::ResourcePack),
        "plugin" => Ok(ContentKind::Plugin),
        _ => Err("INVALID_CONTENT_KIND".into()),
    }
}
fn loader(i: &InstanceData) -> String {
    i.server_type
        .map(|s| {
            match s {
                instance_manager::ServerType::Vanilla => "vanilla",
                instance_manager::ServerType::Paper => "paper",
                instance_manager::ServerType::Purpur => "purpur",
            }
            .to_string()
        })
        .unwrap_or_else(|| format!("{:?}", i.loader).to_lowercase())
}
/// Refuse symlinks at all levels inside a managed instance, not just its root.
/// This intentionally fails closed for linked worlds/content; UI remains available.
pub fn safe_tree(root: &std::path::Path) -> Result<(), String> {
    let mut todo = vec![root.to_path_buf()];
    let mut visited = 0;
    while let Some(p) = todo.pop() {
        visited += 1;
        if visited > 50000 {
            return Err("INSTANCE_TOO_LARGE_FOR_SAFE_MCP; use launcher UI".into());
        }
        let m = std::fs::symlink_metadata(&p).map_err(|_| "MANAGED_PATH_UNAVAILABLE")?;
        if m.file_type().is_symlink() {
            return Err("SYMLINK_NOT_ALLOWED".into());
        }
        if m.is_dir() {
            for e in std::fs::read_dir(p).map_err(|_| "MANAGED_PATH_UNAVAILABLE")? {
                todo.push(e.map_err(|_| "MANAGED_PATH_UNAVAILABLE")?.path());
            }
        }
    }
    Ok(())
}
pub async fn instance(p: &Value) -> Result<InstanceData, String> {
    let id = text(p, "instance_id")?;
    uuid::Uuid::parse_str(id).map_err(|_| "INVALID_INSTANCE_UUID")?;
    let mut matches = instance_manager::list_instances()
        .await
        .into_iter()
        .filter(|i| i.uuid == id)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err("INSTANCE_NOT_FOUND_OR_AMBIGUOUS".into());
    }
    let i = matches.remove(0);
    simple(&i.name)?;
    let root = PathManager::get().get_instance_dir();
    if std::fs::symlink_metadata(root)
        .map_err(|_| "MANAGED_PATH_UNAVAILABLE")?
        .file_type()
        .is_symlink()
    {
        return Err("SYMLINK_NOT_ALLOWED".into());
    }
    let manifest = i.dir().join("instance.tfl.json");
    if std::fs::symlink_metadata(&manifest)
        .map_err(|_| "MANAGED_PATH_UNAVAILABLE")?
        .file_type()
        .is_symlink()
    {
        return Err("SYMLINK_NOT_ALLOWED".into());
    }
    if !i
        .mc_version
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err("INVALID_MC_VERSION".into());
    }
    let real = i
        .dir()
        .canonicalize()
        .map_err(|_| "MANAGED_PATH_UNAVAILABLE")?;
    if real.parent()
        != Some(
            root.canonicalize()
                .map_err(|_| "MANAGED_PATH_UNAVAILABLE")?
                .as_path(),
        )
    {
        return Err("PATH_OUTSIDE_SANDBOX".into());
    }
    Ok(i)
}
fn running(i: &InstanceData) -> bool {
    super::launcher::running_instance_pid(&i.name).is_some()
        || super::server_process::is_server_running(&i.name)
}
fn stopped(i: &InstanceData) -> Result<(), String> {
    if running(i) {
        Err("INSTANCE_RUNNING; stop with local approval before changing files".into())
    } else {
        Ok(())
    }
}
/// Relevant managed state fingerprint includes every relative entry, length and mtime.
/// Revalidated immediately under MCP's mutation lock, and running state is included.
pub async fn fingerprint(i: &InstanceData, p: &Value) -> Result<String, String> {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    i.uuid.hash(&mut h);
    serde_json::to_string(i)
        .map_err(|_| "Manifest error")?
        .hash(&mut h);
    running(i).hash(&mut h);
    super::launcher::running_instance_pid(&i.name).hash(&mut h);
    super::server_process::running_server_pid(&i.name).hash(&mut h);
    p.to_string().hash(&mut h);
    if p["action"] == "stop_instance" {
        return Ok(format!("{:016x}", h.finish()));
    }
    safe_tree(&i.dir())?;
    let root = i.dir();
    let mut todo = vec![root.clone()];
    while let Some(dir) = todo.pop() {
        let mut entries = std::fs::read_dir(&dir)
            .map_err(|_| "Fingerprint unavailable")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Fingerprint unavailable")?;
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let m = e.metadata().map_err(|_| "Fingerprint unavailable")?;
            e.path()
                .strip_prefix(&root)
                .map_err(|_| "Invalid path")?
                .hash(&mut h);
            m.len().hash(&mut h);
            m.modified().ok().hash(&mut h);
            if m.is_dir() {
                todo.push(e.path());
            }
        }
    }
    Ok(format!("{:016x}", h.finish()))
}
pub fn redact(s: &str) -> String {
    static SECRETS: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(?i)(access.?token|refresh.?token|authorization|bearer\s|password|passwd|cookie|client.?secret|sessionid|--uuid|--username|xbox|xsts|oauth)").unwrap()
    });
    let home = directories::UserDirs::new()
        .map(|u| u.home_dir().to_string_lossy().to_string())
        .unwrap_or_default();
    s.lines()
        .take(400)
        .map(|line| {
            if SECRETS.is_match(line) {
                "[sensitive line redacted]".into()
            } else {
                let l = line.chars().take(2048).collect::<String>();
                if home.is_empty() {
                    l
                } else {
                    l.replace(&home, "[home]")
                }
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
pub fn redact_value(v: Value) -> Value {
    match v {
        Value::String(s) => json!(redact(&s)),
        Value::Array(a) => Value::Array(a.into_iter().map(redact_value).collect()),
        Value::Object(m) => Value::Object(
            m.into_iter()
                .filter(|(k, _)| {
                    let normalized = k.to_ascii_lowercase();
                    if ["token", "password", "secret", "cookie", "authorization"]
                        .iter()
                        .any(|word| normalized.contains(word))
                    {
                        return false;
                    }
                    ![
                        "access_token",
                        "refresh_token",
                        "account_uuid",
                        "user",
                        "env_vars",
                        "jvm_args",
                        "path",
                    ]
                    .contains(&k.as_str())
                })
                .map(|(k, v)| (k, redact_value(v)))
                .collect(),
        ),
        v => v,
    }
}
async fn logs(i: &InstanceData, p: &Value) -> Result<Value, String> {
    let path = i.dir().join("logs/latest.log");
    if !path.exists() {
        return Ok(json!({"lines":[],"source":"latest.log","available":false}));
    }
    let mut f = tokio::fs::File::open(path)
        .await
        .map_err(|_| "LOG_UNAVAILABLE")?;
    let size = f.metadata().await.map_err(|_| "LOG_UNAVAILABLE")?.len();
    f.seek(std::io::SeekFrom::Start(size.saturating_sub(256 * 1024)))
        .await
        .map_err(|_| "LOG_UNAVAILABLE")?;
    let mut bytes = Vec::new();
    f.take(256 * 1024)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| "LOG_UNAVAILABLE")?;
    let raw = String::from_utf8_lossy(&bytes);
    let offset = p["offset"].as_u64().unwrap_or(0) as usize;
    let limit = p["limit"].as_u64().unwrap_or(100) as usize;
    let level = p["level"].as_str().unwrap_or("all");
    let lines = raw
        .lines()
        .skip(usize::from(size > 256 * 1024))
        .filter(|l| match level {
            "warn" => l.contains("WARN"),
            "error" => l.contains("ERROR") || l.contains("FATAL"),
            _ => true,
        })
        .map(redact)
        .collect::<Vec<_>>();
    let total = lines.len();
    Ok(
        json!({"lines":lines.into_iter().rev().skip(offset).take(limit).collect::<Vec<_>>(),"order":"newest_first","offset":offset,"total_in_tail":total,"truncated":size>256*1024,"source":"latest.log","untrusted_data":true}),
    )
}
fn summary(i: &InstanceData) -> Value {
    json!({"instance_id":i.uuid,"name":i.name,"mc_version":i.mc_version,"loader":i.loader,"server_type":i.server_type,"min_memory_mb":i.min_memory,"max_memory_mb":i.max_memory,"running":running(i)})
}
pub async fn execute(app: &AppHandle, client: &str, tool: &str, p: Value) -> Result<Value, String> {
    if p["instance_id"].is_string()
        && ![
            "tfl_get_instance",
            "tfl_get_process_stats",
            "tfl_launch_instance",
            "tfl_search_content",
            "tfl_get_content_versions",
        ]
        .contains(&tool)
    {
        let i = instance(&p).await?;
        let dir = i.dir();
        tokio::task::spawn_blocking(move || safe_tree(&dir))
            .await
            .map_err(|_| "Sandbox worker failed")??;
    }
    match tool {
        "tfl_get_app_info" => Ok(
            json!({"version":env!("CARGO_PKG_VERSION"),"platform":std::env::consts::OS,"arch":std::env::consts::ARCH,"contract":1,"transport":"stdio/private_authenticated_ipc","tools":tfl_mcp::catalog::definitions().iter().map(|d|d.name).collect::<Vec<_>>(),"http":false,"remote":false,"session_id":client}),
        ),
        "tfl_get_system_summary" => {
            let r = commands::settings::get_recommended_ram();
            Ok(
                json!({"os":std::env::consts::OS,"total_ram_mib":r.total_mb,"recommended_min_mib":r.recommended_min_mb,"recommended_max_mib":r.recommended_max_mb}),
            )
        }
        "tfl_list_instances" => {
            let q = p["query"].as_str().unwrap_or("").to_lowercase();
            let mut list = instance_manager::list_instances().await;
            list.retain(|i| {
                mcp::STATE.lock().scope(client, &i.uuid) && i.name.to_lowercase().contains(&q)
            });
            list.sort_by(|a, b| a.uuid.cmp(&b.uuid));
            let total = list.len();
            let offset = p["offset"].as_u64().unwrap_or(0) as usize;
            Ok(
                json!({"total":total,"offset":offset,"instances":list.iter().skip(offset).take(p["limit"].as_u64().unwrap_or(50) as usize).map(summary).collect::<Vec<_>>()}),
            )
        }
        "tfl_get_instance" => Ok(summary(&instance(&p).await?)),
        "tfl_get_running_sessions" => {
            let list = instance_manager::list_instances().await;
            Ok(json!(
                list.iter()
                    .filter(|i| running(i) && mcp::STATE.lock().scope(client, &i.uuid))
                    .map(summary)
                    .take(100)
                    .collect::<Vec<_>>()
            ))
        }
        "tfl_get_process_stats" => {
            let i = instance(&p).await?;
            let s = if i.server_type.is_some() {
                commands::instance::get_running_server_stats(i.name)
            } else {
                commands::instance::get_running_instance_stats(i.name)
            };
            Ok(
                json!({"stats":s,"cpu_unit":"percent_of_one_logical_core","ram_unit":"MiB_resident","source":"shared_sysinfo_sampler","available":s.is_some()}),
            )
        }
        "tfl_get_java_status" => Ok(json!(
            super::java_manager::status()
                .into_iter()
                .map(|j| json!({"major":j.major,"installed":j.installed}))
                .collect::<Vec<_>>()
        )),
        "tfl_get_instance_logs" => logs(&instance(&p).await?, &p).await,
        "tfl_get_crash_report" => {
            let i = instance(&p).await?;
            Ok(
                json!({"report":commands::instance::get_crash_report(i.name).await?,"source":"native_jvm_crash","untrusted_data":true}),
            )
        }
        "tfl_diagnose_instance" => {
            let i = instance(&p).await?;
            let evidence = logs(&i, &json!({"limit":200,"level":"error"})).await?;
            Ok(
                json!({"instance_id":i.uuid,"evidence":evidence,"rules":"error/fatal log lines only","cause":"undetermined; evidence does not prove a root cause"}),
            )
        }
        "tfl_create_instance" => {
            let name = text(&p, "name")?;
            simple(name)?;
            let version = text(&p, "mc_version")?;
            if !version
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            {
                return Err("INVALID_MC_VERSION".into());
            }
            let d = super::instance_manager::create_resolved(
                name.into(),
                version.into(),
                text(&p, "loader")?.into(),
            )
            .await?;
            Ok(summary(&d))
        }
        "tfl_duplicate_instance" => {
            let i = instance(&p).await?;
            stopped(&i)?;
            if !mcp::STATE
                .lock()
                .clients
                .iter()
                .find(|c| c.id == client)
                .is_some_and(|c| c.instances.is_empty())
            {
                return Err("INSTANCE_SCOPE_DENIED: duplication creates a new UUID".into());
            }
            Ok(summary(
                &instance_manager::duplicate_instance(&i.name).await?,
            ))
        }
        "tfl_set_instance_scene" => {
            let i = instance(&p).await?;
            let preset = text(&p, "preset")?;
            let current = mcp_preferences::snapshot();
            let mut doc = current
                .values
                .get("scenes")
                .cloned()
                .unwrap_or_else(|| json!({"version":1,"global":{},"instances":{}}));
            if !doc["instances"].is_object() {
                doc["instances"] = json!({});
            }
            if preset == "inherit" {
                doc["instances"].as_object_mut().unwrap().remove(&i.uuid);
            } else {
                let mut config = doc["instances"][&i.uuid]
                    .as_object()
                    .cloned()
                    .unwrap_or_else(|| doc["global"].as_object().cloned().unwrap_or_default());
                config.insert("preset".into(), json!(preset));
                config.insert("mediaId".into(), Value::Null);
                doc["instances"][&i.uuid] = Value::Object(config);
            }
            let next = mcp_preferences::write(
                "scenes",
                doc,
                p["expected_revision"].as_u64().unwrap() as u32,
            )?;
            let _ = app.emit("mcp-preferences-changed", &next);
            Ok(json!({"revision":next.revision,"instance_id":i.uuid,"preset":preset}))
        }
        "tfl_configure_console" => {
            let current = mcp_preferences::snapshot();
            let mut doc = current
                .values
                .get("console")
                .cloned()
                .unwrap_or_else(|| json!({"version":1}));
            let metric = text(&p, "metric")?;
            for (input, field) in [
                ("style", "metricStyles"),
                ("seconds", "metricSeconds"),
                ("visible", "visible"),
                ("color", "colors"),
            ] {
                if !p[input].is_null() {
                    if input == "seconds" && !matches!(p[input].as_u64(), Some(30 | 60 | 180)) {
                        return Err("INVALID_CONSOLE_WINDOW".into());
                    }
                    if input == "color" {
                        mcp_preferences::validate("accent", &p[input])?;
                    }
                    if !doc[field].is_object() {
                        doc[field] = json!({});
                    }
                    doc[field][metric] = p[input].clone();
                }
            }
            if !p["dock"].is_null() {
                doc["dock"] = p["dock"].clone();
            }
            let next = mcp_preferences::write(
                "console",
                doc,
                p["expected_revision"].as_u64().unwrap() as u32,
            )?;
            let _ = app.emit("mcp-preferences-changed", &next);
            Ok(json!({"revision":next.revision,"console":next.values.get("console")}))
        }
        "tfl_rename_instance" => {
            let i = instance(&p).await?;
            stopped(&i)?;
            let name = text(&p, "name")?;
            simple(name)?;
            Ok(summary(
                &instance_manager::rename_instance(&i.name, name.into()).await?,
            ))
        }
        "tfl_set_instance_memory" => {
            let i = instance(&p).await?;
            stopped(&i)?;
            let min = p["min_mb"].as_u64().unwrap() as u32;
            let max = p["max_mb"].as_u64().unwrap() as u32;
            if min > max {
                return Err("MIN_EXCEEDS_MAX".into());
            }
            let total = commands::settings::get_recommended_ram().total_mb;
            if max > total.saturating_sub(512).max(512) {
                return Err("RAM_EXCEEDS_AVAILABLE_BUDGET".into());
            }
            Ok(summary(
                &instance_manager::update_instance_memory(&i.name, Some(min), Some(max)).await?,
            ))
        }
        "tfl_launch_instance" => {
            let i = instance(&p).await?;
            stopped(&i)?;
            if i.server_type.is_some() {
                super::server_process::launch_server(i.name).await?;
            } else {
                super::launcher::launch(app.clone(), i.name, None).await?;
            }
            Ok(json!({"launched":true}))
        }
        "tfl_list_worlds" => {
            let i = instance(&p).await?;
            let folder = i.dir().join(if i.server_type.is_some() {
                "."
            } else {
                "saves"
            });
            let mut names = Vec::new();
            if let Ok(mut entries) = tokio::fs::read_dir(folder).await {
                while let Ok(Some(e)) = entries.next_entry().await {
                    if e.path().join("level.dat").is_file() {
                        names.push(e.file_name().to_string_lossy().to_string());
                    }
                    if names.len() >= 100 {
                        break;
                    }
                }
            }
            Ok(json!({"worlds":names}))
        }
        "tfl_list_backups" => {
            let i = instance(&p).await?;
            Ok(json!(
                commands::world_backups::get_world_backups(i.name)
                    .await?
                    .into_iter()
                    .take(100)
                    .collect::<Vec<_>>()
            ))
        }
        "tfl_create_backup" => {
            let i = instance(&p).await?;
            stopped(&i)?;
            if i.server_type.is_some() {
                return Err("SERVER_BACKUP_NOT_SUPPORTED_BY_EXISTING_SERVICE".into());
            }
            Ok(json!({"backup":commands::world_backups::create_world_backup(i.name).await?}))
        }
        "tfl_search_content" => {
            let i = instance(&p).await?;
            let k = text(&p, "kind")?;
            let pt = match k {
                "modpack" => ProjectType::Modpack,
                _ => kind(&p)?.as_project_type(),
            };
            let l = loader(&i);
            let page = content::service()
                .search(content::search::SearchRequest {
                    query: p["query"].as_str().unwrap_or(""),
                    mc_version: &i.mc_version,
                    loader: &l,
                    categories: &[],
                    project_type: pt,
                    offset: p["offset"].as_u64().unwrap_or(0) as u32,
                    limit: p["limit"].as_u64().unwrap_or(20) as u32,
                    index: content::types::SearchIndex::Relevance,
                })
                .await?;
            Ok(json!({"page":page,"untrusted_data":true}))
        }
        "tfl_get_content_project" => {
            let id = text(&p, "project_id")?;
            simple(id)?;
            let mut v = json!(content::service().project_detail(id).await?);
            if let Some(o) = v.as_object_mut() {
                o.remove("body");
                o.remove("description");
            }
            Ok(json!({"project":v,"untrusted_data":true}))
        }
        "tfl_get_content_versions" => {
            let i = instance(&p).await?;
            let id = text(&p, "project_id")?;
            simple(id)?;
            Ok(json!(
                content::service()
                    .version_summaries(kind(&p)?, id, &i.mc_version, &loader(&i))
                    .await?
                    .into_iter()
                    .take(100)
                    .collect::<Vec<_>>()
            ))
        }
        "tfl_list_installed_content" => {
            let i = instance(&p).await?;
            Ok(
                json!({"files":content::filesystem::list_files(&i.name,kind(&p)?).await?.into_iter().skip(p["offset"].as_u64().unwrap_or(0) as usize).take(100).collect::<Vec<_>>()}),
            )
        }
        "tfl_install_content" => {
            let i = instance(&p).await?;
            stopped(&i)?;
            let k = kind(&p)?;
            let requested_project = text(&p, "project_id")?;
            simple(requested_project)?;
            let project_detail = content::service().project_detail(requested_project).await?;
            let project = project_detail.project_id.as_str();
            let version = text(&p, "version_id")?;
            simple(project)?;
            simple(version)?;
            let l = loader(&i);
            let choices = content::service()
                .version_summaries(k, project, &i.mc_version, &l)
                .await?;
            if !choices.iter().any(|v| v.id == version) {
                return Err("INCOMPATIBLE_CONTENT_VERSION".into());
            }
            let actual = ModrinthProvider.get_version(version).await?;
            if actual.project_id != project {
                return Err("PROJECT_VERSION_MISMATCH".into());
            }
            if (k == ContentKind::Plugin) != i.server_type.is_some() {
                return Err("CONTENT_TARGET_MISMATCH".into());
            }
            content::service()
                .install(content::install::InstallRequest {
                    instance_name: &i.name,
                    mc_version: &i.mc_version,
                    loader: &l,
                    project_id: project,
                    version_id: Some(version),
                    kind: k,
                })
                .await?;
            Ok(json!({"installed":true,"project_id":project,"version_id":version}))
        }
        "tfl_set_content_enabled" => {
            let i = instance(&p).await?;
            stopped(&i)?;
            let f = text(&p, "filename")?;
            simple(f)?;
            content::filesystem::set_enabled(
                &i.name,
                kind(&p)?,
                f,
                p["enabled"].as_bool().unwrap(),
            )
            .await?;
            Ok(json!({"updated":true}))
        }
        "tfl_check_content_updates" => {
            let i = instance(&p).await?;
            Ok(json!(
                content::service()
                    .check_updates(&i.name)
                    .await?
                    .into_iter()
                    .take(100)
                    .collect::<Vec<_>>()
            ))
        }
        "tfl_get_rollback_info" => {
            let i = instance(&p).await?;
            Ok(json!(content::rollback::rollback_info(&i.name).await?))
        }
        "tfl_get_preferences" => Ok(mcp_preferences::public()),
        "tfl_set_preference" => {
            let key = text(&p, "key")?;
            let value = p["value"].clone();
            let rev = p["expected_revision"].as_u64().unwrap() as u32;
            mcp_preferences::validate(key, &value)?;
            let next = mcp_preferences::write(key, value.clone(), rev)?;
            if key == "theme" {
                SettingsManager::write(|s| s.theme = value.as_str().unwrap().into())
                    .map_err(|_| "Settings error")?;
                SettingsManager::save().await?;
            }
            if key == "quality" {
                let q = serde_json::from_value(value).map_err(|_| "Quality error")?;
                commands::settings::set_quality_profile(q).await?;
            }
            let _ = app.emit("mcp-preferences-changed", &next);
            Ok(mcp_preferences::public())
        }
        "tfl_execute_plan" => {
            let plan_id = text(&p, "plan_id")?;
            let plan = {
                let s = mcp::STATE.lock();
                s.plans
                    .iter()
                    .find(|p| p.id == plan_id && p.client_id == client)
                    .cloned()
                    .ok_or("PLAN_NOT_FOUND")?
            };
            let i = instance(&plan.params).await?;
            let dir = i.dir();
            tokio::task::spawn_blocking(move || safe_tree(&dir))
                .await
                .map_err(|_| "Sandbox worker failed")??;
            let fp = fingerprint(&i, &plan.params).await?;
            let plan = mcp::STATE.lock().consume(client, plan_id, &fp)?;
            let action = text(&plan.params, "action")?;
            if action != "stop_instance" {
                stopped(&i)?;
            }
            let resource = plan.params["resource_id"].as_str().unwrap_or("");
            match action {
                "delete_instance" => instance_manager::delete_instance(&i.name).await?,
                "stop_instance" => {
                    if i.server_type.is_some() {
                        super::server_process::stop_server(i.name.clone()).await?
                    } else {
                        super::launcher::stop_running(&i.name).await?
                    }
                }
                "restore_backup" => {
                    simple(resource)?;
                    commands::world_backups::restore_world_backup(i.name.clone(), resource.into())
                        .await?;
                }
                "delete_backup" => {
                    simple(resource)?;
                    commands::world_backups::delete_world_backup(i.name.clone(), resource.into())
                        .await?;
                }
                "remove_content" => {
                    simple(resource)?;
                    content::filesystem::remove_file(&i.name, kind(&plan.params)?, resource)
                        .await?;
                }
                "rollback_mods" => {
                    content::rollback::rollback_update(&i.name).await?;
                }
                _ => return Err("UNSUPPORTED_ACTION".into()),
            }
            Ok(json!({"action":action,"instance_id":i.uuid,"completed":true}))
        }
        _ => Err("UNSUPPORTED_TOOL".into()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn traversal_and_secrets() {
        for s in ["..", "a/b", "a\\b", "x\n"] {
            assert!(simple(s).is_err());
        }
        let r = redact("ok\nAuthorization: Bearer secret\nrefresh_token=secret\nnormal");
        assert!(!r.contains("secret"));
        assert!(r.contains("normal"));
        assert!(
            !redact_value(json!({"access_token":"secret","path":"private","ok":3}))
                .to_string()
                .contains("secret")
        );
    }
    #[test]
    fn symlink_tree_is_denied() {
        let dir = std::env::temp_dir().join(format!("mcp-sandbox-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("safe"), b"ok").unwrap();
        assert!(safe_tree(&dir).is_ok());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/tmp", dir.join("escape")).unwrap();
            assert!(safe_tree(&dir).is_err());
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}
