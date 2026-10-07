//! Medición de rendimiento de TFL Client. Sin telemetría: nada sale de la
//! máquina. Las mediciones van al log (`tracing`, target `tfl::perf`) y a un
//! buffer en memoria que se puede consultar con `get_perf_report` (también
//! desde la consola del WebView: `await window.__tflPerf()`).
//!
//! Está activa en builds de desarrollo (debug) y, en cualquier build, si el
//! proceso arranca con la variable de entorno `TFL_PERF=1` — así se le puede
//! pedir a alguien con un release que mida sin recompilar. Apagada, medir
//! cuesta un `Instant::now()` y nada más.
//!
//! Qué se mide (nombre → qué es):
//! - `startup.setup_done`, `startup.window_shown`: tiempo desde que arranca el
//!   proceso hasta que termina `setup()` y hasta que se muestra la ventana.
//! - `settings.load`, `accounts.load`, `instances.load`: carga de datos.
//! - `content.search`, `content.install` (incluye resolver dependencias y bajar).
//! - `launch.total` (de "Jugar" a proceso de Minecraft arrancado) y sus fases
//!   `launch.java`, `launch.prepare_minecraft`, `launch.auth_refresh`,
//!   `launch.spawn`.
//! - `idle.memory_mb`, `idle.cpu_percent`: consumo del launcher en reposo.
//! - `ui.*`: lo que mide el frontend (p. ej. `ui.usable`) y reporta por
//!   `perf_record`.

use serde::Serialize;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::command;

const MAX_RECORDS: usize = 500;
const MAX_NAME_LEN: usize = 60;

/// Objetivos internos por métrica, en milisegundos. `None` = todavía sin
/// objetivo: se define después de tomar una línea base en una máquina real
/// (ver docs/RENDIMIENTO.md). Si se completa un valor, cada medición que lo
/// supere deja un `warn` en el log, así una regresión salta sola.
fn budget_ms(name: &str) -> Option<f64> {
    match name {
        "startup.window_shown" => None,
        "ui.usable" => None,
        "instances.load" => None,
        "launch.total" => None,
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PerfRecord {
    pub name: String,
    pub ms: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PerfSummary {
    pub count: usize,
    pub min_ms: f64,
    pub avg_ms: f64,
    pub max_ms: f64,
    pub last_ms: f64,
}

/// Consumo del propio launcher en un momento dado.
#[derive(Debug, Clone, Serialize)]
pub struct IdleSample {
    pub memory_mb: f64,
    pub cpu_percent: f32,
}

#[derive(Debug, Serialize)]
pub struct PerfReport {
    pub enabled: bool,
    pub summary: BTreeMap<String, PerfSummary>,
    pub recent: Vec<PerfRecord>,
    pub idle: Option<IdleSample>,
}

/// Buffer de mediciones. Separado del estado global para poder probarlo.
pub struct Recorder {
    enabled: bool,
    records: Mutex<VecDeque<PerfRecord>>,
}

impl Recorder {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            records: Mutex::new(VecDeque::new()),
        }
    }

    pub fn record_ms(&self, name: &str, ms: f64) {
        if !self.enabled {
            return;
        }
        tracing::info!(target: "tfl::perf", metric = name, ms = format_args!("{ms:.1}"));
        if let Some(budget) = budget_ms(name)
            && ms > budget
        {
            tracing::warn!(
                target: "tfl::perf",
                metric = name,
                ms = format_args!("{ms:.1}"),
                budget_ms = budget,
                "se superó el objetivo interno"
            );
        }
        let mut records = self.records.lock().unwrap();
        if records.len() >= MAX_RECORDS {
            records.pop_front();
        }
        records.push_back(PerfRecord {
            name: name.to_string(),
            ms,
        });
    }

    pub fn summary(&self) -> BTreeMap<String, PerfSummary> {
        let records = self.records.lock().unwrap();
        let mut grouped: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        for r in records.iter() {
            grouped.entry(&r.name).or_default().push(r.ms);
        }
        grouped
            .into_iter()
            .map(|(name, values)| {
                let count = values.len();
                let sum: f64 = values.iter().sum();
                let summary = PerfSummary {
                    count,
                    min_ms: values.iter().copied().fold(f64::INFINITY, f64::min),
                    avg_ms: sum / count as f64,
                    max_ms: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                    last_ms: *values.last().unwrap_or(&0.0),
                };
                (name.to_string(), summary)
            })
            .collect()
    }

    pub fn recent(&self, limit: usize) -> Vec<PerfRecord> {
        let records = self.records.lock().unwrap();
        records.iter().rev().take(limit).rev().cloned().collect()
    }
}

static START: LazyLock<Instant> = LazyLock::new(Instant::now);
static RECORDER: OnceLock<Recorder> = OnceLock::new();

fn recorder() -> &'static Recorder {
    RECORDER.get_or_init(|| {
        let from_env = std::env::var("TFL_PERF").is_ok_and(|v| v == "1");
        Recorder::new(cfg!(debug_assertions) || from_env)
    })
}

pub fn enabled() -> bool {
    recorder().enabled
}

/// Fija el "tiempo cero" del arranque: llamar lo primero en `run()`.
pub fn init() {
    LazyLock::force(&START);
    recorder();
}

/// Mide desde que se crea hasta que se descarta (al salir del scope).
pub struct Span {
    name: &'static str,
    started: Instant,
}

