use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};

use crate::byte_utils::{align_block, align_size, BLOCK_SIZE, locate_block, read_aligned_be_number, read_aligned_be_u32, read_aligned_sized_strings, read_name};
use crate::device_tree::InheritedValues;
use crate::error::DeviceTreeError::{NotEnoughLength, ParsingFailed};
use crate::error::Result;
use crate::header::DeviceTreeHeader;

/// Presenting a variety of values that a `NodeProperty` can hold
#[derive(Debug)]
pub enum PropertyValue<'a> {
    /// Empty value
    None,
    /// Single integer
    Integer(u64),
    /// A list of integers
    Integers(Vec<u64>),
    // u64 when interrupt-cells = 2
    /// A pointer referenced by `<specifier>-parent`
    PHandle(u32),
    /// Single string
    String(&'a str),
    /// A list of strings
    Strings(Vec<&'a str>),
    // address, size, size with 0 no size
    /// An address with it's length(size)
    Address(u64, u64),
    /// A list of addresses
    Addresses(Vec<(u64, u64)>),
    // child-bus-address, parent-bus-address, length
    /// A arbitrary number of addresses
    Ranges(Vec<(u64, u64, u64)>),
    /// Out of these varieties and cannot be parsed
    Unknown,
}

impl<'a> Display for PropertyValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            PropertyValue::None => write!(f, ""),
            PropertyValue::Integer(it) => write!(f, "<{:#x}>", it),
            PropertyValue::Integers(it) => write!(f, "<{}>", it.iter().map(|x| format!("{:#x}", x)).collect::<Vec<String>>().join(" ")),
            PropertyValue::String(it) => write!(f, "\"{}\"", it),
            PropertyValue::Strings(it) => write!(f, "\"[{}]\"", it.join("\",\"")),
            PropertyValue::PHandle(it) => write!(f, "<{:#x}>", it),
            PropertyValue::Address(address, size) => write!(f, "<{:#x} {:#x}>", address, size),
            PropertyValue::Addresses(it) => write!(f, "<{}>", it.iter().map(|(address, size)| format!("{:#x} {:#x}", address, size)).collect::<Vec<String>>().join(" ")),
            PropertyValue::Ranges(it) => write!(f, "<{}>", it.iter().map(|(child, parent, length)| format!("{:#x} {:#x} {:#x}", child, parent, length)).collect::<Vec<String>>().join(" ")),
            PropertyValue::Unknown => write!(f, "")
        }
    }
}

/// A property of `DeviceTreeNode`
pub struct NodeProperty<'a> {
    pub(crate) block_count: usize,
    name: &'a str,
    value: PropertyValue<'a>,
}

