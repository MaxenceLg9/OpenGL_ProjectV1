use crate::print_base;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::{mpsc} ;
use std::time::Instant;
use glam;
use winit::window::Window;
use crate::display::renderer::mesh::{chunk_mesh::*, texture::texture_array::TextureArray, shader::shader::Shader};
use crate::game::world::chunk::block::block::BlockType;
use crate::print_debug;
use super::{chunk::chunk::*, player::player::PlayerUser};

pub const WORLD_SIZE : u32 = 8;
pub const WORLD_THREADS : u32 = 16;

pub struct World {
    world_data: Arc<RwLock<WorldData>>,
    chunk_receiver: mpsc::Receiver<Chunk>,
    chunk_sender: mpsc::Sender<Chunk>,
    pos_sender: mpsc::Sender<glam::IVec3>,
    pos_receiver: mpsc::Receiver<glam::IVec3>,
    mesh_receiver: mpsc::Receiver<PackedData>,
    mesh_sender: mpsc::Sender<PackedData>,
}

pub struct WorldData{
    chunks: HashMap<glam::IVec3, Arc<Chunk>>,
    meshes: HashMap<glam::IVec3, Chunk>,
    is_building: AtomicBool,
    is_generating: AtomicBool,
    texture_array: TextureArray,
    chunk_shader: Shader,
}

struct PackedData {
    pos: glam::IVec3,
    mesh : Chunk,
}

impl World {

    pub unsafe fn new() -> World {
        let texture_array = TextureArray::new("textures".to_string());
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/stone.png", BlockType::STONE.get_value() - 1).expect("Cannot add block to texture array");
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/dirt.png", BlockType::DIRT.get_value() - 1).expect("Cannot add block to texture array");
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/deepslate.png", BlockType::DEEPSLATE.get_value() - 1).expect("Cannot add block to texture array");

        let (ctx, crx) = mpsc::channel();
        let (mtx, mrx) = mpsc::channel();
        let (ptx, prx) = mpsc::channel();

        let world = Self {
            world_data: Arc::new(RwLock::new(WorldData::new(texture_array))),
            chunk_receiver: crx,
            chunk_sender: ctx,
            mesh_receiver: mrx,
            mesh_sender: mtx,
            pos_receiver: prx,
            pos_sender: ptx,
        };
        // world_data.create_chunks(glam::IVec3::new(1,1,1));
        world.world_data.read().unwrap().use_shader();
        world
    }

    pub fn get_data(&self) -> Arc<RwLock<WorldData>> {
        self.world_data.clone()
    }

    pub fn create_chunks(&mut self, player_pos : glam::IVec3) {
        // checking if a thread isn't already running
        if self.world_data.read().unwrap().is_generating.load(Ordering::Relaxed) {
            print_debug!("World is generating {}", self.world_data.read().unwrap().is_generating.load(Ordering::Relaxed));
            return;
        }
        // if not running already, locking it
        self.world_data.write().unwrap().is_generating.store(true, Ordering::Relaxed);

        let data = self.get_data().clone();
        let tx = self.chunk_sender.clone();
        std::thread::Builder::new().name("generator_thread".to_string()).spawn(move || {
            let chunks_to_build = Self::get_chunks_to_build(data.clone(),player_pos / CHUNK_SIZE as i32);
            let len = chunks_to_build.len() as u32;
            if len != 0 {
                Self::threads_to_build(chunks_to_build, len, tx.clone());
            }
            print_debug!("Thread finished, releasing is_generating lock");
            data.clone().write().unwrap().is_generating.store(false, Ordering::Relaxed);
        }).unwrap();
    }

    fn threads_to_build(chunks_to_build: Vec<glam::IVec3>, len: u32, tx : mpsc::Sender<Chunk>) {
        let arc_chunks_to_build = Arc::new(chunks_to_build);
        print_debug!("Running threads to generate {} chunks", arc_chunks_to_build.len());
        let mut handles = Vec::new();
        let mut n = 0;
        for i in 0..WORLD_THREADS {
            let tx_thread = tx.clone();
            let chunks_per_thread = ((len + WORLD_THREADS - 1) / WORLD_THREADS).max(4);
            let start = chunks_per_thread * i;
            let end = (chunks_per_thread * (i + 1)).min(len);
            let arc = arc_chunks_to_build.clone();
            handles.push(
                std::thread::Builder::new().name("child_generator_thread".to_string()).spawn(move || {
                    World::generate_chunks(start, end, tx_thread, arc);
                }).unwrap());
            n += 1;
            if end == len {
                break;
            }
        }
        print_debug!("Launched {} threads",n);
        for handle in handles {
            handle.join().expect("Cannot join thread");
        }
    }

