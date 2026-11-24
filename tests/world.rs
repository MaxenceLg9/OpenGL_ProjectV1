//! Integration tests for the world model of the `shared` library:
//! positions, chunks, chunk map, block registry, assets and world generation.

use mlua::Lua;
use shared::common::compression::compress;
use shared::common::world::block::block::{BlockState, BlockTier, BlocksRegistries};
use shared::common::world::chunk::chunk::Chunk;
use shared::common::world::chunk::chunkmap::{ChunkMap, MAX_Y_CHUNK, MIN_Y_CHUNK};
use shared::common::world::pos::blockpos::BlockPos;
use shared::common::world::pos::chunkpos::{CHUNK_SIZE, ChunkPos};
use shared::common::world::pos::iblockpos::IBlockPos;
use shared::common::world::pos::pos_trait::PosTrait;
use shared::worldgen::Generator;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

const C: i32 = CHUNK_SIZE as i32;

fn empty_chunk(pos: ChunkPos) -> Chunk {
    Chunk::new(pos, Arc::new(vec![0; CHUNK_SIZE.pow(3)]))
}

/// Loads `blocks.lua` exactly like the client and the server do.
fn load_registry() -> BlocksRegistries {
    let lua = Lua::new();
    let env = lua.globals();
    let table = lua.create_table().unwrap();
    table
        .set(
            "new",
            lua.create_function(|_, (tier, is_solid): (BlockTier, bool)| Ok(BlockState::new(tier, is_solid)))
                .unwrap(),
        )
        .unwrap();
    env.set("BlockState", table).unwrap();
    let source = std::fs::read_to_string("src/assets/lua/blocks.lua").expect("cannot read blocks.lua");
    let chunk = lua.load(source).set_environment(env).into_function().unwrap();
    BlocksRegistries::from_map(chunk.call::<HashMap<String, HashMap<String, BlockState>>>(()).unwrap())
}

// ------------------------------------------------------------------ positions

#[test]
fn iblockpos_splits_negative_coordinates_with_euclidean_division() {
    let (relative, chunk) = IBlockPos::new(-1, 0, C + 1).as_split_pos();
    assert_eq!(relative, IBlockPos::new(C - 1, 0, 1));
    assert_eq!(chunk, ChunkPos::new(-1, 0, 1));

    let (relative, chunk) = IBlockPos::new(-C, -C - 1, 2 * C - 1).as_split_pos();
    assert_eq!(relative, IBlockPos::new(0, C - 1, C - 1));
    assert_eq!(chunk, ChunkPos::new(-1, -2, 1));
}

#[test]
fn chunkpos_plus_relative_pos_gives_back_the_absolute_pos() {
    for absolute in [IBlockPos::new(0, 0, 0), IBlockPos::new(-1, -1, -1), IBlockPos::new(-140, 108, 476), IBlockPos::new(1000, -250, -64)] {
        let (relative, chunk) = absolute.as_split_pos();
        assert_eq!(chunk + relative, absolute);
    }
}

#[test]
fn block_index_is_x_major_then_y_then_z() {
    assert_eq!(IBlockPos::new(0, 0, 0).get_index(), 0);
    assert_eq!(IBlockPos::new(0, 0, 1).get_index(), 1);
    assert_eq!(IBlockPos::new(0, 1, 0).get_index(), CHUNK_SIZE);
    assert_eq!(IBlockPos::new(1, 0, 0).get_index(), CHUNK_SIZE * CHUNK_SIZE);
    assert_eq!(IBlockPos::new(C - 1, C - 1, C - 1).get_index(), CHUNK_SIZE.pow(3) - 1);
}

#[test]
fn blockpos_floors_towards_negative_infinity() {
    assert_eq!(BlockPos::new(0.5, 1.99, -0.5).as_ivec3(), glam::ivec3(0, 1, -1));
    assert_eq!(BlockPos::new(-1.0, -64.01, 63.9).as_ivec3(), glam::ivec3(-1, -65, 63));

    assert_eq!(BlockPos::new(-0.1, 0.0, 63.9).get_chunk_pos(), ChunkPos::new(-1, 0, 0));
    assert_eq!(BlockPos::new(-0.1, 0.0, 64.0).get_chunk_pos(), ChunkPos::new(-1, 0, 1));
    assert_eq!(BlockPos::new(-0.5, 5.5, 70.2).get_relative_block_pos(), IBlockPos::new(C - 1, 5, 6));
}

