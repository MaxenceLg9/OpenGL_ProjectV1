use std::borrow::Cow;

#[repr(u8)]
#[derive(Debug)]
pub enum CustomErrorKind {
    Io,
    Other,
    NotFound,
    InvalidData,
    NulError,
    KeyNotExists,
    Compression,
    ReceiveError,
    NetworkSocket,
    SendError,
    Lua,
}
#[macro_export]
macro_rules! custom_error {
    ($kind:expr, $descriptor:expr) => {
        GameError::new($kind, format!("{}:{}:{}, {}", file!(), line!(), column!(), $descriptor))
    };
}

pub struct GameError {
    kind: CustomErrorKind,
    descriptor: Cow<'static, str>,
}

impl GameError {
    pub fn new(kind: CustomErrorKind, descriptor: impl Into<Cow<'static, str>>) -> Self {
        Self::create(kind, descriptor)
    }

    fn create(kind: CustomErrorKind, descriptor: impl Into<Cow<'static, str>>) -> Self {
        Self { descriptor: descriptor.into(), kind }
    }
}

// impl From<std::io::Error> for CustomError {
//     fn from(value: std::io::Error) -> Self {
//         custom_error!(CustomErrorKind::Io, "", value)
//     }
// }

impl From<std::ffi::NulError> for GameError {
    fn from(value: std::ffi::NulError) -> Self {
        custom_error!(CustomErrorKind::NulError, value.to_string())
        // Self::new_sourced(CustomErrorKind::NulError, "", value)
    }
}

impl From<std::io::Error> for GameError {
    fn from(value: std::io::Error) -> Self {
        custom_error!(CustomErrorKind::Io, value.to_string())
    }
}

impl From<mlua::Error> for GameError {
    fn from(value: mlua::Error) -> Self {
        custom_error!(CustomErrorKind::Lua, value.to_string())
        // Self::new_sourced(CustomErrorKind::NulError, "", value)
    }
}

impl std::error::Error for GameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Debug for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&self.to_string()).finish()
    }
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error CustomErrorKind::{:?} ", self.kind))?;
        f.write_fmt(format_args!("At {}", self.descriptor))

        // f.write_fmt(format_args!("{}\n\n", self.context))
    }
}
