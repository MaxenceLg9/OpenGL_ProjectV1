use mlua::{FromLua, Lua, ObjectLike, UserData, UserDataMethods, Value};
use std::collections::hash_map::{Entry, Iter};
use std::fmt::{Display, Formatter, Write};

#[derive(Clone)]
pub struct BlockState {
    tier: BlockTier,
}

pub type BlockTier = u8;

impl BlockState {
    pub fn new(tier: BlockTier, is_solid: bool) -> Self {
        Self { tier }
    }
}
impl UserData for BlockState {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // &self, not &mut self -> use add_method, not add_method_mut
        methods.add_function("new", |_lua, (tier, is_solid): (BlockTier, bool)| Ok(BlockState::new(tier, is_solid)));
    }
}

impl FromLua for BlockState {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.take::<BlockState>()?),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "BlockState".to_string(),
                message: Some("expected userdata".to_string()),
            }),
        }
    }
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type BlockId = u16;

pub struct BlocksRegistry {
    map: HashMap<String, BlockId>,
}

impl BlocksRegistry {
    pub fn new() -> BlocksRegistry {
        Self { map: HashMap::new() }
    }

    pub fn get(&self, name: &str) -> Option<BlockId> {
        self.map.get(name).copied()
    }

    pub fn register(&mut self, name: &str, id: BlockId) {
        self.map.insert(name.to_string(), id);
    }

    pub fn iter(&self) -> Iter<'_, String, BlockId> {
        self.map.iter()
    }
}
pub struct BlocksRegistries {
    list: Vec<BlockState>, // index = BlockId
    map: HashMap<String, u16>,
}

impl BlocksRegistries {
    fn new() -> Self {
        Self { list: Vec::new(), map: HashMap::new() }
    }

    fn register(&mut self, register: &str, state: BlockState, name: &str) -> BlockId {
        let id = self.list.len() as BlockId;
        self.map.insert(format!("{}:{}", register, name), id);
        self.list.push(state);
        id
    }

    pub fn get_state(&self, id: BlockId) -> &BlockState {
        &self.list[id as usize]
    }

    pub fn get_by_name(&self, name: &str) -> Option<BlockId> {
        self.map.get(name).copied()
    }

    // pub fn get_list(&self) -> HashMap<String, BlockId> {
    //     self.map.clone()
    // }

    pub fn from_map(mut map: HashMap<String, HashMap<String, BlockState>>) -> Self {
        let mut registry = Self::new();
        let air = map.get_mut("minecraft").unwrap().remove("air").unwrap();
        registry.register("minecraft", air, "air");
        for (register_name, register) in map {
            for (block, state) in register {
                registry.register(&register_name, state, &block);
            }
        }
        registry
    }
    pub fn iter(&self) -> Iter<'_, String, u16> {
        self.map.iter()
    }
}
pub struct Registry<T, U, V> {
    list: Vec<T>,
    map: HashMap<U, V>,
}
