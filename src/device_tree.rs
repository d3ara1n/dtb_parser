use crate::error::{DeviceTreeError, Result};
use crate::header::DeviceTreeHeader;
use crate::node::DeviceTreeNode;

pub struct DeviceTree<'a> {
    header: DeviceTreeHeader,
    root: DeviceTreeNode<'a>,
}

impl<'a> DeviceTree<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Result<Self> {
        let magic = &data[0..4];
        if magic != [0xd0, 0x0d, 0xfe, 0xed] {
            return Err(DeviceTreeError::InvalidMagicNumber);
        }

        let header = match DeviceTreeHeader::from_bytes(data) {
            Ok(it) => it,
            Err(err) => return Err(err),
        };

        let root = match DeviceTreeNode::from_bytes(
            data,
            &header,
            header.off_dt_struct as usize,
            2,
            2,
        ) {
            Ok(it) => it,
            Err(err) => return Err(err),
        };

        Ok(Self { header, root })
    }

    pub fn magic(&self) -> usize {
        self.header.magic as usize
    }

    pub fn total_size(&self) -> usize {
        self.header.total_size as usize
    }

    pub fn off_dt_struct(&self) -> usize {
        self.header.off_dt_struct as usize
    }

    pub fn off_dt_strings(&self) -> usize {
        self.header.off_dt_strings as usize
    }

    pub fn off_mem_reserved(&self) -> usize {
        self.header.off_mem_reserved as usize
    }

    pub fn version(&self) -> usize {
        self.header.version as usize
    }

    pub fn last_comp_version(&self) -> usize {
        self.header.last_comp_version as usize
    }

    pub fn boot_cpu_id(&self) -> usize {
        self.header.boot_cpu_id as usize
    }

    pub fn size_dt_strings(&self) -> usize {
        self.header.size_dt_strings as usize
    }

    pub fn size_dt_struct(&self) -> usize {
        self.header.size_dt_struct as usize
    }

    pub fn root(&self) -> &DeviceTreeNode {
        &self.root
    }
}