    fn get_chunks_to_build(world_data: Arc<RwLock<WorldData>>, player_ipos: glam::IVec3) -> Vec<glam::IVec3> {
        let mut chunks_to_build: Vec<glam::IVec3> = Vec::new();
        // iterating over the chunks surrounding the player to see if a chunk hasn't been generated yet
        for x in -(WORLD_SIZE as i32)..WORLD_SIZE as i32 {
            for z in -(WORLD_SIZE as i32)..WORLD_SIZE as i32 {
                for y in 0..WORLD_SIZE as i32 {
                    // computing the relative chunk position based on the player
                    let mut chunk_pos = player_ipos.clone();
                    chunk_pos.z = chunk_pos.z + z;
                    chunk_pos.x = chunk_pos.x + x;
                    chunk_pos.y = y;
                    // if the chunk doesn't exist, adding it
                    if !world_data.read().unwrap().chunks.contains_key(&chunk_pos) {
                        print_base!("chunk {} doesn't exist, to build ", chunk_pos);
                        chunks_to_build.push(chunk_pos);
                    }
                }
            }
        }
        chunks_to_build
    }

    pub fn generate_chunks(start: u32, end : u32, chunk_sender: mpsc::Sender<Chunk>, chunks_to_build : Arc<Vec<glam::IVec3>>) {
        let time : Instant = Instant::now();
        // iterating from start to end indexes in the chunks_to_build vector
        let mut n = 0;
        for index in start..end {
            // sending the created chunk into thempsc::Sender
            chunk_sender.send(Chunk::new(chunks_to_build.get(index as usize).unwrap().clone())).expect("Error when sending chunk");
            n += 1;
        }
        print_debug!("Generator {} finished generating {} chunks in {} seconds generated from {} to {} chunks", start / (end - start) + 1, n, Instant::now().duration_since(time).as_secs_f32(), start, end);
    }

    fn receive_chunks(&mut self, pos_sender :mpsc::Sender<glam::IVec3>) -> bool {
        print_debug!("Iterating over the chunk_receiver to get the chunks freshly created");

        let mut n = 0;

        for chunk in self.chunk_receiver.try_iter() {
            n += 1;
            pos_sender.send(*chunk.get_chunk_pos()).expect("Error when sending chunk");
            self.world_data.write().unwrap().chunks.insert(chunk.get_chunk_pos(), Arc::new(chunk));
        }
        if n > 0 {
            print_base!("Collected {} chunks", n);
        }
        n > 0
    }

    fn receive_positions_to_build(&self, pos_sender :mpsc::Sender<glam::IVec3>) -> Result<(Vec<glam::IVec3>, HashMap<glam::IVec3,Arc<Chunk>>), String> {
        let mut map_chunks = HashMap::new();
        let mut new_chunks = Vec::new();

        print_debug!("Iterating over the pos_receiver to get the chunks buffered");

        let mut n = 0;
        let world = self.world_data.read().unwrap();
        let pos_s = self.pos_receiver.try_iter().collect::<Vec<glam::IVec3>>();
        for pos in pos_s {
            n += 1;
            if !world.chunks.contains_key(&pos) {
                continue;
            }
            let neighbours_count = Self::get_neighbours_if_exist(&world, pos, &mut map_chunks);
            if neighbours_count == 6 {
                new_chunks.push(pos);
            } else if neighbours_count > 0 {
                pos_sender.send(pos.clone()).expect("Error when sending chunk");
            }
        }

        print_base!("Iterated over {} chunks, len of world_data.chunks is {}, len of new chunks is {}", n, world.chunks.len(), new_chunks.len());

        if new_chunks.is_empty() {
            return Err("No new chunks found".to_string());
        }


        print_base!("Succesfully added {} chunks into the chunks vector", new_chunks.len());
        Ok((new_chunks, map_chunks))
    }

