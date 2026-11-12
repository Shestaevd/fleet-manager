use fleet_model::MacAddress;

use crate::process::network::{machine_mac_address, pick_mac_address};

#[test]
fn the_smallest_real_address_of_a_physical_card_is_picked() {
    let picked = pick_mac_address([
        (true, [0x52, 0x54, 0x00, 0x00, 0x00, 0x09]),
        (true, [0x08, 0x00, 0x27, 0x00, 0x00, 0x01]),
        (false, [0x02, 0x42, 0xac, 0x11, 0x00, 0x02]), // a container bridge, smaller but virtual
        (true, [0; 6]),                                // no real address
    ]);
    assert_eq!(picked, Some(MacAddress([0x08, 0x00, 0x27, 0x00, 0x00, 0x01])));
}

#[test]
fn without_a_physical_card_with_an_address_there_is_none() {
    assert_eq!(pick_mac_address([]), None);
    assert_eq!(pick_mac_address([(false, [1, 2, 3, 4, 5, 6]), (true, [0; 6])]), None);
}

#[test]
fn the_machines_address_is_the_same_every_time_it_is_asked() {
    assert_eq!(machine_mac_address(), machine_mac_address());
}
