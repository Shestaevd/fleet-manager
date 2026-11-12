use crate::MacAddress;

#[test]
fn a_mac_address_is_written_as_six_lowercase_hex_pairs() {
    let mac = MacAddress([0x00, 0x1A, 0xB2, 0x0c, 0xFF, 0x07]);
    assert_eq!(mac.to_string(), "00:1a:b2:0c:ff:07");
    assert_eq!(format!("{mac:?}"), "MacAddress(00:1a:b2:0c:ff:07)");
}
