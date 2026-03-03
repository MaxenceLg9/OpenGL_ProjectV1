use std::collections::{HashMap};
use glam::IVec3;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use shared::common::world::pos::chunkpos::CHUNK_SIZE;
use shared::print_base;

pub struct ServerPlayer {
    pos : glam::Vec3,
    direction : glam::Vec3,
    up : glam::Vec3,
    roll : f32,
    speed_multiplier: HashMap<i32, f32>,
    keyboard : HashMap<PhysicalKey,KeyState>,
    fov : f32,
}

pub struct KeyState {
    last_state: ElementState,
    current_state: ElementState,
}

impl ServerPlayer {

    pub fn get_chunk_pos(&self) -> IVec3 {
        self.pos.as_ivec3() / CHUNK_SIZE as i32
    }
    fn move_forward(&mut self, delta : f32, time : f32) {
        self.pos += self.direction * delta * 3.5 * self.get_speed(time);
        // println!("Pos : {},{},{}", self.pos.x, self.pos.y, self.pos.z);
        // println!("Direction : {},{},{}", self.direction.x, self.direction.y, self.direction.z);
    }

    fn move_left(&mut self, delta : f32, time : f32) {
        let right : glam::Vec3 = glam::Vec3::normalize(glam::Vec3::cross(self.direction, self.up));
        // println!("Moving right: %f\n", glam::length(right));
        self.pos = self.pos + right * delta * 3.5 * self.get_speed(time);
        // println!("Pos : {},{},{}", self.pos.x, self.pos.y, self.pos.z);
    }

    fn move_up(&mut self, delta : f32, time : f32) {
        // Rotate baseUp around front vector (XZ plane) by roll angle to get the rolled-up vector
        // println!("Length rolledUp: {}", glam::Vec3::length(self.up));
        // Move along local-up
        self.pos += self.up * delta * 2.5 * self.get_speed(time);
    }

    pub fn get_speed(&self, time: f32) -> f32{
        let mut speed = time;
        for elt in self.speed_multiplier.values() {
            speed *= elt;
        }
        speed
    }

    fn add_fov(&mut self, fov : f32) {
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

    pub fn new(x : f32, y : f32, z : f32) -> Self {
        print_base!("Creating player at {},{},{}", x, y, z);
        Self {
            pos : glam::vec3(x, y, z),
            speed_multiplier: HashMap::new(),
            up: glam::vec3(0.0,1.0,0.0),
            direction: glam::vec3(1.0,0.0,1.0),
            fov: 140_f32,
            roll: 0_f32,
            keyboard: HashMap::new()
        }
    }

    pub fn get_coords(&self) -> glam::Vec3 {
        self.pos
    }

    pub fn get_direction(&self) -> glam::Vec3 {
        self.direction
    }

    fn make_roll(&mut self, angle : f32) {
        self.roll += angle;
        self.compute_up_angle(angle);
    }

    pub fn get_up(&self) -> glam::Vec3 {
        self.up
    }

    fn move_camera(&mut self, x_offset : f64, y_offset : f64) {
        // 1. Rotate around the UP vector (Yaw)
        let yaw_rotation = glam::Mat4::from_axis_angle(self.up, x_offset.to_radians() as f32);
        self.direction = yaw_rotation.transform_vector3(self.direction);

        // 2. Rotate around the RIGHT vector (Pitch)
        // Compute the right vector on the fly
        let right = self.direction.cross(self.up).normalize();
        let pitch_rotation = glam::Mat4::from_axis_angle(right, -y_offset.to_radians() as f32);
        self.direction = pitch_rotation.transform_vector3(self.direction);
        // println!("Direction : {},{},{}", self.direction.x, self.direction.y, self.direction.z);
        // 3. Re-compute the up vector
        self.compute_up();
    }

    pub fn mouse_callback(&mut self, xoffset : f64, yoffset : f64) {
        let sensitivity = 0.12;  // much smaller for fine rotation
        // xoffset *= -sensitivity;
        // yoffset *= -sensitivity;

        self.move_camera(xoffset * sensitivity, yoffset * sensitivity);
    }

    pub fn key_callback(&mut self, input : KeyEvent) {
        if self.keyboard.contains_key(&input.physical_key) {
            let key_state = self.keyboard.get_mut(&input.physical_key).unwrap();
            key_state.last_state = key_state.current_state;
            key_state.current_state = input.state;
        } else {
            self.keyboard.insert(input.physical_key, KeyState {
                last_state: input.state,
                current_state: input.state,
            });
        }
    }

    pub fn poll_keys(&mut self, time : f32) {
        let keyboard = self.keyboard.clone();
        for elt in keyboard.iter() {
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
                        }
                        _ => {}
                    }
                }
                ElementState::Released => {
                    match elt.0 {
                        PhysicalKey::Code(KeyCode::ShiftLeft) => {
                            self.remove_speed_multiplier(0)
                        },
                        _ => ()
                    }
                }
            }
        }
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

    pub fn add_speed_multiplier(&mut self, key : i32, multi : f32) {
        self.speed_multiplier.insert(key, multi);
    }


    pub fn remove_speed_multiplier(&mut self, key : i32) {
        self.speed_multiplier.remove(&key);
    }
}

impl Clone for KeyState {
    fn clone(&self) -> KeyState {
        Self {
            current_state: self.current_state.clone(),
            last_state: self.last_state.clone(),
        }
    }
}






