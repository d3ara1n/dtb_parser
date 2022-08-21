use dtb_parser::device_tree::DeviceTree;

fn main() {
    let tree = DeviceTree::from_bytes(include_bytes!("../tests/device.dtb")).unwrap();

    println!("{}", tree);
}