#[test]
fn chunk_center_is_in_the_middle_of_the_chunk() {
    assert_eq!(ChunkPos::new(0, 0, 0).center().as_vec3(), glam::vec3(32.0, 32.0, 32.0));
    assert_eq!(ChunkPos::new(-1, 2, 0).center().as_vec3(), glam::vec3(-32.0, 160.0, 32.0));
}

#[test]
fn positions_serialize_round_trip() {
    let chunk_pos = ChunkPos::new(-7, MIN_Y_CHUNK, i32::MAX);
    assert_eq!(ChunkPos::deserialize(chunk_pos.serialize().into_vec()), chunk_pos);

    let iblock_pos = IBlockPos::new(i32::MIN, 0, -1);
    assert_eq!(IBlockPos::deserialize(iblock_pos.serialize().into_vec()), iblock_pos);

    let block_pos = BlockPos::new(-0.125, 1e-3, 123456.5);
    assert_eq!(BlockPos::deserialize(block_pos.serialize().into_vec()), block_pos);
}

// --------------------------------------------------------------------- chunks

#[test]
fn chunk_set_block_reports_whether_something_changed() {
    let pos = ChunkPos::new(1, 0, -1);
    let mut chunk = empty_chunk(pos);
    let block = pos + IBlockPos::new(3, 4, 5);

    assert!(chunk.set_block(block, 7));
    assert_eq!(chunk.get_block_at(IBlockPos::new(3, 4, 5)), 7);
    assert!(!chunk.set_block(block, 7), "setting the same block twice is not a change");
}

#[test]
fn chunk_out_of_bounds_reads_are_air() {
    let chunk = Chunk::new(ChunkPos::new(0, 0, 0), Arc::new(vec![1; CHUNK_SIZE.pow(3)]));
    for pos in [IBlockPos::new(-1, 0, 0), IBlockPos::new(0, C, 0), IBlockPos::new(0, 0, C + 10)] {
        assert_eq!(chunk.get_block_at(pos), 0);
    }
}

#[test]
fn chunk_edits_are_copy_on_write() {
    // the mesher threads hold `Arc` clones of the blocks: editing must not mutate their snapshot
    let mut chunk = empty_chunk(ChunkPos::new(0, 0, 0));
    let snapshot = chunk.get_blocks();
    chunk.set_block(IBlockPos::new(1, 1, 1), 9);
    assert_eq!(snapshot[IBlockPos::new(1, 1, 1).get_index()], 0);
    assert_eq!(chunk.get_block_at(IBlockPos::new(1, 1, 1)), 9);
}

#[test]
fn chunk_serializes_blocks_as_little_endian_u16() {
    let mut blocks = vec![0u16; CHUNK_SIZE.pow(3)];
    blocks[0] = 0x0102;
    let bytes = Chunk::new(ChunkPos::new(0, 0, 0), Arc::new(blocks)).serialize();
    assert_eq!(bytes.len(), CHUNK_SIZE.pow(3) * 2);
    assert_eq!(&bytes[0..2], &[0x02, 0x01]);
}

// ------------------------------------------------------------------ chunk map

#[test]
fn neighbours_are_the_27_chunks_of_the_surrounding_cube() {
    let center = ChunkPos::new(5, -2, -9);
    let neighbours = ChunkMap::get_neighbours_chunks_pos(center);
    let unique: HashSet<_> = neighbours.iter().copied().collect();

    assert_eq!(neighbours.len(), 27);
    assert_eq!(unique.len(), 27);
    assert!(unique.contains(&center));
    assert!(neighbours.iter().all(|p| (*p - center).abs().max_element() <= 1));
}

#[test]
fn chunk_map_only_stores_chunks_inside_the_world_height() {
    let mut map = ChunkMap::new();
    for y in [MIN_Y_CHUNK - 1, MAX_Y_CHUNK + 1] {
        map.add_chunk(empty_chunk(ChunkPos::new(0, y, 0)));
        assert!(!map.contains_chunk(&ChunkPos::new(0, y, 0)));
    }
    for y in [MIN_Y_CHUNK, MAX_Y_CHUNK] {
        map.add_chunk(empty_chunk(ChunkPos::new(0, y, 0)));
        assert!(map.contains_chunk(&ChunkPos::new(0, y, 0)));
    }
    assert_eq!(map.len(), 2);
}

