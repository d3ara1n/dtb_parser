# dtb_parser


## Usage

```rust
pub const BLOB: &[u8] = include_bytes!("device.dtb");

fn main() {
    let tree = DeviceTree::from_bytes(BLOB).unwrap();
    println!("{}", tree);
    
    assert!(!matches!(tree.find_child("memory@0"), None));
}
```

## TODO
- [x] Tree&Node parsing
- [x] Property with inherited value (#address-cells etc)
- [x] Display trait for the whole tree (output has subtle differences with dts mainly in values presentation which affected by `#<specifier>-cells`)
- [ ] PHandle binding
- [ ] Nexus node and specifier mapping