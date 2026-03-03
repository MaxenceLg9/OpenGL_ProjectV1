pub enum BlockType {
    AIR = 0,
    DIRT = 1,
    STONE = 2,
    DEEPSLATE = 3,
    PlatinumOre = 4,
    IkrineBlock = 5,
    IkrineOre = 6,
    StoneBricks = 7
}

impl BlockType {
    pub fn get_value(self) -> u16 {
        let value = self as u16;
        value
    }
}