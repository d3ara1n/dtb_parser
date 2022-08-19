use alloc::vec::Vec;

use crate::byte_utils::{align_block, align_size, BLOCK_SIZE, locate_block, read_aligned_be_number, read_aligned_be_u32, read_aligned_sized_strings, read_name};
use crate::error::{DeviceTreeError, Result};
use crate::error::DeviceTreeError::ParsingFailed;
use crate::header::DeviceTreeHeader;

#[derive(Debug)]
pub enum PropertyValue<'a> {
    None,
    Integer(u64),
    Integers(Vec<u64>), // u64 when interrupt-cells = 2
    PHandle(u32),
    String(&'a str),
    Strings(Vec<&'a str>),
    // address, size, size with 0 no size
    Address(u64, u64),
    Addresses(Vec<(u64, u64)>),
    Range(u64, u64, u64),
    Ranges(Vec<(u64,u64,u64)>),
    Unknown,
}

pub struct NodeProperty<'a>  {
    pub(crate) block_count: usize,
    name: &'a str,
    value: PropertyValue<'a>,
}

impl<'dt, 'a> NodeProperty<'a> {
    pub fn from_bytes(data: &'a [u8], header: &DeviceTreeHeader, start: usize, address_cells: usize, size_cells: usize) -> Result<NodeProperty<'a>> {
        let prop_block_start = align_block(start);
        if let Some(prop_val_size) = read_aligned_be_u32(data, prop_block_start + 1) {
            if let Some(name_offset) = read_aligned_be_u32(data, prop_block_start + 2)
            {
                if let Some(name) = read_name(data, (header.off_dt_strings + name_offset) as usize)
                {
                    let value_index = prop_block_start + 3;
                    // standard properties
                    let value = if prop_val_size > 0 {
                        match name {
                            "compatible" | "model" | "status" => {
                                if let Some(strs) = read_aligned_sized_strings(data, value_index, prop_val_size as usize) {
                                    if strs.len() > 1 {
                                        PropertyValue::Strings(strs)
                                    } else {
                                        PropertyValue::String(strs[0])
                                    }
                                } else {
                                    return Err(ParsingFailed);
                                }
                            }
                            "#address-cells" | "#size-cells" | "virtual-reg" => {
                                if let Some(int) = read_aligned_be_u32(data, value_index) {
                                    PropertyValue::Integer(int as u64)
                                } else {
                                    return Err(ParsingFailed);
                                }
                            },
                            "phandle" => {
                                if let Some(phandle) = read_aligned_be_u32(data, value_index) {
                                    PropertyValue::PHandle(phandle)
                                } else {
                                    return Err(ParsingFailed);
                                }
                            }
                            "reg" => {
                                let group_size = align_size(prop_val_size as usize) / (address_cells + size_cells);
                                if group_size > 1 {
                                    let mut regs = Vec::<(u64, u64)>::new();
                                    for i in 0..group_size {
                                        let group_index = value_index + i * (address_cells + size_cells);
                                        let res = (read_aligned_be_number(data, group_index, address_cells).unwrap(), read_aligned_be_number(data, group_index + address_cells, size_cells).unwrap());
                                        regs.push(res);
                                    }
                                    PropertyValue::Addresses(regs)
                                } else {
                                    PropertyValue::Address(read_aligned_be_number(data, value_index, address_cells).unwrap(), read_aligned_be_number(data, value_index + address_cells, size_cells).unwrap())
                                }
                            }
                            "ranges" | "dma-ranges" => {
                                // TODO: make ranges fucking work
                                PropertyValue::String("it's easy to get but u wont need this so i delete from the source")
                            }
                            _ => {
                                let raw_value = &data[locate_block(value_index)..(locate_block(value_index) + prop_val_size as usize)];
                                let a = prop_val_size as usize % BLOCK_SIZE == 0; // str or int | must str
                                let b = raw_value[0] != b'\0' && raw_value[(prop_val_size - 1) as usize] == b'\0'; // A then must str
                                if !a || a && b {
                                    // must be str
                                    if let Some(strs) = read_aligned_sized_strings(raw_value, 0, prop_val_size as usize) {
                                        if strs.len() > 1 {
                                            PropertyValue::Strings(strs)
                                        } else {
                                            PropertyValue::String(strs[0])
                                        }
                                    } else {
                                        return Err(ParsingFailed);
                                    }
                                } else {
                                    // must be integer(s)
                                    let size = prop_val_size as usize / BLOCK_SIZE;
                                    if size > 1 {
                                        // integers
                                        // TODO: interrupt-cells = 2
                                        let mut res = Vec::<u64>::new();
                                        for i in 0..size{
                                            if let Some(num) = read_aligned_be_u32(raw_value, i){
                                                res.push(num as u64);
                                            }else{
                                                return Err(ParsingFailed);
                                            }
                                        }
                                        PropertyValue::Integers(res)
                                    } else {
                                        PropertyValue::Integer(read_aligned_be_u32(raw_value, 0).unwrap() as u64)
                                    }
                                }
                            }
                        }
                    } else {
                        PropertyValue::None
                    };
                    Ok(Self {
                        block_count: 3 + align_size(prop_val_size as usize),
                        name,
                        value,
                    })
                } else {
                    Err(DeviceTreeError::NotEnoughLength)
                }
            } else {
                Err(DeviceTreeError::NotEnoughLength)
            }
        } else {
            Err(DeviceTreeError::NotEnoughLength)
        }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn value(&self) -> &PropertyValue {
        &self.value
    }
}