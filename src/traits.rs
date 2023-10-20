use crate::node::DeviceTreeNode;
use crate::prop::{NodeProperty, PropertyValue};

/// Indicates a struct should have [NodeProperty] and can be indexed by its name
pub trait HasNamedProperty {
    /// Having properties or not
    fn has_props(&self) -> bool;

    /// Look for a property by its name
    fn find_prop(&self, name: &str) -> Option<&NodeProperty>;

    /// Look for a property by its name and returning its value if present
    fn of_value(&self, name: &str) -> Option<&PropertyValue>;
}

/// Indicates a struct should have [DeviceTreeNode] and can be indexed by its name
pub trait HasNamedChildNode {
    /// Having child nodes or not
    fn has_children(&self) -> bool;

    /// Look for a child by its name
    fn find_child(&self, name: &str) -> Option<&DeviceTreeNode>;
}

/// Ability to access a node's property value
pub trait FindPropertyValue {
    /// Find property and get its value in one call
    fn value(&self, prop_name: &str) -> Option<&PropertyValue>;
}

impl<T> FindPropertyValue for T
where
    T: HasNamedProperty,
{
    fn value(&self, prop_name: &str) -> Option<&PropertyValue> {
        self.find_prop(prop_name).map(|f| f.value())
    }
}
