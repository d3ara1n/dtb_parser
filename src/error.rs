/// Result for `DeviceTree` parsing
pub type Result<T> = core::result::Result<T, DeviceTreeError>;

/// Errors for `DeviceTree` parsing
#[derive(Debug, Copy, Clone)]
pub enum DeviceTreeError {
    /// Wrong magic number
    InvalidMagicNumber,
    /// Data are too short for parsing
    NotEnoughLength,
    /// Wrong token number
    InvalidToken,
    /// Read bytes failed
    ParsingFailed,
    /// Memory cannot be accessed
    MemoryAccessFailed,
}