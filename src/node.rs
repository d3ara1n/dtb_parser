use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Write};

use crate::byte_utils::{align_size, locate_block, read_aligned_be_u32, read_aligned_name};
use crate::device_tree::InheritedValues;
use crate::error::DeviceTreeError;
use crate::error::Result;
use crate::header::DeviceTreeHeader;
use crate::prop::{NodeProperty, PropertyValue};
use crate::traits::{HasNamedChildNode, HasNamedProperty};

/// Node of [crate::device_tree::DeviceTree]
/// Contains owned children and properties
pub struct DeviceTreeNode<'a> {
    pub(crate) block_count: usize,
    name: &'a str,
    props: Vec<NodeProperty<'a>>,
    nodes: Vec<DeviceTreeNode<'a>>,
}

impl<'a> DeviceTreeNode<'a> {
    pub(crate) fn from_bytes(
        data: &'a [u8],
        header: &DeviceTreeHeader,
        start: usize,
        inherited: InheritedValues<'a>,
        mut owned: InheritedValues<'a>,
    ) -> Result<Self> {
        let block_start = align_size(start);
        if let Some(begin_node) = read_aligned_be_u32(data, block_start) {
            if begin_node == 0x1 {
                if let Some(name) = read_aligned_name(data, block_start + 1) {
                    let mut props = Vec::<NodeProperty>::new();
                    let mut nodes = Vec::<DeviceTreeNode>::new();
                    let name_blocks = if align_size(name.len() + 1) == 0 {
                        // including zero-length name which occupied one block
                        1
                    } else {
                        // it only contains asciis so does equal to byte length
                        align_size(name.len() + 1)
                    };

                    let mut current_block = block_start + name_blocks + 1;

                    while let Some(token) = read_aligned_be_u32(data, current_block) {
                        match token {
                            0x3 => {
                                if let Ok(prop) = NodeProperty::from_bytes(
                                    data,
                                    header,
                                    locate_block(current_block),
                                    &inherited,
                                    &owned,
                                ) {
                                    current_block += prop.block_count;
                                    if nodes.is_empty() {
                                        // it's inheritable value
                                        if let PropertyValue::Integer(v) = prop.value() {
                                            owned.update(prop.name(), *v);
                                        }
                                    }
                                    props.push(prop);
                                } else {
                                    return Err(DeviceTreeError::ParsingFailed);
                                }
                            }
                            0x1 => {
                                if let Ok(node) = DeviceTreeNode::from_bytes(
                                    data,
                                    header,
                                    locate_block(current_block),
                                    owned.clone(),
                                    owned.clone(),
                                ) {
                                    current_block += node.block_count;
                                    nodes.push(node);
                                } else {
                                    return Err(DeviceTreeError::ParsingFailed);
                                }
                            }
                            0x2 | 0x9 => {
                                current_block += 1;
                                break;
                            }
                            _ => current_block += 1,
                        };
                    }
                    Ok(Self {
                        block_count: current_block - block_start,
                        name,
                        props,
                        nodes,
                    })
                } else {
                    Err(DeviceTreeError::ParsingFailed)
                }
            } else {
                Err(DeviceTreeError::InvalidToken)
            }
        } else {
            Err(DeviceTreeError::ParsingFailed)
        }
    }

    /// Get the name of this node
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Get a reference of its owned properties
    pub fn props(&self) -> &[NodeProperty<'a>] {
        &self.props
    }

    /// Get a reference of its owned children
    pub fn nodes(&self) -> &[DeviceTreeNode<'a>] {
        &self.nodes
    }
}

impl<'a> Display for DeviceTreeNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if let Err(err) = writeln!(f, "{} {{", self.name) {
            return Err(err);
        }
        for i in &self.props {
            if let Err(err) = writeln!(f, "\t{}", i) {
                return Err(err);
            }
        }
        for i in &self.nodes {
            let mut buffer = String::new();
            if let Err(err) = write!(buffer, "\t{}", i) {
                return Err(err);
            }
            let mut first_line = true;
            for j in buffer.split('\n') {
                if !first_line {
                    if let Err(err) = write!(f, "\t") {
                        return Err(err);
                    }
                } else {
                    first_line = false;
                }
                if let Err(err) = writeln!(f, "{}", j) {
                    return Err(err);
                }
            }
        }
        write!(f, "}};")
    }
}

impl<'a> HasNamedChildNode for DeviceTreeNode<'a> {
    fn has_children(&self) -> bool {
        !self.nodes().is_empty()
    }

    fn find_child(&self, name: &str) -> Option<&DeviceTreeNode> {
        let mut option: Option<&DeviceTreeNode> = None;
        for i in &self.nodes {
            if i.name() == name {
                option = Some(i);
                break;
            }
        }
        option
    }
}

impl<'a> HasNamedProperty for DeviceTreeNode<'a> {
    fn has_props(&self) -> bool {
        !self.props.is_empty()
    }

    fn find_prop(&self, name: &str) -> Option<&NodeProperty> {
        let mut option: Option<&NodeProperty> = None;
        for i in &self.props {
            if i.name() == name {
                option = Some(i);
                break;
            }
        }
        option
    }
}