// it wont create value, node does
impl<'a> NodeProperty<'a> {
    pub(crate) fn from_bytes(data: &'a [u8], header: &DeviceTreeHeader, start: usize, inherited: &InheritedValues, owned: &InheritedValues) -> Result<NodeProperty<'a>> {
        let prop_block_start = align_block(start);
        if let Some(prop_val_size) = read_aligned_be_u32(data, prop_block_start + 1) {
            if let Some(name_offset) = read_aligned_be_u32(data, prop_block_start + 2)
            {
                if let Some(name) = read_name(data, (header.off_dt_strings + name_offset) as usize)
                {
                    let value_index = prop_block_start + 3;
                    // standard properties
                    if prop_val_size > 0 {
                        let raw_value = &data[locate_block(value_index)..(locate_block(value_index) + prop_val_size as usize)];
                        match NodeProperty::parse_value(raw_value, name, inherited, owned) {
                            Ok(value) => Ok(Self {
                                block_count: 3 + align_size(prop_val_size as usize),
                                name,
                                value,
                            }),
                            Err(err) => Err(err)
                        }
                    } else {
                        Ok(Self {
                            block_count: 3,
                            name,
                            value: PropertyValue::None,
                        })
                    }
                } else {
                    Err(NotEnoughLength)
                }
            } else {
                Err(NotEnoughLength)
            }
        } else {
            Err(NotEnoughLength)
        }
    }

    pub(crate) fn parse_value(raw_value: &'a [u8], name: &str, inherited: &InheritedValues, owned: &InheritedValues) -> Result<PropertyValue<'a>> {
        match name {
            "compatible" | "model" | "status" => {
                if let Some(strs) = read_aligned_sized_strings(raw_value, 0, raw_value.len()) {
                    if strs.len() > 1 {
                        Ok(PropertyValue::Strings(strs))
                    } else {
                        Ok(PropertyValue::String(strs[0]))
                    }
                } else {
                    Err(ParsingFailed)
                }
            }
            "phandle" | "virtual-reg" => {
                if let Some(int) = read_aligned_be_u32(raw_value, 0) {
                    Ok(PropertyValue::Integer(int as u64))
                } else {
                    Err(ParsingFailed)
                }
            }
            "reg" => {
                let address_cells = match inherited.find("#address-cells") {
                    Some(v) => v as usize,
                    _ => 2
                };
                let size_cells = match inherited.find("#size-cells") {
                    Some(v) => v as usize,
                    _ => 2
                };

                let group_size = align_size(raw_value.len()) / (address_cells + size_cells);
                if group_size > 1 {
                    let mut regs = Vec::<(u64, u64)>::new();
                    for i in 0..group_size {
                        let group_index = i * (address_cells + size_cells);
                        let res = (read_aligned_be_number(raw_value, group_index, address_cells).unwrap(), read_aligned_be_number(raw_value, group_index + address_cells, size_cells).unwrap());
                        regs.push(res);
                    }
                    Ok(PropertyValue::Addresses(regs))
                } else {
                    Ok(PropertyValue::Address(read_aligned_be_number(raw_value, 0, address_cells).unwrap(), read_aligned_be_number(raw_value, address_cells, size_cells).unwrap()))
                }
            }
            "ranges" | "dma-ranges" => {
                // TODO: make ranges fucking work
                let child_cells = match owned.find("#address-cells") {
                    Some(v) => v as usize,
                    _ => 2
                };
                let parent_cells = match inherited.find("#address-cells") {
                    Some(v) => v as usize,
                    _ => 2
                };
                let size_cells = match owned.find("#size-cells") {
                    Some(v) => v as usize,
                    _ => 2
                };
                let single_size = child_cells + parent_cells + size_cells;
                let group_size = align_size(raw_value.len()) / single_size;
                let mut rags = Vec::<(u64, u64, u64)>::new();
                for i in 0..group_size {
                    let group_index = i * single_size;
                    let res = (read_aligned_be_number(raw_value, group_index, child_cells).unwrap(), read_aligned_be_number(raw_value, group_index, parent_cells).unwrap(), read_aligned_be_number(raw_value, group_index, size_cells).unwrap());
                    rags.push(res);
                }
                Ok(PropertyValue::Ranges(rags))
            }
            x if x.ends_with("-parent") => {
                if let Some(int) = read_aligned_be_u32(raw_value, 0) {
                    Ok(PropertyValue::PHandle(int))
                } else {
                    Err(ParsingFailed)
                }
            }
            // nexus node's property
            x if x.starts_with("#") && x.ends_with("cells") => {
                if let Some(int) = read_aligned_be_u32(raw_value, 0) {
                    Ok(PropertyValue::Integer(int as u64))
                } else {
                    Err(ParsingFailed)
                }
            }
            _ => {
                let a = raw_value.len() as usize % BLOCK_SIZE == 0; // str or int | must str
                let b = raw_value[0] != b'\0' && raw_value[(raw_value.len() - 1) as usize] == b'\0'; // A then must str
                if !a || a && b {
                    // must be str
                    if let Some(strs) = read_aligned_sized_strings(raw_value, 0, raw_value.len()) {
                        if strs.len() > 1 {
                            Ok(PropertyValue::Strings(strs))
                        } else {
                            Ok(PropertyValue::String(strs[0]))
                        }
                    } else {
                        Err(ParsingFailed)
                    }
                } else {
                    // must be integer(s)
                    let size = raw_value.len() as usize / BLOCK_SIZE;
                    if size > 1 {
                        // integers
                        // TODO: interrupt-cells = 2
                        let mut res = Vec::<u64>::new();
                        for i in 0..size {
                            if let Some(num) = read_aligned_be_u32(raw_value, i) {
                                res.push(num as u64);
                            } else {
                                return Err(ParsingFailed);
                            }
                        }
                        Ok(PropertyValue::Integers(res))
                    } else {
                        Ok(PropertyValue::Integer(read_aligned_be_u32(raw_value, 0).unwrap() as u64))
                    }
                }
            }
        }
    }

    /// Get its name
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Get its value
    pub fn value(&self) -> &PropertyValue {
        &self.value
    }
}

impl<'a> Display for NodeProperty<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match &self.value {
            PropertyValue::Unknown | PropertyValue::None => write!(f, "{};", self.name),
            other => write!(f, "{} = {};", self.name, other)
        }
    }
}