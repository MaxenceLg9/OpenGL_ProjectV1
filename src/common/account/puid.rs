use std::fmt::{Display, Formatter};
use std::hash::Hash;

#[derive(Copy)]
pub struct PUID {
    id : u32,
}

impl PUID {
    pub fn new(id : u32) -> PUID {
        Self { id }
    }
    
    pub fn id(&self) -> u32 {
        self.id
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

impl Display for PUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"PUID : {}",self.id)
    }
}

impl Clone for PUID {
    fn clone(&self) -> Self {
        Self { id: self.id }
    }
}