    pub fn build_chunk_mesh(&mut self) {

        if self.world_data.read().unwrap().is_building.load(Ordering::Relaxed) {
            print_debug!("Already building, skipping...");
            return;
        }

        let pos_sender_clone = self.pos_sender.clone();

        if !self.receive_chunks(pos_sender_clone.clone()) {
            return;
        }

        let result = self.receive_positions_to_build(pos_sender_clone.clone());

        if result.is_err() {
            return;
        }
        let (new_chunks, map_chunks) = result.unwrap();

        let world_data_clone = self.get_data();
        let tx = self.mesh_sender.clone();

        print_debug!("Initiating thread to build mesh with {} chunks", new_chunks.len());
        // 6. Spawn (or better yet, send to a thread pool like 'rayon')
        std::thread::Builder::new().name("meshes_thread".to_string()).spawn(move || {
            world_data_clone.read().unwrap().is_building.store(true, Ordering::Relaxed);
            print_debug!("Acquiring lock on is building");
            World::thread_build_mesh(map_chunks, tx, new_chunks);
            world_data_clone.read().unwrap().is_building.store(false, Ordering::Relaxed);
            print_debug!("Releasing the lock on is_building attr");
        }).unwrap();

    }

    fn thread_build_mesh(chunks_map : HashMap<glam::IVec3,Arc<Chunk>>, mesh_sender: mpsc::Sender<PackedData>, chunks: Vec<glam::IVec3>) {
        print_debug!("Building meshes");
        // acquiring a read lock on the world_data
        let mut n = 0;
        // iterating over the chunks in the vector
        for pos in chunks.iter() {
            let chunk = chunks_map.get(pos).unwrap();
            n += 1;
            let mesh = chunk.build_mesh(&chunks_map);
            let packed = PackedData {
                pos : pos.clone(),
                mesh
            };
            if mesh_sender.send(packed).is_err() {
                print_debug!("Failed to send chunk");
                return;
            }
        }
        drop(chunks_map);
        print_base!("Built mesh of {} chunks", n);
    }

    fn get_neighbours_if_exist(world: &RwLockReadGuard<WorldData>, pos : glam::IVec3, map : &mut HashMap<glam::IVec3,Arc<Chunk>>) -> u8 {
        let v_pos= Self::get_neighbours_chunks_pos(&pos);
        let mut count = 0;
        for pos in v_pos.iter() {
            if world.chunks.contains_key(&pos) {
                count += 1;
            }
        }
        if count == 6 {
            for p in v_pos.iter() {
                map.entry(*p).or_insert(world.chunks.get(p).unwrap().clone());
            }
            map.entry(pos).or_insert(world.chunks.get(&pos).unwrap().clone());
            print_debug!("Neighbours exist");
        }
        // print_debug!("Neighbours don't exist X-1:{}, X+1:{}, Y-1:{} , Y+1:{}, Z-1:{}, Z+1:{}", (world_data.chunks.contains_key(&x_neg)), (world_data.chunks.contains_key(&x_pos)), (world_data.chunks.contains_key(&y_neg)), (world_data.chunks.contains_key(&y_pos)), (world_data.chunks.contains_key(&z_neg)), (world_data.chunks.contains_key(&z_pos)));
        count
    }

    pub unsafe fn collect_meshes(&mut self) {
        let mut n = 0;
        let start_time = Instant::now();
        // Iterating in the receiver to link (main thread)
        for mut packed in self.mesh_receiver.try_iter() {
            n += 1;

            print_debug!("Linking chunk at {}", packed.pos);
            packed.mesh.link();

            // inserting the mesh into the HashMap of the world_data
            self.get_data().write().unwrap().meshes.insert(packed.pos, packed.mesh);
            if start_time.elapsed().as_millis() > 2 { break; }
        }

        if n > 0 {
            print_debug!("Linked {} packed chunks data",n);
        }
    }

    pub unsafe fn render(&mut self, window: &Window, player: &PlayerUser) {
        self.create_chunks(player.get_coords().as_ivec3());
        let pos_s = self.get_data().read().unwrap().render(window, player);
        self.get_data().write().unwrap().clean_chunks(pos_s,player);

    }

    pub fn get_block_at(&self, ipos : glam::IVec3) -> u16 {
        self.world_data.read().unwrap().get_block_at(ipos)
    }

    pub fn tick(&mut self, delta_time: f64) {
        self.world_data.read().unwrap().tick(delta_time);
    }

}

impl Drop for World {
    fn drop(&mut self) {
        print_base!("Dropping the world_data")
    }
}

impl WorldData {

