#[cfg(not(feature = "std"))]
use alloc::{collections::VecDeque, string::String, vec, vec::Vec};
#[cfg(not(feature = "std"))]
use core::fmt::{Display, Formatter};
#[cfg(feature = "std")]
use std::fmt::{Display, Formatter};
#[cfg(feature = "std")]
use std::{collections::VecDeque, string::String, vec, vec::Vec};

use crate::error::{DeviceTreeError, Result};
use crate::header::DeviceTreeHeader;
use crate::node::DeviceTreeNode;
use crate::traits::HasNamedChildNode;

/// The tree structure
/// Reads data from a slice of bytes and parses into [DeviceTree]
/// Indexed by nodes and properties' names or by path for the whole tree
pub struct DeviceTree {
    header: DeviceTreeHeader,
    root: DeviceTreeNode,
}

impl DeviceTree {
    /// Parses a slice of bytes and constructs [DeviceTree]
    /// The structure should live as long as the `data`
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let magic = &data[0..4];
        if magic != [0xd0, 0x0d, 0xfe, 0xed] {
            return Err(DeviceTreeError::InvalidMagicNumber);
        }

        let header = DeviceTreeHeader::from_bytes(data)?;

        let root = DeviceTreeNode::from_bytes(
            data,
            &header,
            header.off_dt_struct as usize,
            InheritedValues::new(),
        )?;

        Ok(Self { header, root })
    }

    #[cfg(not(feature = "std"))]
    /// Parses from address where a device tree blob is located at
    pub fn from_address(addr: usize) -> Result<Self> {
        let header_bytes = unsafe { core::slice::from_raw_parts(addr as *const u8, 40) };
        let magic = &header_bytes[0..4];
        if magic != [0xd0, 0x0d, 0xfe, 0xed] {
            return Err(DeviceTreeError::InvalidMagicNumber);
        }
        let header = DeviceTreeHeader::from_bytes(header_bytes)?;
        let data =
            unsafe { core::slice::from_raw_parts(addr as *const u8, header.total_size as usize) };
        Self::from_bytes(data)
    }

    /// Its magic number extracted from the header
    pub fn magic(&self) -> usize {
        self.header.magic as usize
    }

    /// Its total size extracted from the header
    pub fn total_size(&self) -> usize {
        self.header.total_size as usize
    }

    /// Its offset of the struct region extracted from the header
    pub fn off_dt_struct(&self) -> usize {
        self.header.off_dt_struct as usize
    }

    /// Its offset of the strings region extracted from the header
    pub fn off_dt_strings(&self) -> usize {
        self.header.off_dt_strings as usize
    }

    /// Its offset of the reserved memory region extracted from the header
    pub fn off_mem_reserved(&self) -> usize {
        self.header.off_mem_reserved as usize
    }

    /// Its version extracted from the header
    pub fn version(&self) -> usize {
        self.header.version as usize
    }

    /// Its last compatible version extracted from the header
    pub fn last_comp_version(&self) -> usize {
        self.header.last_comp_version as usize
    }

    /// Its boot cpu id extracted from the header
    pub fn boot_cpu_id(&self) -> usize {
        self.header.boot_cpu_id as usize
    }

    /// Its size of the strings region extracted from the header
    pub fn size_dt_strings(&self) -> usize {
        self.header.size_dt_strings as usize
    }

    /// Its size of the struct region extracted from the header
    pub fn size_dt_struct(&self) -> usize {
        self.header.size_dt_struct as usize
    }

    /// Get a reference of the root node
    pub fn root(&self) -> &DeviceTreeNode {
        &self.root
    }

    /// Find the node by given node path
    pub fn find_node(&self, path: &str) -> Option<&DeviceTreeNode> {
        let mut slices = path.split('/');
        if let Some("") = slices.next() {
            let mut first = &self.root;
            for i in slices {
                if let Some(node) = first.find_child(i) {
                    first = node;
                } else {
                    return None;
                }
            }
            Some(first)
        } else {
            None
        }
    }

    /// Find the node by given node path with all the nodes traveled
    pub fn find_along_path(&self, path: &str) -> Option<Vec<&DeviceTreeNode>> {
        let mut slices: Vec<&str> = path.split('/').collect();
        let mut container = Vec::<&DeviceTreeNode>::new();
        if slices.len() > 0 && self.root.name() == slices[0] {
            container.push(&self.root);
            if Self::find_along_path_internal(&self.root, &mut slices, 1, &mut container) {
                Some(container)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn find_along_path_internal<'tree>(
        node: &'tree DeviceTreeNode,
        slices: &mut [&str],
        index: usize,
        container: &mut Vec<&'tree DeviceTreeNode>,
    ) -> bool {
        if index == slices.len() {
            return true;
        }
        let name = slices[index];
        for node in node.nodes() {
            if node.name() == name {
                container.push(node);
                return Self::find_along_path_internal(node, slices, index + 1, container);
            }
        }
        return false;
    }
}

/// Iterator for all the tree nodes
pub struct DeviceTreeNodeIter<'a> {
    queue: VecDeque<&'a DeviceTreeNode>,
}

impl<'a> Iterator for DeviceTreeNodeIter<'a> {
    type Item = &'a DeviceTreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.queue.pop_front();
        match res {
            Some(node) if node.has_children() => {
                for i in node.nodes() {
                    self.queue.push_back(i);
                }
            }
            _ => {}
        }
        res
    }
}

impl Display for DeviceTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{}", self.root)
    }
}

impl<'a> IntoIterator for &'a DeviceTree {
    type Item = &'a DeviceTreeNode;
    type IntoIter = DeviceTreeNodeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        DeviceTreeNodeIter {
            queue: VecDeque::from([self.root()]),
        }
    }
}

#[derive(Clone)]
pub(crate) struct InheritedValues(Vec<(String, u64)>);

impl InheritedValues {
    pub const fn new() -> Self {
        InheritedValues(vec![])
    }

    pub fn find(&self, name: &str) -> Option<u64> {
        for i in &self.0 {
            if i.0.as_str() == name {
                return Some(i.1);
            }
        }
        None
    }

    pub fn insert(&mut self, name: String, value: u64) {
        self.0.push((name, value));
    }
}
