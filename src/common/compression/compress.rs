use bitvec::{order::Lsb0, vec::BitVec, view::BitView};
use zstd::compression_level_range;

use crate::{
    common::error::err_type::{CustomErrorKind, GameError},
    custom_error,
};

pub fn compress(data: Vec<u8>, level: i32) -> Result<Vec<u8>, GameError> {
    let level = if compression_level_range().contains(&level) {
        level
    } else {
        compression_level_range()
            .max()
            .ok_or_else(|| custom_error!(CustomErrorKind::Compression, "Cannot get compression level"))?
    };
    let compressed = zstd::bulk::compress(&data, level).map_err(|e| custom_error!(CustomErrorKind::Compression, e.to_string()))?;

    Ok(compressed)
}

pub fn decompress(compressed: Vec<u8>, len: u32) -> Result<Vec<u8>, GameError> {
    let data = zstd::bulk::decompress(&compressed, len as usize * 8usize).map_err(|e| custom_error!(CustomErrorKind::Compression, e.to_string()))?;

    Ok(data)
}
