use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::sync::mpsc;
use std::time::Instant;
use winit::window::Window;
use crate::display::renderer::mesh::{chunk_mesh::*, texture::texture_array::TextureArray, shader::shader::Shader};
use crate::game::world::chunk::block::block::BlockType;
use super::{chunk::chunk::*, player::player::Player};

pub const WORLD_SIZE : u32 = 4;
pub const WORLD_THREADS : u32 = 16;

pub struct World {
    world_data: Arc<RwLock<WorldData>>,
    chunk_receiver: Option<mpsc::Receiver<Chunk>>,
    mesh_receiver: Option<mpsc::Receiver<PackedData>>,
}

pub struct WorldData{
    chunks: HashMap<glam::IVec3, Chunk>,
    meshes: HashMap<glam::IVec3, ChunkMesh>,
    is_building: AtomicBool,
    texture_array: TextureArray,
    chunk_shader: Shader,
    player: Player,
}

struct PackedData {
    chunk: Chunk,
    mesh : ChunkMesh
}

impl World {

    pub unsafe fn new() -> World {
        let texture_array = TextureArray::new("textures".to_string());
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/stone.png", BlockType::STONE.get_value() - 1).expect("Cannot add block to texture array");
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/dirt.png", BlockType::DIRT.get_value() - 1).expect("Cannot add block to texture array");
        texture_array.add_texture("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/textures/blocks/deepslate.png", BlockType::DEEPSLATE.get_value() - 1).expect("Cannot add block to texture array");

        let mut world = Self {
            world_data: Arc::new(RwLock::new(WorldData::new(texture_array))),
            chunk_receiver: None,
            mesh_receiver: None,
        };

        world.create_chunks();
        world.world_data.read().unwrap().use_shader();
        world
    }

    pub fn create_chunks(&mut self) {
        let (tx, rx) = mpsc::channel();
        println!("Running threads to generate the chunks");
        for i in 0..WORLD_THREADS {
            let tx_thread = tx.clone();
            std::thread::spawn(move || {
                World::generate_chunks(i,tx_thread);
            });
        }
        self.chunk_receiver = Some(rx);
    }

    pub fn generate_chunks(part : u32, chunk_sender: mpsc::Sender<Chunk>) {
        let time : Instant = Instant::now();
        let total_chunks: u32 = WORLD_SIZE * WORLD_SIZE * WORLD_SIZE;
        let chunk_per_thread: u32 = (total_chunks + WORLD_THREADS - 1) / WORLD_THREADS;

        let start_index: u32 = part * chunk_per_thread;
        let end_index: u32 = if start_index + chunk_per_thread > total_chunks { total_chunks } else { start_index + chunk_per_thread };
        for index in start_index..end_index {
            let i = (index / (WORLD_SIZE * WORLD_SIZE)) as i32;
            let j = ((index / WORLD_SIZE) % WORLD_SIZE) as i32;
            let k = (index % WORLD_SIZE) as i32;

            //        Logs::debug("Thread " + std::to_string(part) + " generating chunk at position: " + std::to_string(i) + "," + std::to_string(j) + "," + std::to_string(k));
            let chunk = Chunk::new(glam::ivec3(i, j, k));
            chunk_sender.send(chunk).expect("TODO: panic message");
        }
        println!("Thread {} finished generating chunks in {} seconds generated from {} to {} chunks", part + 1, Instant::now().duration_since(time).as_secs_f32(), start_index, end_index);
    }

    pub fn build_chunk_mesh(&mut self) {
        // 1. Lock the data ONCE.

        if self.mesh_receiver.is_some() || self.world_data.read().unwrap().is_building.load(Ordering::Relaxed) {
            // println!("Already building, skipping...");
            return;
        }

        // 4. Collect updates from the receiver (using your pop logic)
        // Note: We don't need the lock for the receiver!
        let mut new_chunks = Vec::new();
        if let Some(rx) = &self.chunk_receiver {
            new_chunks.extend(rx.try_iter());
        }
        if new_chunks.is_empty() {
            // println!("No new chunks to build");
            return;
        }

        // 5. Prepare thread data
        let (tx, rx) = mpsc::channel();
        let world_data_clone = Arc::clone(&self.world_data);

        println!("Initiating thread to build mesh with {} chunks", new_chunks.len());
        // 6. Spawn (or better yet, send to a thread pool like 'rayon')
        std::thread::spawn(move || {
            world_data_clone.read().unwrap().is_building.store(true, Ordering::Relaxed);
            println!("Acquiring lock on is building");
            World::thread_chunk_mesh(world_data_clone.clone(), tx, new_chunks);
            world_data_clone.read().unwrap().is_building.store(false, Ordering::Relaxed);
            println!("Releasing the lock on is_building attr");
        });

        self.mesh_receiver = Some(rx);
    }

    pub unsafe fn collect_meshes(&mut self) {
        if self.mesh_receiver.is_none() {
            return;
        }
        println!("Starting to collect meshes");
        let mut n = 0;
        println!("Check 1");
        for mut packed in self.mesh_receiver.as_ref().unwrap().try_iter() {
            println!("Linking chunk at {:?}", packed.chunk.get_chunk_pos());
            // If it crashes here, the issue is inside link()
            packed.mesh.link();

            let mut data = self.world_data.write().unwrap();
            data.meshes.insert(packed.chunk.get_chunk_pos(), packed.mesh);
            data.chunks.insert(packed.chunk.get_chunk_pos(), packed.chunk);
            n += 1;
        }
        println!("Check 2");

        // If we got the data, reset the receiver to None so we can build again
        if n > 0 {
            // println!("Collected {} packed chunks data",n);
        }
        if !self.world_data.read().unwrap().is_building.load(Ordering::Relaxed) {
            println!("Finished building meshes, mesh receiver is None");
            self.mesh_receiver = None;
        }
        println!("Check 3");
    }

