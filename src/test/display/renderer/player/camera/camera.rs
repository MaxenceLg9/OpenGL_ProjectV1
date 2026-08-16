use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use glam::Vec3;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};
use shared::common::world::pos::blockpos::BlockPos;
use shared::common::world::pos::chunkpos::ChunkPos;
use shared::common::world::pos::iblockpos::IBlockPos;
use crate::test::display::renderer::player::keyboard::Inputs;

pub struct Camera {
    position: BlockPos,
    yaw: f32,   // degrees
    pitch: f32, // degrees, clamped

    // derived each frame
    direction: glam::Vec3,
    right: glam::Vec3,
    up: glam::Vec3,
    fov : f32,
    speed_multiplier: HashMap<i32, f32>,
    roll : f32,
}

impl Camera {

    pub fn new(x : f32, y : f32, z : f32) -> Self {
        Self {
            position : BlockPos::new(x,y,z),
            yaw : 0.0,
            pitch : 0.0,
            up: glam::vec3(0.0,1.0,0.0),
            direction: glam::vec3(1.0,0.0,1.0),
            right : glam::vec3(1.0, 0.0, 0.0),
            speed_multiplier: HashMap::new(),
            fov : 140_f32,
            roll : 0.0
        }
    }
    pub fn get_speed(&self, time: f32) -> f32{
        let mut speed = time;
        for elt in self.speed_multiplier.values() {
            speed *= elt;
        }
        speed
    }
    pub fn add_speed_multiplier(&mut self, key : i32, multi : f32) {
        self.speed_multiplier.insert(key, multi);
    }
    pub fn remove_speed_multiplier(&mut self, key : i32) {
        self.speed_multiplier.remove(&key);
    }

    pub fn poll_keys(&mut self, buttons: &mut Inputs, time : f32, debug : &mut bool) {
        for elt in buttons.get_keyboard().iter_mut() {
            match elt.1.current_state {
                ElementState::Pressed => {
                    match elt.0 {
                        PhysicalKey::Code(KeyCode::KeyW) => {
                            self.move_forward(1.0, time);
                        }
                        PhysicalKey::Code(KeyCode::KeyS) => {
                            self.move_forward(-1.0, time);
                        }
                        PhysicalKey::Code(KeyCode::KeyA) => {
                            self.move_left(1.0, time);
                        }
                        PhysicalKey::Code(KeyCode::KeyD) => {
                            self.move_left(-1.0, time);
                        },
                        PhysicalKey::Code(KeyCode::Space) => {
                            self.move_up(1.0, time);
                        },
                        PhysicalKey::Code(KeyCode::ControlLeft) => {
                            self.move_up(-1.0, time);
                        },
                        PhysicalKey::Code(KeyCode::ShiftLeft) => {
                            self.add_speed_multiplier(0, 50.0);
                        },
                        _ => {}
                    }
                }
                ElementState::Released => {
                    match elt.0 {
                        PhysicalKey::Code(KeyCode::ShiftLeft) => {
                            self.remove_speed_multiplier(0)
                        },
                        PhysicalKey::Code(KeyCode::F3) => {
                            if elt.1.last_state == ElementState::Pressed {
                                elt.1.last_state = ElementState::Released;
                                *debug = !*debug;
                            }
                        },
                        _ => ()
                    }
                }
            }
        }
    }
    // pub fn ray_cast(direction : glam::Vec3, pos : BlockPos, client_chunk_map: &ClientChunkMap) -> Option<(u16, IBlockPos, glam::IVec3)> {
    //     let max_distance = 3.0;
    //     let mut step_dir = 0;
    //     // Track the integer block coordinates directly
    //     let mut current_block = pos.as_ivec3();
    //
    //     let mut voxel_normal = glam::IVec3::splat(0);
    //
    //     // step to increase the block pos
    //     let step_x = if direction.x > 0.0 { 1 } else { -1 };
    //     let step_y = if direction.y > 0.0 { 1 } else { -1 };
    //     let step_z = if direction.z > 0.0 { 1 } else { -1 };
    //
    //     // step to update the x,y,z distances travelled
    //     let t_delta_x = if direction.x.abs() > 0.0 { (1.0 / direction.x).abs() } else { f32::MAX };
    //     let t_delta_y = if direction.y.abs() > 0.0 { (1.0 / direction.y).abs() } else { f32::MAX };
    //     let t_delta_z = if direction.z.abs() > 0.0 { (1.0 / direction.z).abs() } else { f32::MAX };
    //
    //     // initializing x,y,z distances with the floating part of the x,y,z coordinates
    //     let fract_x = pos.x - pos.x.floor();
    //     let mut x_distance = if direction.x > 0.0 { (1.0 - fract_x) * t_delta_x } else { fract_x * t_delta_x };
    //
    //     let fract_y = pos.y - pos.y.floor();
    //     let mut y_distance = if direction.y > 0.0 { (1.0 - fract_y) * t_delta_y } else { fract_y * t_delta_y };
    //
    //     let fract_z = pos.z - pos.z.floor();
    //     let mut z_distance = if direction.z > 0.0 { (1.0 - fract_z) * t_delta_z } else { fract_z * t_delta_z };
    //
    //     let mut distance = 0.0;
    //
    //     while distance <= max_distance {
    //         let result = client_chunk_map.get_block_at(IBlockPos::from_vec3(current_block));
    //         if result != 0 {
    //             if step_dir == 0 {
    //                 voxel_normal.x = -step_x;
    //             }
    //             else if step_dir == 1 {
    //                 voxel_normal.y = -step_y;
    //             }
    //             else {
    //                 voxel_normal.z = -step_z;
    //             }
    //             return Some((result, IBlockPos::from_vec3(current_block), voxel_normal));
    //         }
    //
    //         if x_distance < y_distance {
    //             if x_distance < z_distance {
    //                 current_block.x += step_x; // Step by exactly 1 block
    //                 distance = x_distance;        // Update total distance travelled
    //                 x_distance += t_delta_x;      // Set target to the next X boundary
    //                 step_dir = 0;
    //             } else {
    //                 current_block.z += step_z;
    //                 distance = z_distance;
    //                 z_distance += t_delta_z;
    //                 step_dir = 2;
    //             }
    //         } else {
    //             if y_distance < z_distance {
    //                 current_block.y += step_y;
    //                 distance = y_distance;
    //                 y_distance += t_delta_y;
    //                 step_dir = 1;
    //             } else {
    //                 current_block.z += step_z;
    //                 distance = z_distance;
    //                 z_distance += t_delta_z;
    //                 step_dir = 2;
    //             }
    //         }
    //     }
    //     None
    // }