    pub unsafe fn new(texture_array: TextureArray) -> WorldData {
        Self {
            chunks: HashMap::new(),
            meshes: HashMap::new(),
            is_building: Default::default(),
            is_generating: Default::default(),
            texture_array,
            chunk_shader: Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/vertex.vert".to_string(), "/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/fragment.frag".to_string()),
        }
    }
    pub fn get_block_at(&self, ipos : glam::IVec3) -> u16 {
        let sz = CHUNK_SIZE as i32;

        let chunk_pos = ipos.div_euclid(glam::IVec3::new(sz,sz,sz));
        let block_pos = ipos.rem_euclid(glam::IVec3::new(sz,sz,sz));

        self.chunks.get(&chunk_pos).map_or(0, |chunk| chunk.get_block_at(block_pos))
    }

    fn tick(&self, delta_time: f64) {
        // self.player.set_delta_time(delta_time);
        //    light.setColor(glfwGetTime());
        // light.setColor(100);
        // handleKeysPressed(window->OGLwindow, &player);
    }

    unsafe fn render(&self, window: &Window, player: &PlayerUser) -> Vec<glam::IVec3>{
        //    Logs::debug("Rendering world_data");
        let camera_pos: glam::Vec3 = player.get_coords();
        let camera_target: glam::Vec3 = camera_pos + player.get_direction();

        // build view matrix
        let view: glam::Mat4 = glam::Mat4::look_at_lh(camera_pos, camera_target, player.get_up());
        let projection: glam::Mat4 = glam::Mat4::perspective_lh(player.get_fov().to_radians(), window.inner_size().width as f32 / window.inner_size().height as f32,
                                                                0.01_f32, 1000.0_f32);
        let pro_view = projection * view;

        gl::DepthFunc(gl::LESS);
        let light_pos = glam::vec3(100.0 + 100.0 * 10.0, 1000.0, 100.0 + 100.0 * 10.0);
        // light.render(pro_view, player.get_coords() + let(0.0f, 100.0f, 0.0f));

        self.chunk_shader.use_shader();
        self.texture_array.use_textures(&self.chunk_shader);
        // camera/view transformation
        //    let color = light.getColor();
        let color = glam::Vec3::new(1.0_f32, 1.0_f32, 1.0_f32); // Default color for debugging
        self.chunk_shader.set_vec3("color", color);
        self.chunk_shader.set_vec3("uniformLightColor", glam::vec3(1.0, 1.0, 1.0));
        self.chunk_shader.set_vec3("uniformLightPos", light_pos);
        self.chunk_shader.set_vec3("uniformViewPos", camera_pos);
        self.chunk_shader.set_matrix4fv("uniform_projection_view", pro_view);
        print_debug!("Len of meshes {}, len of chunks {}",self.meshes.len(), self.chunks.len());


        // iterating over the hashmap
        let mut pos_to_remove = Vec::new();
        // iterating over all the meshes to render them / or remove the ones that are too far from the player
        for (pos, mesh) in &self.meshes {
            if Self::is_chunk_too_far_from_player(pos,player) {
                pos_to_remove.push(pos.clone());
                continue;
            }
            // small calculations for the 3d rendering
            let mut model = glam::Mat4::IDENTITY;
            model = model * glam::Mat4::from_translation(pos.as_vec3() * CHUNK_SIZE as f32);
            self.chunk_shader.set_matrix4fv("uniform_model", model);
            mesh.draw();
        }
        pos_to_remove
    }

    fn clean_chunks(&mut self, pos_s : Vec<glam::IVec3>, player_user: &PlayerUser) {
        let mut n = 0;

        for pos in pos_s {
            n += 1;
            self.remove_chunk(&pos);
        }

        self.chunks.retain(|pos, _c| {
            !Self::is_chunk_too_far_from_player(pos, player_user)
        });

        if n > 0 {
            print_base!("Wiped {} chunks", n)
        };
    }

    fn is_chunk_too_far_from_player(pos: &glam::IVec3, player: &PlayerUser) -> bool {
        let current_chunk = player.get_coords().as_ivec3() / CHUNK_SIZE as i32;

        pos.x < current_chunk.x - WORLD_SIZE as i32 || pos.x > current_chunk.x + WORLD_SIZE as i32
            || pos.z < current_chunk.z - WORLD_SIZE as i32 || pos.z > current_chunk.z + WORLD_SIZE as i32
    }

    fn remove_chunk(&mut self, pos : &glam::IVec3) {
        self.chunks.remove(&pos);
        self.meshes.remove(&pos);
    }

    unsafe fn use_shader(&self){
        self.chunk_shader.use_shader();
    }
}

impl Drop for WorldData {
    fn drop(&mut self) {
        print_base!("Dropping the world_data data");
    }
}