#[test]
fn chunk_map_add_chunk_returns_true_when_the_chunk_is_stored() {
    // `ClientChunkMap::add_chunk` (full-client) relies on this to queue the chunk for meshing
    let mut map = ChunkMap::new();
    assert!(map.add_chunk(empty_chunk(ChunkPos::new(0, 0, 0))), "a new chunk was stored");
    assert!(!map.add_chunk(empty_chunk(ChunkPos::new(0, MAX_Y_CHUNK + 1, 0))), "an out of world chunk was rejected");
}

#[test]
fn chunk_map_reads_and_writes_blocks_with_absolute_positions() {
    let mut map = ChunkMap::new();
    map.add_chunk(empty_chunk(ChunkPos::new(-1, 0, 0)));

    let block = IBlockPos::new(-1, 10, 20);
    map.set_block(block, 3);
    assert_eq!(map.get_block_at(block), 3);
    assert_eq!(map.get_chunk(&ChunkPos::new(-1, 0, 0)).unwrap().get_block_at(IBlockPos::new(C - 1, 10, 20)), 3);

    assert_eq!(map.get_block_at(IBlockPos::new(500, 10, 20)), 0, "missing chunks read as air");

    map.remove_chunk(&ChunkPos::new(-1, 0, 0));
    assert_eq!(map.get_block_at(block), 0);
}

// ----------------------------------------------------------- registry & assets

#[test]
fn air_is_always_block_id_zero() {
    let registry = load_registry();
    assert_eq!(registry.get_by_name("minecraft:air"), Some(0));
}

#[test]
fn every_block_has_a_unique_id() {
    let registry = load_registry();
    let ids: Vec<u16> = registry.iter().map(|(_, id)| *id).collect();
    let unique: HashSet<u16> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len());
    // ids are dense: they index the texture array and the state list
    assert_eq!(unique, (0..ids.len() as u16).collect());
}

#[test]
fn every_registered_block_has_a_texture() {
    // the client loads `textures/blocks/<name>.png` for each entry of the server's block table
    for (name, _) in load_registry().iter() {
        let short = name.split(':').nth(1).expect("block names are `namespace:name`");
        if short == "air" {
            continue;
        }
        let path = format!("src/assets/textures/blocks/{short}.png");
        assert!(Path::new(&path).exists(), "missing texture {path} for {name}");
    }
}

#[test]
fn compression_round_trip() {
    let data: Vec<u8> = (0..10_000u32).map(|i| (i % 7) as u8).collect();
    let compressed = compress::compress(data.clone(), 10).unwrap();
    assert!(compressed.len() < data.len());
    assert_eq!(compress::decompress(compressed, data.len() as u32).unwrap(), data);
}

// ------------------------------------------------------------- world generation

#[test]
fn terrain_generation_is_deterministic_for_a_seed() {
    let registries = Arc::new(HashMap::from([("blocks".to_owned(), load_registry())]));
    let a = Generator::new(1, registries.clone());
    let b = Generator::new(1, registries.clone());
    let other_seed = Generator::new(2, registries);

    let points = [(0, 0), (-500, 1234), (40_000, -7)];
    for (x, z) in points {
        assert_eq!(a.get_terrain_height(x, z), b.get_terrain_height(x, z));
    }
    assert!(points.iter().any(|&(x, z)| a.get_terrain_height(x, z) != other_seed.get_terrain_height(x, z)));
}

#[test]
fn generator_only_produces_registered_blocks() {
    let registry = load_registry();
    let known: HashSet<u16> = registry.iter().map(|(_, id)| *id).collect();
    let generator = Generator::new(1, Arc::new(HashMap::from([("blocks".to_owned(), registry)])));

    for x in (-64..64).step_by(16) {
        for z in (-64..64).step_by(16) {
            let height = generator.get_terrain_height(x, z);
            for y in (MIN_Y_CHUNK * C..=MAX_Y_CHUNK * C).step_by(32) {
                let block = generator.density_of(x as f64, y as f64, z as f64, height, 0.4, 1.5);
                assert!(block.is_some_and(|id| known.contains(&id)), "unknown block {block:?} at {x} {y} {z}");
            }
        }
    }
}
