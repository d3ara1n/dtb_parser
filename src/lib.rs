#![no_std]
#![warn(missing_docs)]

//! # dtb_parser
//!
//! Parses a device tree blob to a human-friendly data structure.
//!
//! The no [std] but [alloc] library is required.

pub use device_tree::DeviceTree;

extern crate alloc;

mod byte_utils;
mod header;

/// `DeviceTree`
pub mod device_tree;
/// `DeviceTreeError`
pub mod error;
/// `DeviceTreeNode`
pub mod node;
/// `NodeProperty`
pub mod prop;
/// Traits for the crate
pub mod traits;
