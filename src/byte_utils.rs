use alloc::vec::Vec;

pub(crate) const BLOCK_SIZE: usize = 4;

pub(crate) fn align_block(index: usize) -> usize {
    index / (BLOCK_SIZE)
}

pub(crate) fn locate_block(index: usize) -> usize {
    index * (BLOCK_SIZE)
}

pub(crate) fn align_size(raw_size: usize) -> usize {
    (raw_size + BLOCK_SIZE - 1) / BLOCK_SIZE
}

pub(crate) fn read_aligned_block(data: &[u8], index: usize) -> Option<[u8; BLOCK_SIZE]> {
    let first = locate_block(index);
    if first + BLOCK_SIZE > data.len() {
        None
    } else {
        Some([
            data[first],
            data[first + 1],
            data[first + 2],
            data[first + 3],
        ])
    }
}

pub(crate) fn read_aligned_be_u32(data: &[u8], index: usize) -> Option<u32> {
    read_aligned_block(data, index).map(|block| u32::from_be_bytes(block))
}

pub(crate) fn read_aligned_be_big_number(
    data: &[u8],
    index: usize,
    block_size: usize,
) -> Option<u128> {
    match block_size {
        0 | 1 | 2 => read_aligned_be_number(data, index, block_size).map(|f| f as u128),
        3 | 4 => {
            let mut num = 0u128;
            for i in 0..block_size {
                let bytes = read_aligned_block(data, index + i)?;
                let single = u32::from_be_bytes(bytes) as u128;
                num = (num << 32) + single;
            }
            Some(num)
        }
        _ => None,
    }
}

pub(crate) fn read_aligned_be_number(data: &[u8], index: usize, block_size: usize) -> Option<u64> {
    match block_size {
        0 => Some(0),
        1 => read_aligned_be_u32(data, index).map(|res| res as u64),
        2 => {
            let mut num = 0u64;
            for i in 0..block_size{
                let bytes = read_aligned_block(data, index + i)?;
                let single = u32::from_be_bytes(bytes) as u64;
                num = (num << 32) + single;
            }
            Some(num)
        }
        _ => None,
    }
}

pub(crate) fn read_name(data: &[u8], offset: usize) -> Option<&str> {
    let first = offset;
    if first > data.len() {
        None
    } else {
        let mut end = first;
        while data[end] != '\0' as u8 {
            end = end + 1;
        }
        match core::str::from_utf8(&data[first..end]) {
            Ok(s) => Some(s),
            _ => None,
        }
    }
}

pub(crate) fn read_aligned_name(data: &[u8], index: usize) -> Option<&str> {
    read_name(data, locate_block(index))
}

pub(crate) fn read_aligned_sized_strings(
    data: &[u8],
    index: usize,
    size: usize,
) -> Option<Vec<&str>> {
    let first = locate_block(index);
    if first + size > data.len() {
        None
    } else {
        let mut current = first;
        let mut last = current;
        let mut res = Vec::<&str>::new();
        while current < first + size {
            if data[current] == b'\0' {
                // collect
                let value = core::str::from_utf8(&data[last..current]).unwrap();
                res.push(value);
                last = current + 1;
            }
            current += 1;
        }
        Some(res)
    }
}
