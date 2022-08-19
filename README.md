# dtb_parser

```rust
pub const BLOB: &[u8] = include_bytes!("device.dtb");

fn main() {
    // no std but alloc depended device tree blob parsing lib
    let tree = DeviceTree::from_bytes(BLOB).unwrap();
    print_node(tree.root(), 0);
}

fn print_node(node: &DeviceTreeNode, level: usize) {
    for _ in 0..level {
        print!("\t");
    }
    println!("{} {{", node.name());
    for prop in node.props() {
        print_prop(prop, level + 1);
    }
    for child in node.nodes() {
        print_node(child, level + 1);
    }
    for _ in 0..level {
        print!("\t");
    }
    println!("}}");
}

fn print_prop(prop: &NodeProperty, level: usize) {
    for _ in 0..level {
        print!("\t");
    }
    match prop.value(){
        PropertyValue::None => println!("{};", prop.name()),
        _ => println!("{} = {:?};", prop.name(), prop.value())
    }
}
```