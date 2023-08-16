use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};

use crate::error::{DeviceTreeError, Result};
use crate::header::DeviceTreeHeader;
use crate::node::DeviceTreeNode;
use crate::traits::HasNamedChildNode;

/// The tree structure
/// Reads data from a slice of bytes and parses into [DeviceTree]
/// Indexed by nodes and properties' names or by path for the whole tree
pub struct DeviceTree<'a> {
    header: DeviceTreeHeader,
    root: DeviceTreeNode<'a>,
}

impl<'a> DeviceTree<'a> {
    /// Parses a slice of bytes and constructs [DeviceTree]
    /// The structure should live as long as the `data`
    pub fn from_bytes(data: &'a [u8]) -> Result<Self> {
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
            InheritedValues::new(),
        )?;

        Ok(Self { header, root })
    }

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
                }
            }
            Some(first)
        } else {
            None
        }
    }
}

/// Iterator for all the tree nodes
pub struct DeviceTreeNodeIter<'a> {
    queue: VecDeque<&'a DeviceTreeNode<'a>>,
}

impl<'a> Iterator for DeviceTreeNodeIter<'a> {
    type Item = &'a DeviceTreeNode<'a>;

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

impl<'a> Display for DeviceTree<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{}", self.root)
    }
}

impl<'a> IntoIterator for &'a DeviceTree<'a> {
    type Item = &'a DeviceTreeNode<'a>;
    type IntoIter = DeviceTreeNodeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        DeviceTreeNodeIter {
            queue: VecDeque::from([self.root()]),
        }
    }
}

#[derive(Clone)]
pub(crate) struct InheritedValues<'a>(Vec<(&'a str, u64)>);

impl<'a> InheritedValues<'a> {
    pub const fn new() -> InheritedValues<'a> {
        InheritedValues(vec![])
    }

    pub fn find(&self, name: &str) -> Option<u64> {
        for i in &self.0 {
            if i.0 == name {
                return Some(i.1);
            }
        }
        None
    }

    pub fn update(&mut self, name: &'a str, value: u64) {
        let mut dirty = false;
        for i in 0..self.0.len() {
            if self.0[i].0 == name {
                self.0[i].1 = value;
                dirty = true;
                break;
            }
        }
        if !dirty {
            self.0.push((name, value));
        }
    }
}
