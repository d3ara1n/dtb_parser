use crate::node::DeviceTreeNode;
use crate::prop::NodeProperty;

/// Indicates a struct should have [NodeProperty] and can be indexed by its name
pub trait HasNamedProperty {
    /// Having properties or not
    fn has_props(&self) -> bool;

    /// Look for a property by its name
    fn find_prop(&self, name: &str) -> Option<&NodeProperty>;
}

/// Indicates a struct should have [DeviceTreeNode] and can be indexed by its name
pub trait HasNamedChildNode {
    /// Having child nodes or not
    fn has_children(&self) -> bool;

    /// Look for a child by its name
    fn find_child(&self, name: &str) -> Option<&DeviceTreeNode>;
}
