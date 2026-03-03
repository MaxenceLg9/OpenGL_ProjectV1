use std::hash::Hash;

pub struct PUID {
    id : u32,
}

impl PUID {
    pub fn new(id : u32) -> PUID {
        Self { id }
    }
}

impl PartialEq<Self> for PUID {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for PUID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for PUID {}