    fn thread_chunk_mesh(_world: Arc<RwLock<WorldData>>, sender : mpsc::Sender<PackedData>, chunks : Vec<Chunk>) {
        println!("Building meshes");
        let world: RwLockReadGuard<WorldData> = _world.read().unwrap();
        let mut n = 0;
        for chunk in chunks {
            n += 1;
            println!("Check 4");
            let mesh = chunk.build_mesh(&world);
            let packed = PackedData {
                chunk,
                mesh
            };
            if sender.send(packed).is_err() {
                println!("Thread failed to send chunk");
                return;
            }
        }

        if n > 0 {
            println!("Built mesh of {} chunks", n)
        };
    }

    pub unsafe fn render(&mut self, window: &Window) {
        let mut d = self.world_data.read().unwrap();
        d.render(window);
        d.tick(0.005_f64);
    }

    pub fn get_block_at(&self, ipos : glam::IVec3) -> u16 {
        self.world_data.read().unwrap().get_block_at(ipos)
    }

    pub fn tick(&self, delta_time: f64) {
        self.world_data.read().unwrap().tick(delta_time);
    }

}

/*impl Drop for World {
    fn drop(&mut self) {
        for chunk in self.chunks.iter()
        {
            // Free the ChunkMesh
        }
        for mesh in self.meshes.iter()
        {
            mesh; // Free the ChunkMesh
        }
    }
}*/

impl WorldData {

    pub unsafe fn new(texture_array: TextureArray) -> WorldData {
        Self {
            chunks: HashMap::new(),
            meshes: HashMap::new(),
            is_building: Default::default(),
            texture_array,
            chunk_shader: Shader::new("/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/vertex.vert".to_string(), "/home/maxence/Documents/Dev/Prog/Rust/Projects/OpenGL_ProjectV1/src/assets/shaders/chunk/fragment.frag".to_string()),
            player: Player::new(-10_f32,0_f32,-10_f32),
        }
    }
    pub fn get_block_at(&self, ipos : glam::IVec3) -> u16 {
        let chunk_pos = glam::ivec3(ipos.x / CHUNK_SIZE as i32, ipos.y / CHUNK_SIZE as i32, ipos.z / CHUNK_SIZE as i32);
        let block_pos = glam::ivec3 (ipos.x % CHUNK_SIZE as i32, ipos.y % CHUNK_SIZE as i32, ipos.z % CHUNK_SIZE as i32);

        self.chunks.get(&chunk_pos)
            .map_or(0, |chunk| chunk.get_block_at(block_pos))
    }

    fn tick(&self, delta_time: f64) {
        // self.player.set_delta_time(delta_time);
        //    light.setColor(glfwGetTime());
        // light.setColor(100);
        // handleKeysPressed(window->OGLwindow, &player);
    }

    unsafe fn render(&self, window: &Window) {
        //    Logs::debug("Rendering world");
        let camera_pos: glam::Vec3 = self.player.get_coords();
        let camera_target: glam::Vec3 = camera_pos + self.player.get_direction();

        // build view matrix
        let view: glam::Mat4 = glam::Mat4::look_at_lh(camera_pos, camera_target, self.player.get_up());
        let projection: glam::Mat4 = glam::Mat4::perspective_lh(self.player.get_fov().to_radians(), window.inner_size().width as f32 / window.inner_size().height as f32,
                                                                0.01_f32, 1000.0_f32);
        let pro_view = projection * view;

        gl::DepthFunc(gl::LESS);
        // light.render(pro_view, player.get_coords() + let(0.0f, 100.0f, 0.0f));

        self.chunk_shader.use_shader();
        self.texture_array.use_textures(&self.chunk_shader);
        // camera/view transformation
        //    let color = light.getColor();
        let color = glam::Vec3::new(1.0_f32, 1.0_f32, 1.0_f32); // Default color for debugging
        self.chunk_shader.set_vec3("color".to_string(), &color);
        let mut n = 0;
        for i in 0..WORLD_SIZE as i32
        {
            for j in 0..WORLD_SIZE as i32
            {
                for k in 0..WORLD_SIZE as i32
                {
                    if (!self.meshes.contains_key(&glam::ivec3(i, j, k))) {
                        continue;
                    }
                    // Use get_mut to get a mutable reference
                    if let Some(mesh) = self.meshes.get(&glam::ivec3(i, j, k)) {
                        let mut model = glam::Mat4::IDENTITY;
                        model = model * glam::Mat4::from_translation(glam::vec3(i as f32, j as f32, k as f32) * CHUNK_SIZE as f32);
                        self.chunk_shader.set_matrix4fv("p_v_m".to_string(), &(pro_view * model));
                        mesh.draw();
                        n += 1;
                    }
                }
            }
        }
        if n > 0 {
            // println!("Rendered {} chunks", n);
        }

    }
    unsafe fn use_shader(&self){
        self.chunk_shader.use_shader();
    }
}
