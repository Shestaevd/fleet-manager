use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct Telemetry {
    pub timestamp: Duration,
    pub uptime: Duration,
    pub cpu_cores: u32,
    pub cpu_usage_percent: f32,
    pub cpu_core_usage_percent: Vec<f32>,
    pub load_average: [f64; 3],
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
}

impl Default for Telemetry {
    fn default() -> Self {
        Telemetry {
            timestamp: Default::default(),
            uptime: Default::default(),
            cpu_cores: 0,
            cpu_usage_percent: 0.0,
            cpu_core_usage_percent: vec![],
            load_average: [0f64; 3],
            memory_total_bytes: 0,
            memory_used_bytes: 0,
        }
    }
}
