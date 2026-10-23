//! Authoritative, bounded operation history. Only typed public fields cross this boundary.
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock, mpsc},
    time::{Duration, Instant},
};

const HISTORY: usize = 100;
const ACTIVE: usize = 64;
const MAX_FILE: u64 = 256 * 1024;
const FLUSH_DELAY: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Ui,
    Mcp,
    System,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Queued,
    Running,
    Succeeded,
    Failed,
    CancelRequested,
    Cancelled,
    Interrupted,
}
impl Status {
    pub fn terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::Interrupted
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    #[serde(rename = "content.install")]
    ContentInstall,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    Mod,
    Shader,
    ResourcePack,
    Plugin,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Waiting,
    Resolving,
    Downloading,
    Installing,
    Finished,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskError {
    InstallFailed,
    Interrupted,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub current: u64,
    pub total: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub id: String,
    pub revision: u64,
    pub kind: Kind,
    pub source: Source,
    pub target: Target,
    pub instance_id: Option<String>,
    pub project_id: Option<String>,
    pub status: Status,
    pub stage: Stage,
    pub created_at: u64,
    pub updated_at: u64,
    pub finished_at: Option<u64>,
    pub progress: Option<Progress>,
    pub cancellable: bool,
    pub error: Option<TaskError>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub revision: u64,
    pub tasks: Vec<Task>,
    pub storage_warning: bool,
}
struct State {
    snapshot: Snapshot,
    last_progress: std::collections::HashMap<String, Instant>,
}
pub struct TaskHub {
    state: Mutex<State>,
    dirty: Option<mpsc::SyncSender<()>>,
}
static HUB: OnceLock<Arc<TaskHub>> = OnceLock::new();
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn empty() -> Snapshot {
    Snapshot {
        revision: 0,
        tasks: vec![],
        storage_warning: false,
    }
}

impl TaskHub {
    fn recover(raw: &[u8]) -> Snapshot {
        let mut snapshot = if raw.len() <= MAX_FILE as usize {
            serde_json::from_slice::<Snapshot>(raw).ok()
        } else {
            None
        };
        let Some(ref mut s) = snapshot else {
            return Snapshot {
                storage_warning: true,
                ..empty()
            };
        };
        let mut ids = std::collections::HashSet::new();
        if s.tasks.len() > HISTORY + ACTIVE
            || s.revision > (1u64 << 50)
            || s.tasks.iter().any(|t| {
                uuid::Uuid::parse_str(&t.id).is_err()
                    || !ids.insert(t.id.clone())
                    || t.revision > s.revision
                    || t.cancellable
                    || t.instance_id
                        .as_ref()
                        .is_some_and(|id| uuid::Uuid::parse_str(id).is_err())
                    || t.project_id.as_ref().is_some_and(|id| {
                        id.len() != 8 || !id.bytes().all(|b| b.is_ascii_alphanumeric())
                    })
                    || t.progress
                        .as_ref()
                        .is_some_and(|p| p.total == 0 || p.current > p.total)
            })
        {
            return Snapshot {
                storage_warning: true,
                ..empty()
            };
        }
        for t in &mut s.tasks {
            if !t.status.terminal() {
                s.revision += 1;
                t.revision = s.revision;
                t.status = Status::Interrupted;
                t.error = Some(TaskError::Interrupted);
                t.finished_at = Some(now());
                t.updated_at = now();
            }
        }
        s.tasks.sort_by_key(|t| std::cmp::Reverse(t.revision));
        s.tasks.truncate(HISTORY);
        snapshot.unwrap()
    }
    pub fn snapshot(&self) -> Snapshot {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .snapshot
            .clone()
    }
    fn notify(&self) {
        if let Some(tx) = &self.dirty {
            let _ = tx.try_send(());
        }
    }
    pub fn create(&self, source: Source, target: Target) -> Result<String, String> {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if s.snapshot
            .tasks
            .iter()
            .filter(|t| !t.status.terminal())
            .count()
            >= ACTIVE
        {
            return Err("ACTIVITY_CAPACITY".into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        s.snapshot.revision += 1;
        let revision = s.snapshot.revision;
        s.snapshot.tasks.insert(
            0,
            Task {
                id: id.clone(),
                revision,
                kind: Kind::ContentInstall,
                source,
                target,
                instance_id: None,
                project_id: None,
                status: Status::Queued,
                stage: Stage::Waiting,
                created_at: now(),
                updated_at: now(),
                finished_at: None,
                progress: None,
                cancellable: false,
                error: None,
            },
        );
        let mut terminal = 0;
        s.snapshot.tasks.retain(|t| {
            if t.status.terminal() {
                terminal += 1;
                terminal <= HISTORY
            } else {
                true
            }
        });
        drop(s);
        self.notify();
        Ok(id)
    }
    fn update(
        &self,
        id: &str,
        expected: Option<u64>,
        status: Status,
        stage: Stage,
        progress: Option<Progress>,
        error: Option<TaskError>,
    ) -> Result<(), String> {
        if progress
            .as_ref()
            .is_some_and(|p| p.total == 0 || p.current > p.total)
        {
            return Err("ACTIVITY_PROGRESS_INVALID".into());
        }
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        let i = s
            .snapshot
            .tasks
            .iter()
            .position(|t| t.id == id)
            .ok_or("ACTIVITY_NOT_FOUND")?;
        let t = &s.snapshot.tasks[i];
        if t.status.terminal() || expected.is_some_and(|r| r != t.revision) {
            return Err("ACTIVITY_STALE_UPDATE".into());
        }
        let valid = matches!(
            (t.status, status),
            (
                Status::Queued,
                Status::Running | Status::Failed | Status::Interrupted
            ) | (
                Status::Running,
                Status::Running | Status::Succeeded | Status::Failed | Status::Interrupted
            )
        );
        if !valid {
            return Err("ACTIVITY_TRANSITION_INVALID".into());
        }
        if t.status == status && t.stage == stage && t.progress == progress && t.error == error {
            return Ok(());
        }
        if progress.is_some()
            && s.last_progress
                .get(id)
                .is_some_and(|last| last.elapsed() < FLUSH_DELAY)
        {
            return Ok(());
        }
        if progress.is_some() {
            s.last_progress.insert(id.into(), Instant::now());
        }
        s.snapshot.revision += 1;
        let revision = s.snapshot.revision;
        let t = &mut s.snapshot.tasks[i];
        t.revision = revision;
        t.status = status;
        t.stage = stage;
        t.progress = progress;
        t.error = error;
        t.updated_at = now();
        if status.terminal() {
            t.finished_at = Some(now());
            s.last_progress.remove(id);
        }
        // Bound terminal history immediately, including completions of concurrent tasks.
        s.snapshot
            .tasks
            .sort_by_key(|t| std::cmp::Reverse(t.revision));
        let mut terminal = 0;
        s.snapshot.tasks.retain(|t| {
            if t.status.terminal() {
                terminal += 1;
                terminal <= HISTORY
            } else {
                true
            }
        });
        drop(s);
        self.notify();
        Ok(())
    }
    pub fn cancel(&self, id: &str) -> Result<(), String> {
        let s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        let t = s
            .snapshot
            .tasks
            .iter()
            .find(|t| t.id == id)
            .ok_or("ACTIVITY_NOT_FOUND")?;
        if t.status.terminal() {
            return Err("ACTIVITY_ALREADY_FINISHED".into());
        }
        Err("ACTIVITY_NOT_CANCELLABLE".into())
    }
}
fn persist(path: &std::path::Path, snapshot: &Snapshot) -> std::io::Result<()> {
    use std::io::Write;
    let parent = path.parent().ok_or(std::io::ErrorKind::InvalidInput)?;
    std::fs::create_dir_all(parent)?;
    let tmp = path.with_extension("pending");
    let result = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(&serde_json::to_vec(snapshot)?)?;
        f.sync_all()?;
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}
pub fn init() {
    let _ = hub();
}
pub fn hub() -> &'static Arc<TaskHub> {
    HUB.get_or_init(|| {
        let path: PathBuf = crate::core::PathManager::get()
            .get_settings_dir()
            .join("activities-v1.json");
        let snapshot = match std::fs::metadata(&path) {
            Ok(m) if m.len() <= MAX_FILE => std::fs::read(&path)
                .map(|v| TaskHub::recover(&v))
                .unwrap_or(Snapshot {
                    storage_warning: true,
                    ..empty()
                }),
            Ok(_) => Snapshot {
                storage_warning: true,
                ..empty()
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => empty(),
            Err(_) => Snapshot {
                storage_warning: true,
                ..empty()
            },
        };
        let (tx, rx) = mpsc::sync_channel(1);
        let hub = Arc::new(TaskHub {
            state: Mutex::new(State {
                snapshot,
                last_progress: Default::default(),
            }),
            dirty: Some(tx),
        });
        let weak = Arc::downgrade(&hub);
        std::thread::Builder::new()
            .name("activity-history".into())
            .spawn(move || {
                while rx.recv().is_ok() {
                    std::thread::sleep(FLUSH_DELAY);
                    while rx.try_recv().is_ok() {}
                    let Some(hub) = weak.upgrade() else {
                        break;
                    };
                    let snapshot = hub.snapshot();
                    // One invalidation per batch across ALL concurrent operations, before IO.
                    crate::core::event_bus::activity_changed();
                    let failed = persist(&path, &snapshot).is_err();
                    let mut s = hub.state.lock().unwrap_or_else(|p| p.into_inner());
                    if s.snapshot.storage_warning != failed {
                        s.snapshot.storage_warning = failed;
                        s.snapshot.revision += 1;
                        drop(s);
                        crate::core::event_bus::activity_changed();
                    }
                }
            })
            .expect("activity history worker");
        if let Some(tx) = &hub.dirty {
            let _ = tx.try_send(());
        }
        hub
    })
}

/// Drop also covers unwinding and dropped/aborted futures; no optimistic UI completion.
pub struct Operation {
    hub: Arc<TaskHub>,
    id: String,
    finished: bool,
}
impl Operation {
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Store only canonical identifiers, never user names, paths or arbitrary provider text.
    pub fn bind_target(&self, instance_id: &str, project_id: Option<&str>) {
        let instance_id = uuid::Uuid::parse_str(instance_id)
            .ok()
            .map(|id| id.to_string());
        let project_id = project_id
            .filter(|id| id.len() == 8 && id.bytes().all(|b| b.is_ascii_alphanumeric()))
            .map(str::to_owned);
        let mut s = self.hub.state.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(i) = s
            .snapshot
            .tasks
            .iter()
            .position(|t| t.id == self.id && !t.status.terminal())
        {
            s.snapshot.revision += 1;
            let revision = s.snapshot.revision;
            let task = &mut s.snapshot.tasks[i];
            task.instance_id = instance_id;
            task.project_id = project_id;
            task.revision = revision;
            task.updated_at = now();
            drop(s);
            self.hub.notify();
        }
    }
    pub fn content(source: Source, target: Target) -> Result<Self, String> {
        Self::with_hub(hub().clone(), source, target)
    }
    fn with_hub(hub: Arc<TaskHub>, source: Source, target: Target) -> Result<Self, String> {
        let id = hub.create(source, target)?;
        Ok(Self {
            hub,
            id,
            finished: false,
        })
    }
    pub fn stage(&self, stage: Stage) {
        let _ = self
            .hub
            .update(&self.id, None, Status::Running, stage, None, None);
    }
    pub fn bytes(&self, current: u64, total: Option<u64>) {
        let progress = total
            .filter(|t| *t > 0 && current <= *t)
            .map(|total| Progress { current, total });
        let _ = self.hub.update(
            &self.id,
            None,
            Status::Running,
            Stage::Downloading,
            progress,
            None,
        );
    }
    pub fn finish(mut self, succeeded: bool) {
        let _ = self.hub.update(
            &self.id,
            None,
            if succeeded {
                Status::Succeeded
            } else {
                Status::Failed
            },
            Stage::Finished,
            None,
            if succeeded {
                None
            } else {
                Some(TaskError::InstallFailed)
            },
        );
        self.finished = true;
    }
}
impl Drop for Operation {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.hub.update(
                &self.id,
                None,
                Status::Interrupted,
                Stage::Finished,
                None,
                Some(TaskError::Interrupted),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn isolated() -> Arc<TaskHub> {
        Arc::new(TaskHub {
            state: Mutex::new(State {
                snapshot: empty(),
                last_progress: Default::default(),
            }),
            dirty: None,
        })
    }
    #[test]
    fn transitions_revisions_and_nonfictional_cancel() {
        let h = isolated();
        let id = h.create(Source::Ui, Target::Mod).unwrap();
        assert!(
            h.update(&id, None, Status::Succeeded, Stage::Finished, None, None)
                .is_err()
        );
        let rev = h.snapshot().revision;
        assert_eq!(h.cancel(&id).unwrap_err(), "ACTIVITY_NOT_CANCELLABLE");
        assert_eq!(h.snapshot().revision, rev);
        h.update(
            &id,
            Some(rev),
            Status::Running,
            Stage::Resolving,
            None,
            None,
        )
        .unwrap();
        assert!(
            h.update(
                &id,
                Some(rev),
                Status::Running,
                Stage::Downloading,
                None,
                None
            )
            .is_err()
        );
        h.update(&id, None, Status::Succeeded, Stage::Finished, None, None)
            .unwrap();
        assert!(
            h.update(&id, None, Status::Running, Stage::Resolving, None, None)
                .is_err()
        );
    }
    #[test]
    fn concurrent_unique_bounded_and_abandoned_operations_close() {
        let h = isolated();
        let threads: Vec<_> = (0..16)
            .map(|_| {
                let h = h.clone();
                std::thread::spawn(move || {
                    for _ in 0..20 {
                        let op =
                            Operation::with_hub(h.clone(), Source::Mcp, Target::Shader).unwrap();
                        op.stage(Stage::Resolving);
                        op.finish(true);
                    }
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        let s = h.snapshot();
        assert_eq!(s.tasks.len(), HISTORY);
        assert_eq!(
            s.tasks
                .iter()
                .map(|t| &t.id)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            HISTORY
        );
        {
            let op = Operation::with_hub(h.clone(), Source::System, Target::Plugin).unwrap();
            op.stage(Stage::Resolving);
        }
        assert_eq!(h.snapshot().tasks[0].status, Status::Interrupted);
    }
    #[test]
    fn progress_recovery_corruption_and_whitelist() {
        let h = isolated();
        let op = Operation::with_hub(h.clone(), Source::Ui, Target::ResourcePack).unwrap();
        op.stage(Stage::Resolving);
        op.bytes(2, Some(10));
        assert_eq!(
            h.snapshot().tasks[0].progress,
            Some(Progress {
                current: 2,
                total: 10
            })
        );
        let raw = serde_json::to_vec(&h.snapshot()).unwrap();
        let recovered = TaskHub::recover(&raw);
        assert_eq!(recovered.tasks[0].status, Status::Interrupted);
        assert!(TaskHub::recover(b"broken").storage_warning);
        let mut v = serde_json::to_value(h.snapshot()).unwrap();
        v["tasks"][0]["access_token"] = "secret".into();
        assert!(
            TaskHub::recover(&serde_json::to_vec(&v).unwrap())
                .tasks
                .is_empty()
        );
        assert!(!String::from_utf8(raw).unwrap().contains("instance_name"));
        assert!(
            h.update(
                &op.id,
                None,
                Status::Running,
                Stage::Downloading,
                Some(Progress {
                    current: 11,
                    total: 10
                }),
                None
            )
            .is_err()
        );
    }
    #[test]
    fn persistence_roundtrip_and_unwinding() {
        let h = isolated();
        let h2 = h.clone();
        let _ = std::panic::catch_unwind(move || {
            let op = Operation::with_hub(h2, Source::Ui, Target::Mod).unwrap();
            op.stage(Stage::Resolving);
            panic!("fixture");
        });
        assert_eq!(h.snapshot().tasks[0].status, Status::Interrupted);
        let dir = std::env::temp_dir().join(format!("tfl-activity-{}", uuid::Uuid::new_v4()));
        let p = dir.join("history.json");
        persist(&p, &h.snapshot()).unwrap();
        assert_eq!(TaskHub::recover(&std::fs::read(&p).unwrap()).tasks.len(), 1);
        std::fs::remove_file(p).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn capacity_invalid_identifiers_and_write_failure_are_bounded() {
        let h = isolated();
        for _ in 0..ACTIVE {
            h.create(Source::System, Target::Mod).unwrap();
        }
        assert_eq!(
            h.create(Source::Ui, Target::Mod).unwrap_err(),
            "ACTIVITY_CAPACITY"
        );
        let h = isolated();
        let op = Operation::with_hub(h.clone(), Source::Ui, Target::Mod).unwrap();
        op.bind_target(
            "/sensitive/path?token=secret",
            Some("eyJhbGciOiJIUzI1NiJ9.secret"),
        );
        assert!(h.snapshot().tasks[0].instance_id.is_none());
        assert!(h.snapshot().tasks[0].project_id.is_none());
        let tmp = std::env::temp_dir().join(format!("tfl-activity-file-{}", uuid::Uuid::new_v4()));
        std::fs::write(&tmp, b"fixture").unwrap();
        assert!(persist(&tmp.join("history"), &h.snapshot()).is_err());
        std::fs::remove_file(tmp).unwrap();
        assert_eq!(h.snapshot().tasks.len(), 1);
    }
}
