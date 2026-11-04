use fleet_model::AgentId;
use sysinfo::System;

use crate::domain::model::IntoHeartBeat;
use fleet_model::Telemetry;
use crate::process::telemetry::collect_telemetry;

#[test]
fn collected_telemetry_has_sane_values() {
    let mut system = System::new();
    let telemetry = collect_telemetry(&mut system);

    assert!(telemetry.cpu_cores > 0);
    assert_eq!(telemetry.cpu_core_usage_percent.len(), telemetry.cpu_cores as usize);
    assert!(telemetry.memory_total_bytes > 0);
    assert!(telemetry.memory_used_bytes <= telemetry.memory_total_bytes);
    assert!(telemetry.timestamp_secs > 1_700_000_000);
    assert!(telemetry.load_average.iter().all(|load| *load >= 0.0));
}

#[test]
fn heart_beat_copies_telemetry_and_gets_a_new_request_id_each_time() {
    let telemetry = Telemetry {
        timestamp_secs: 1_700_000_000,
        uptime_secs: 3600,
        cpu_cores: 4,
        cpu_usage_percent: 12.5,
        cpu_core_usage_percent: vec![10.0, 15.0, 5.0, 20.0],
        load_average: [0.5, 0.4, 0.3],
        memory_total_bytes: 16 << 30,
        memory_used_bytes: 4 << 30,
    };
    let agent_id = AgentId::new();

    let first = telemetry.clone().into_heart_beat(agent_id);
    let second = telemetry.into_heart_beat(agent_id);

    assert_eq!(first.agent_id, agent_id);
    assert_eq!(first.uptime_secs, 3600);
    assert_eq!(first.cpu_cores, 4);
    assert_eq!(first.cpu_core_usage_percent, vec![10.0, 15.0, 5.0, 20.0]);
    assert_eq!(first.load_average, [0.5, 0.4, 0.3]);
    assert_eq!(first.memory_used_bytes, 4 << 30);
    assert_ne!(first.package_id, second.package_id);
}
