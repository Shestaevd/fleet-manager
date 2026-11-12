use std::path::Path;

use fleet_model::MacAddress;
use sysinfo::Networks;

const SYS_CLASS_NET: &str = "/sys/class/net";

pub fn machine_mac_address() -> MacAddress {
    let networks: Networks = Networks::new_with_refreshed_list();
    pick_mac_address(
        networks
            .iter()
            .map(|(name, data)| (is_physical(name), data.mac_address().0)),
    )
    .unwrap_or(MacAddress::UNSPECIFIED)
}

fn is_physical(interface: &str) -> bool {
    Path::new(SYS_CLASS_NET).join(interface).join("device").exists()
}

pub(crate) fn pick_mac_address(interfaces: impl IntoIterator<Item = (bool, [u8; 6])>) -> Option<MacAddress> {
    interfaces
        .into_iter()
        .filter(|(physical, mac)| *physical && *mac != [0u8; 6])
        .map(|(_, mac)| MacAddress(mac))
        .min_by_key(|mac| mac.0)
}
