use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

pub trait Reader: Display {
    fn get_path(&self) -> &PathBuf;
}

pub struct AssetsReader {
    path: PathBuf,
}

impl AssetsReader {
    pub fn new(str: &str) -> AssetsReader {
        let mut path = PathBuf::from("src/assets/");
        path.push(str);
        AssetsReader { path }
    }
}

impl Reader for AssetsReader {
    fn get_path(&self) -> &PathBuf {
        &self.path
    }
}

impl Display for AssetsReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path.fmt(f)
    }
}
