use dtb_parser::device_tree::DeviceTree;
use dtb_parser::traits::{HasNamedChildNode, HasNamedProperty};

const DTB: &[u8] = include_bytes!("device.dtb");

#[test]
fn node_name() {
    let tree = DeviceTree::from_bytes(DTB).unwrap();
    assert_eq!(tree.root().name(), "");
    assert!(!matches!(tree.root().find_child("cpus"), None));
    assert!(!matches!(tree.root().find_child("memory@0"), None));
    assert!(matches!(tree.root().find_child("cpu@0"), None));
}

#[test]
fn prop_name() {
    let tree = DeviceTree::from_bytes(DTB).unwrap();
    assert!(!matches!(tree.root().find_prop("model"), None));
    assert!(!matches!(tree.root().find_prop("#address-cells"), None));
    assert!(matches!(tree.root().find_prop("#interrupt-cells"), None));

    let soc = tree.root().find_child("soc");
    assert!(!matches!(soc, None));
    assert!(matches!(soc.unwrap().find_child("soc_gpio1"), None));
}
