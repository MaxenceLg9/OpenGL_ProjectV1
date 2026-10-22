use std::fs;

use crate::{
    common::{
        error::err_type::{CustomErrorKind, GameError},
        file::path::Reader,
    },
    custom_error,
};

pub fn read_file_to_string<R: Reader>(reader: &R) -> Result<String, GameError> {
    let path = reader.get_path();
    fs::read_to_string(&path).map_err(|e| custom_error!(CustomErrorKind::Io, format!("Cannot open file {}, {}", path.display(), e)))
}
