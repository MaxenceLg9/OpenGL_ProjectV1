use std::sync::Arc;
use shared::print_base;
use crate::test::display::renderer::player::camera::camera::Camera;
use crate::test::display::renderer::player::keyboard::Inputs;

pub struct ClientPlayer {
    camera : Camera,
    inputs: Inputs,
}



impl ClientPlayer {
    pub fn new(x : f32, y : f32, z : f32) -> Self {
        print_base!("Creating player at {},{},{}", x, y, z);
        Self {
            inputs : Inputs::new(),
            camera : Camera::new(x,y,z)

        }
    }
    pub fn get_keyboard(&mut self) -> &mut Inputs {
        &mut self.inputs
    }
    pub fn get_camera(&mut self) -> &mut Camera {
        &mut self.camera
    }
    pub fn poll_keys(&mut self, time : f32, debug : &mut bool) {
        self.camera.poll_keys(&mut self.inputs, time, debug);
    }
}






