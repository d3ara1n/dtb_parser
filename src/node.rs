use alloc::vec::Vec;

use crate::byte_utils::{align_size, locate_block, read_aligned_name, read_aligned_token};
use crate::error::DeviceTreeError;
use crate::error::Result;
use crate::header::DeviceTreeHeader;
use crate::prop::{NodeProperty, PropertyValue};
use crate::structure::StructureToken;

pub struct DeviceTreeNode<'a> {
    pub(crate) block_count: usize,
    name: &'a str,
    props: Vec<NodeProperty<'a>>,
    nodes: Vec<DeviceTreeNode<'a>>,
}

impl<'a> DeviceTreeNode<'a> {
    pub fn from_bytes(data: &'a [u8], header: &DeviceTreeHeader, start: usize, address_cells: usize, size_cells: usize) -> Result<Self> {
        let mut address_cells = address_cells;
        let mut size_cells = size_cells;
        let block_start = align_size(start);
        if let Some(begin_node) = read_aligned_token(data, block_start) {
            if begin_node == StructureToken::BeginNode {
                if let Some(name) = read_aligned_name(data, block_start + 1) {
                    let mut props = Vec::<NodeProperty>::new();
                    let mut nodes = Vec::<DeviceTreeNode>::new();

                    let name_blocks = if align_size(name.len() + 1) == 0 { // including \t
                        1
                    } else {
                        align_size(name.len() + 1)
                    }; // it only contains ascii so equals to byte length

                    let mut current_block = block_start + name_blocks + 1;

                    while let Some(token) = read_aligned_token(data, current_block) {
                        match token {
                            StructureToken::Property => {
                                if let Ok(prop) = NodeProperty::from_bytes(data, header, locate_block(current_block), address_cells, size_cells) {
                                    current_block += prop.block_count;
                                    if prop.name() == "#address-cells" {
                                        address_cells = match prop.value(){
                                            PropertyValue::Integer(it) => *it as usize,
                                            _ => address_cells
                                        };
                                    }
                                    if prop.name() == "#size-cells" {
                                        size_cells = match prop.value(){
                                            PropertyValue::Integer(it) => *it as usize,
                                            _ => size_cells
                                        };
                                    }
                                    props.push(prop);
                                } else {
                                    return Err(DeviceTreeError::ParsingFailed);
                                }
                            }
                            StructureToken::BeginNode => {
                                if let Ok(node) = DeviceTreeNode::from_bytes(data, header, locate_block(current_block), address_cells, size_cells) {
                                    current_block += node.block_count;
                                    nodes.push(node);
                                } else {
                                    return Err(DeviceTreeError::ParsingFailed);
                                }
                            }
                            StructureToken::Nop => current_block += 1,
                            StructureToken::EndNode | StructureToken::End => {
                                current_block += 1;
                                break;
                            }
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

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn props(&self) -> &[NodeProperty<'a>] {
        &self.props
    }

    pub fn nodes(&self) -> &[DeviceTreeNode<'a>] {
        &self.nodes
    }
}