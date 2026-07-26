use strum::FromRepr;

#[derive(Clone,Copy,Eq, PartialEq, FromRepr)]
#[repr(u16)]
pub enum BlockType {
    AIR = 0,
    DIRT = 1,
    STONE = 2,
    DEEPSLATE = 3,
    PlatinumOre = 4,
    IKRINEBLOCK = 5,
    IkrineOre = 6,
    StoneBricks = 7,
    GRASS = 8,
}

pub enum BlockLevel {
    DIAMOND,
    IRON,
    COBALT,
    STONE,
    WOOD
}

impl BlockType {
    pub fn get_value(self) -> u16 {
        let value = self as u16;
        value
    }
}