pub fn span(name: &'static str) -> Span {
    Span {
        name,
        started: Instant::now(),
    }
}

impl Span {
    /// Milisegundos transcurridos hasta ahora (sin cerrar el span).
    pub fn elapsed_ms(&self) -> f64 {
        self.started.elapsed().as_secs_f64() * 1000.0
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        recorder().record_ms(self.name, self.elapsed_ms());
    }
}

/// Registra el tiempo transcurrido desde el arranque del proceso.
pub fn mark_since_start(name: &'static str) {
    recorder().record_ms(name, START.elapsed().as_secs_f64() * 1000.0);
}

/// Consumo (RAM y CPU) del propio proceso del launcher. La CPU necesita dos
/// lecturas separadas en el tiempo, por eso tarda ~1 segundo.
pub async fn idle_sample() -> Option<IdleSample> {
    let pid = sysinfo::Pid::from_u32(std::process::id());
    let mut sys = sysinfo::System::new();
    sys.refresh_process(pid);
    tokio::time::sleep(Duration::from_millis(1000)).await;
    sys.refresh_process(pid);
    let process = sys.process(pid)?;
    Some(IdleSample {
        memory_mb: process.memory() as f64 / (1024.0 * 1024.0),
        cpu_percent: process.cpu_usage(),
    })
}

/// Toma una muestra de reposo unos segundos después de abrir y la deja en el
/// log (y en el buffer, como `idle.memory_mb` / `idle.cpu_percent`).
pub fn spawn_idle_logger(delay: Duration) {
    if !enabled() {
        return;
    }
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(delay).await;
        if let Some(sample) = idle_sample().await {
            recorder().record_ms("idle.memory_mb", sample.memory_mb);
            recorder().record_ms("idle.cpu_percent", f64::from(sample.cpu_percent));
        }
    });
}

#[command]
pub async fn get_perf_report() -> PerfReport {
    let enabled = enabled();
    PerfReport {
        enabled,
        summary: recorder().summary(),
        recent: recorder().recent(50),
        idle: if enabled { idle_sample().await } else { None },
    }
}

/// El frontend reporta sus mediciones (`ui.*`). Se ignora lo que no tenga un
/// nombre razonable: el nombre termina en el log.
#[command]
pub fn perf_record(name: String, ms: f64) {
    let valid = !name.is_empty()
        && name.len() <= MAX_NAME_LEN
        && name.starts_with("ui.")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if valid && ms.is_finite() && ms >= 0.0 {
        recorder().record_ms(&name, ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_recorder_stores_nothing() {
        let r = Recorder::new(false);
        r.record_ms("x", 5.0);
        assert!(r.summary().is_empty());
        assert!(r.recent(10).is_empty());
    }

    #[test]
    fn summary_groups_by_name() {
        let r = Recorder::new(true);
        r.record_ms("a", 10.0);
        r.record_ms("a", 30.0);
        r.record_ms("b", 7.0);
        let s = r.summary();
        assert_eq!(s.len(), 2);
        let a = &s["a"];
        assert_eq!(a.count, 2);
        assert_eq!(a.min_ms, 10.0);
        assert_eq!(a.max_ms, 30.0);
        assert_eq!(a.avg_ms, 20.0);
        assert_eq!(a.last_ms, 30.0);
        assert_eq!(s["b"].count, 1);
    }

    #[test]
    fn buffer_is_capped_and_keeps_the_newest() {
        let r = Recorder::new(true);
        for i in 0..(MAX_RECORDS + 25) {
            r.record_ms("n", i as f64);
        }
        let s = &r.summary()["n"];
        assert_eq!(s.count, MAX_RECORDS);
        assert_eq!(s.min_ms, 25.0);
        assert_eq!(s.last_ms, (MAX_RECORDS + 24) as f64);
        let recent = r.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[2].ms, (MAX_RECORDS + 24) as f64);
    }

    #[test]
    fn frontend_names_are_validated() {
        // `perf_record` escribe en el recorder global: solo se prueba que
        // nombres inválidos no rompen nada y que los válidos no entran con
        // tiempos absurdos.
        perf_record("no-ui-prefix".into(), 1.0);
        perf_record("ui.".to_string() + &"x".repeat(100), 1.0);
        perf_record("ui.bad name".into(), 1.0);
        perf_record("ui.ok".into(), f64::NAN);
        perf_record("ui.ok".into(), -3.0);
        let names: Vec<String> = recorder().summary().into_keys().collect();
        assert!(
            !names
                .iter()
                .any(|n| n.contains("no-ui-prefix") || n.contains("bad name"))
        );
        assert!(!names.iter().any(|n| n.len() > MAX_NAME_LEN));
    }

    #[test]
    fn span_measures_elapsed_time() {
        let r = Recorder::new(true);
        let started = Instant::now();
        std::thread::sleep(Duration::from_millis(15));
        r.record_ms("sleepy", started.elapsed().as_secs_f64() * 1000.0);
        assert!(r.summary()["sleepy"].last_ms >= 15.0);
    }

    #[tokio::test]
    async fn idle_sample_reads_this_process() {
        let sample = idle_sample()
            .await
            .expect("el propio proceso tiene que existir");
        assert!(sample.memory_mb > 1.0);
        assert!(sample.cpu_percent >= 0.0);
    }
}
