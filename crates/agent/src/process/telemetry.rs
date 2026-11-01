use std::time::{SystemTime, UNIX_EPOCH};

use sysinfo::{LoadAvg, System};

use crate::domain::model::Telemetry;

pub fn collect_telemetry(system: &mut System) -> Telemetry {
    system.refresh_cpu_usage();
    system.refresh_memory();

    let load: LoadAvg = System::load_average();
    Telemetry {
        timestamp_secs: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs())
            .unwrap_or_default(),
        uptime_secs: System::uptime(),
        cpu_cores: system.cpus().len() as u32,
        cpu_usage_percent: system.global_cpu_usage(),
        cpu_core_usage_percent: system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect(),
        load_average: [load.one, load.five, load.fifteen],
        memory_total_bytes: system.total_memory(),
        memory_used_bytes: system.used_memory(),
    }
}
