pub type Result<T> = core::result::Result<T, DeviceTreeError>;

#[derive(Debug, Copy, Clone)]
pub enum DeviceTreeError {
    InvalidMagicNumber,
    NotEnoughLength,
    InvalidToken,
    ParsingFailed,
}