#![no_std]
#![warn(missing_docs)]

//! # dtb_parser
//!
//! Parses a device tree blob to a human-friendly data structure.
//!
//! The no `std` but `alloc` library is required.

extern crate alloc;

mod byte_utils;
mod header;

/// `DeviceTree`
pub mod device_tree;
/// `DeviceTreeError`
pub mod error;
/// `NodeProperty`
pub mod prop;
/// `DeviceTreeNode`
pub mod node;
/// Traits for the crate
pub mod traits;