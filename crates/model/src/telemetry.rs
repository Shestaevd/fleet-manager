#[derive(Debug, Clone, PartialEq)]
pub struct Telemetry {
    pub timestamp_secs: u64,
    pub uptime_secs: u64,
    pub cpu_cores: u32,
    pub cpu_usage_percent: f32,
    pub cpu_core_usage_percent: Vec<f32>,
    pub load_average: [f64; 3],
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
}
