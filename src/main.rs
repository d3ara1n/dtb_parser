use dtb_parser::device_tree::DeviceTree;

static DTB: &[u8] = include_bytes!("../tests/device.dtb");

fn main() {
    let tree = DeviceTree::from_address(DTB.as_ptr() as usize).unwrap();
    println!("{}", tree);
}
