use crate::node::DeviceTreeNode;
use crate::prop::NodeProperty;

pub trait HasNamedProperty {
    fn has_props(&self) -> bool;

    fn find_prop(&self, name: &str) -> Option<&NodeProperty>;
}

pub trait HasNamedChildNode {
    fn has_children(&self) -> bool;

    fn find_child(&self, name: &str) -> Option<&DeviceTreeNode>;
}