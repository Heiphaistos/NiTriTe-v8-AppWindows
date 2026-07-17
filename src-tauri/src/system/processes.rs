use serde::Serialize;
use sysinfo::{ProcessStatus, System};

#[derive(Debug, Serialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub status: String,
    pub path: String,
    pub parent_pid: u32,
    pub user: String,
    pub start_time: u64,
    pub virtual_memory_mb: f64,
    pub gpu_percent: Option<f64>,
}

/// Collecte les processus actifs avec valeurs CPU précises.
/// Cette fonction est BLOQUANTE (300ms de sleep) — toujours appeler via spawn_blocking
/// depuis un contexte async Tauri pour ne pas bloquer le runtime tokio.
pub fn collect_processes() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();
    // Double refresh : le premier établit la baseline CPU, le second mesure la delta.
    // std::thread::sleep est intentionnel ici — la fonction doit être appelée via spawn_blocking.
    std::thread::sleep(std::time::Duration::from_millis(300));
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let total_cpus = sys.cpus().len().max(1) as f64;

    let mut list: Vec<ProcessInfo> = sys.processes().values().map(|p| {
        let status = match p.status() {
            ProcessStatus::Run      => "Running",
            ProcessStatus::Suspended => "Suspended",
            ProcessStatus::Sleep    => "Sleeping",
            ProcessStatus::Idle     => "Idle",
            ProcessStatus::Stop     => "Stopped",
            ProcessStatus::Zombie   => "Zombie",
            ProcessStatus::Dead     => "Dead",
            ProcessStatus::Tracing  => "Tracing",
            ProcessStatus::Wakekill => "Wakekill",
            ProcessStatus::Waking   => "Waking",
            ProcessStatus::Parked   => "Parked",
            ProcessStatus::LockBlocked => "LockBlocked",
            ProcessStatus::UninterruptibleDiskSleep => "DiskSleep",
            ProcessStatus::Unknown(_) => "Unknown",
        };
        ProcessInfo {
            pid: p.pid().as_u32(),
            name: p.name().to_string_lossy().to_string(),
            cpu_percent: ((p.cpu_usage() as f64 / total_cpus) * 10.0).round() / 10.0,
            memory_mb: (p.memory() as f64 / 1_048_576.0 * 100.0).round() / 100.0,
            virtual_memory_mb: (p.virtual_memory() as f64 / 1_048_576.0 * 100.0).round() / 100.0,
            status: status.to_string(),
            path: p.exe().map(|e| e.to_string_lossy().to_string()).unwrap_or_default(),
            parent_pid: p.parent().map(|pp| pp.as_u32()).unwrap_or(0),
            user: p.user_id().map(|uid| format!("{:?}", uid)).unwrap_or_default(),
            start_time: p.start_time(),
            gpu_percent: None, // GPU usage non disponible via sysinfo — placeholder pour futures intégrations WMI
        }
    }).collect();

    list.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
    list
}
