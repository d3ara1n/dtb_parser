#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

//! # dtb_parser
//!
//! Parses a device tree blob to a human-friendly data structure.
//!
//! The no [std] but [alloc] library is required.

#[cfg(not(feature = "std"))]
extern crate alloc;

pub use device_tree::DeviceTree;

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
