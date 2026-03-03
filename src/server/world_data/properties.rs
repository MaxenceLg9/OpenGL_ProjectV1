pub struct ServerWorldProperties {
    difficulty : Difficulty,
    name : String,
}

pub enum Difficulty {
    Easy,
    Medium,
    Hard
}

impl ServerWorldProperties {
    pub fn new(name : String, difficulty: Difficulty) -> Self {
        Self {
            difficulty,
            name,
        }
    }
}