    pub fn get_coords(&self) -> BlockPos {
        self.position
    }

    pub fn get_direction(&self) -> glam::Vec3 {
        self.direction
    }

    fn make_roll(&mut self, angle : f32) {
        self.roll += angle;
        self.compute_up_angle(angle);
    }

    pub fn compute_up(&mut self){
        // Rotate the standard world_data up (0,1,0) by the roll around the current direction
        let roll_rotation = glam::Mat4::from_axis_angle(self.direction, self.roll.to_radians());
        let new_up = roll_rotation.transform_vector3(glam::Vec3::Y);
        self.up = new_up.normalize();
    }
    fn compute_up_angle(&mut self, angle : f32){
        let rotation = glam::Mat4::from_axis_angle(self.direction, angle.to_radians());
        // transform_vector3 handles the Vec4 conversion and truncation for you
        self.up = rotation.transform_vector3(self.up).normalize();
        //    println!("up: %f %f %f\n", self.up.x, self.up.y, self.up.z);
    }

    pub fn get_up(&self) -> glam::Vec3 {
        self.up
    }

    pub fn mouse_callback(&mut self, xoffset : f64, yoffset : f64) {
        let sensitivity = 0.12;  // much smaller for fine rotation
        // xoffset *= -sensitivity;
        // yoffset *= -sensitivity;

        self.move_camera(xoffset * sensitivity, yoffset * sensitivity);
    }

    pub fn get_chunk_pos(&self) -> ChunkPos {
        self.position.get_chunk_pos()
    }

    pub fn get_pos_info(&self) -> (BlockPos, Vec3, Vec3, f32) {
        (self.position,self.direction,self.up, self.fov)
    }

    pub fn set_pos(&mut self, pos : BlockPos) {
        self.position = pos;
    }

    fn move_forward(&mut self, delta : f32, time : f32) {
        self.position += self.direction * delta * 3.5 * self.get_speed(time);
        // println!("Pos : {},{},{}", self.position.x, self.position.y, self.position.z);
        // println!("Direction : {},{},{}", self.direction.x, self.direction.y, self.direction.z);
    }

    fn move_left(&mut self, delta : f32, time : f32) {
        let right : glam::Vec3 = glam::Vec3::normalize(glam::Vec3::cross(self.direction, self.up));
        // println!("Moving right: %f\n", glam::length(right));
        self.position = self.position + right * delta * 3.5 * self.get_speed(time);
        // println!("Pos : {},{},{}", self.position.x, self.position.y, self.position.z);
    }

    fn move_up(&mut self, delta : f32, time : f32) {
        // Rotate baseUp around front vector (XZ plane) by roll angle to get the rolled-up vector
        // println!("Length rolledUp: {}", glam::Vec3::length(self.up));
        // Move along local-up
        self.position += Vec3::Y * delta * 2.5 * self.get_speed(time);
    }
    pub fn move_camera(&mut self, x_offset: f64, y_offset: f64) {
        self.yaw -= x_offset as f32;
        self.pitch -= y_offset as f32; // flip sign if inverted for your convention

        // Prevent hitting the singularity at the poles
        self.pitch = self.pitch.clamp(-89.0, 89.0);

        self.update_vectors();
    }

    pub fn add_fov(&mut self, mouse_scroll_delta: MouseScrollDelta) {
        let fov = match mouse_scroll_delta {
            MouseScrollDelta::LineDelta(x, y) => {
                y * 2.0
            }
            MouseScrollDelta::PixelDelta(pixels) => {
                pixels.y as f32 * 0.1
            }
        };
        self.fov -=fov;
        if self.fov < 30.0 {
            self.fov = 30.0;
        } else if self.fov > 140.0 {
            self.fov = 140.0;
        }
    }

    pub fn get_fov(&self) -> f32 {
        self.fov
    }

    pub fn update_vectors(&mut self) {
        let yaw_r = self.yaw.to_radians();
        let pitch_r = self.pitch.to_radians();

        let direction = glam::vec3(
            yaw_r.cos() * pitch_r.cos(),
            pitch_r.sin(),
            yaw_r.sin() * pitch_r.cos(),
        );
        self.direction = direction.normalize();

        // world_up is fixed, so this cross product is never degenerate
        self.right = self.direction.cross(glam::Vec3::Y).normalize();
        self.up = self.right.cross(self.direction).normalize();
    }
}