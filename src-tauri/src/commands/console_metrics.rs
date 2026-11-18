//! On-demand console providers. No worker/timer survives closing a console.
//! Network and GPU fall back to the computer, never to an invented per-game value.
use super::{ProcessStats, process_stats};
use serde::Serialize;
use std::{
    path::Path,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use sysinfo::{Disks, MemoryRefreshKind, Networks, RefreshKind, System};

#[derive(Clone, Serialize)]
pub struct NetworkMetrics {
    pub scope: &'static str,
    pub provider: &'static str,
    pub received_bytes_per_sec: Option<f64>,
    pub transmitted_bytes_per_sec: Option<f64>,
    pub total_received_bytes: u64,
    pub total_transmitted_bytes: u64,
    pub interval_ms: u64,
    pub total_period: &'static str,
}
#[derive(Clone, Serialize)]
pub struct StorageMetrics {
    pub scope: &'static str,
    pub provider: &'static str,
    pub used_bytes: u64,
    pub total_bytes: u64,
}
#[derive(Clone, Serialize)]
pub struct IoMetrics {
    pub scope: &'static str,
    pub provider: &'static str,
    pub read_bytes_per_sec: Option<f64>,
    pub write_bytes_per_sec: Option<f64>,
    pub total_read_bytes: Option<u64>,
    pub total_write_bytes: Option<u64>,
    pub total_period: &'static str,
}
#[derive(Clone, Serialize)]
pub struct GpuMetrics {
    pub name: String,
    pub scope: &'static str,
    pub provider: &'static str,
    pub utilization_percent: Option<f64>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
}
#[derive(Clone, Serialize)]
pub struct ConsoleMetrics {
    pub ui_mode: String,
    pub process: Option<ProcessStats>,
    pub running: bool,
    pub sampled_at_ms: u64,
    pub interval_ms: u64,
    pub memory_total_mb: u64,
    pub network: Option<NetworkMetrics>,
    pub storage: Option<StorageMetrics>,
    pub io: Option<IoMetrics>,
    pub gpus: Vec<GpuMetrics>,
}
struct ComputerSampler {
    networks: Networks,
    previous: Option<Instant>,
    network: Option<NetworkMetrics>,
    gpu_at: Option<Instant>,
    gpus: Vec<GpuMetrics>,
    #[cfg(target_os = "windows")]
    disk: Option<windows_disk::DiskQuery>,
    #[cfg(target_os = "windows")]
    io: Option<IoMetrics>,
}
static COMPUTER: LazyLock<Mutex<ComputerSampler>> = LazyLock::new(|| {
    Mutex::new(ComputerSampler {
        networks: Networks::new_with_refreshed_list(),
        previous: None,
        network: None,
        gpu_at: None,
        gpus: vec![],
        #[cfg(target_os = "windows")]
        disk: windows_disk::DiskQuery::open(),
        #[cfg(target_os = "windows")]
        io: None,
    })
});

pub fn collect(pid: Option<u32>, directory: &Path) -> Result<ConsoleMetrics, String> {
    let process = pid.and_then(process_stats);
    let now = Instant::now();
    let mut computer = COMPUTER
        .lock()
        .map_err(|_| "El monitor no está disponible")?;
    let interval = computer
        .previous
        .map(|previous| now.duration_since(previous).as_millis() as u64);
    let ready_interval = interval.filter(|ms| *ms > 0 && *ms <= 10_000);
    if interval.is_none_or(|ms| ms >= 1000) {
        computer.networks.refresh_list();
        let mut rx = 0u64;
        let mut tx = 0u64;
        let mut total_rx = 0u64;
        let mut total_tx = 0u64;
        let mut found = false;
        for (_, interface) in &computer.networks {
            found = true;
            rx = rx.saturating_add(interface.received());
            tx = tx.saturating_add(interface.transmitted());
            total_rx = total_rx.saturating_add(interface.total_received());
            total_tx = total_tx.saturating_add(interface.total_transmitted());
        }
        computer.network = found.then(|| NetworkMetrics {
            scope: "machine_interfaces",
            provider: "sysinfo / OS interface counters",
            received_bytes_per_sec: ready_interval.map(|ms| rx as f64 * 1000.0 / ms as f64),
            transmitted_bytes_per_sec: ready_interval.map(|ms| tx as f64 * 1000.0 / ms as f64),
            total_received_bytes: total_rx,
            total_transmitted_bytes: total_tx,
            interval_ms: ready_interval.unwrap_or(0),
            total_period: "interface_uptime",
        });
        #[cfg(target_os = "windows")]
        {
            computer.io = computer
                .disk
                .as_mut()
                .and_then(|query| query.sample(ready_interval.is_some()));
        }
        computer.previous = Some(now);
    }
    if computer
        .gpu_at
        .is_none_or(|at| now.duration_since(at) >= Duration::from_secs(2))
    {
        computer.gpus = nvidia_metrics().unwrap_or_default();
        computer.gpu_at = Some(now);
    }
    let network = computer.network.clone();
    let gpus = computer.gpus.clone();
    #[cfg(target_os = "windows")]
    let io = computer.io.clone();
    #[cfg(not(target_os = "windows"))]
    let io = process.as_ref().map(|process| IoMetrics {
        scope: "process_tree",
        provider: "sysinfo / OS process disk counters",
        read_bytes_per_sec: (process.ready && process.sample_interval_ms > 0).then(|| {
            process.disk_read_bytes_delta as f64 * 1000.0 / process.sample_interval_ms as f64
        }),
        write_bytes_per_sec: (process.ready && process.sample_interval_ms > 0).then(|| {
            process.disk_write_bytes_delta as f64 * 1000.0 / process.sample_interval_ms as f64
        }),
        total_read_bytes: Some(process.disk_read_bytes_total),
        total_write_bytes: Some(process.disk_write_bytes_total),
        total_period: "current_process_tree_lifetime",
    });
    let interval_ms = network.as_ref().map(|n| n.interval_ms).unwrap_or(0);
    drop(computer);
    let disks = Disks::new_with_refreshed_list();
    let storage = disks
        .list()
        .iter()
        .filter(|disk| directory.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .map(|disk| StorageMetrics {
            scope: "instance_volume",
            provider: "sysinfo / filesystem space",
            used_bytes: disk.total_space().saturating_sub(disk.available_space()),
            total_bytes: disk.total_space(),
        });
    let system = System::new_with_specifics(
        RefreshKind::new().with_memory(MemoryRefreshKind::new().with_ram()),
    );
    Ok(ConsoleMetrics {
        ui_mode: crate::services::SettingsManager::read().ui_mode.clone(),
        running: pid.is_some(),
        process,
        sampled_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        interval_ms,
        memory_total_mb: system.total_memory() / 1048576,
        network,
        storage,
        io,
        gpus,
    })
}

// nvidia-smi is an optional installed driver provider. Never install a tool,
// invoke a shell, interpolate input or infer unsupported per-process GPU metrics.
fn nvidia_metrics() -> Option<Vec<GpuMetrics>> {
    #[cfg(target_os = "macos")]
    {
        return None;
    }
    #[cfg(not(target_os = "macos"))]
    {
        use std::process::{Command, Stdio};
        let mut command = Command::new("nvidia-smi");
        command
            .args([
                "--query-gpu=name,utilization.gpu,memory.used,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.spawn().ok()?;
        let deadline = Instant::now() + Duration::from_millis(900);
        loop {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => break,
                Ok(Some(_)) => return None,
                Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                Ok(None) if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
        let output = child.wait_with_output().ok()?;
        if output.stdout.len() > 65536 {
            return None;
        }
        Some(parse_nvidia(std::str::from_utf8(&output.stdout).ok()?))
    }
}
fn parse_nvidia(csv: &str) -> Vec<GpuMetrics> {
    fn number(value: &str) -> Option<f64> {
        value
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite() && *value >= 0.0)
    }
    csv.lines()
        .take(32)
        .filter_map(|line| {
            let fields: Vec<_> = line.split(',').map(str::trim).collect();
            if fields.len() != 4 || fields[0].is_empty() {
                return None;
            }
            Some(GpuMetrics {
                name: fields[0].into(),
                scope: "machine_adapter",
                provider: "NVIDIA driver / nvidia-smi",
                utilization_percent: number(fields[1]).filter(|value| *value <= 100.0),
                vram_used_bytes: number(fields[2]).map(|mib| (mib * 1048576.0) as u64),
                vram_total_bytes: number(fields[3]).map(|mib| (mib * 1048576.0) as u64),
            })
        })
        .collect()
}

#[cfg(target_os = "windows")]
mod windows_disk {
    use super::IoMetrics;
    #[repr(C)]
    struct Value {
        status: u32,
        value: f64,
    }
    #[link(name = "pdh")]
    unsafe extern "system" {
        fn PdhOpenQueryW(source: *const u16, data: usize, query: *mut isize) -> u32;
        fn PdhAddEnglishCounterW(
            query: isize,
            path: *const u16,
            data: usize,
            counter: *mut isize,
        ) -> u32;
        fn PdhCollectQueryData(query: isize) -> u32;
        fn PdhGetFormattedCounterValue(
            counter: isize,
            format: u32,
            kind: *mut u32,
            value: *mut Value,
        ) -> u32;
        fn PdhCloseQuery(query: isize) -> u32;
    }
    pub(super) struct DiskQuery {
        query: isize,
        read: isize,
        write: isize,
    }
    impl DiskQuery {
        pub(super) fn open() -> Option<Self> {
            let mut query = Self {
                query: 0,
                read: 0,
                write: 0,
            };
            // Handles live behind the ComputerSampler mutex; buffers are NUL-terminated.
            unsafe {
                if PdhOpenQueryW(std::ptr::null(), 0, &mut query.query) != 0 {
                    return None;
                }
                for (path, counter) in [
                    (
                        r"\PhysicalDisk(_Total)\Disk Read Bytes/sec",
                        &mut query.read,
                    ),
                    (
                        r"\PhysicalDisk(_Total)\Disk Write Bytes/sec",
                        &mut query.write,
                    ),
                ] {
                    let path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
                    if PdhAddEnglishCounterW(query.query, path.as_ptr(), 0, counter) != 0 {
                        return None;
                    }
                }
            }
            Some(query)
        }
        pub(super) fn sample(&mut self, ready: bool) -> Option<IoMetrics> {
            unsafe {
                if PdhCollectQueryData(self.query) != 0 {
                    return None;
                }
            }
            fn read(counter: isize) -> Option<f64> {
                let mut value = Value {
                    status: u32::MAX,
                    value: 0.0,
                };
                unsafe {
                    if PdhGetFormattedCounterValue(counter, 0x200, std::ptr::null_mut(), &mut value)
                        != 0
                    {
                        return None;
                    }
                }
                (value.status <= 1 && value.value.is_finite() && value.value >= 0.0)
                    .then_some(value.value)
            }
            Some(IoMetrics {
                scope: "machine_disks",
                provider: "Windows PDH / PhysicalDisk(_Total)",
                read_bytes_per_sec: ready.then(|| read(self.read)).flatten(),
                write_bytes_per_sec: ready.then(|| read(self.write)).flatten(),
                total_read_bytes: None,
                total_write_bytes: None,
                total_period: "unavailable",
            })
        }
    }
    impl Drop for DiskQuery {
        fn drop(&mut self) {
            if self.query != 0 {
                unsafe {
                    PdhCloseQuery(self.query);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gpu_provider_preserves_adapters_and_missing_values_without_zeroes() {
        let gpus = parse_nvidia(
            "NVIDIA Test A, 42, 512, 8192\nNVIDIA Test B, [N/A], [N/A], 16384\nmalformed\n",
        );
        assert_eq!(gpus.len(), 2);
        assert_eq!(gpus[0].utilization_percent, Some(42.0));
        assert_eq!(gpus[0].vram_used_bytes, Some(512 * 1048576));
        assert_eq!(gpus[1].utilization_percent, None);
        assert_eq!(gpus[1].vram_used_bytes, None);
        assert_eq!(gpus[1].scope, "machine_adapter");
    }
    #[test]
    fn stopped_instance_monitor_still_labels_real_computer_data() {
        let snapshot = collect(None, &std::env::temp_dir()).unwrap();
        assert!(!snapshot.running);
        assert!(snapshot.process.is_none());
        assert!(snapshot.memory_total_mb > 0);
        if let Some(network) = snapshot.network {
            assert_eq!(network.scope, "machine_interfaces");
        }
        if let Some(storage) = snapshot.storage {
            assert!(storage.total_bytes >= storage.used_bytes);
        }
    }
}
