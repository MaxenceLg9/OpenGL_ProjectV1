use std::collections::HashMap;

pub struct Player {
    pos : glam::Vec3,
    direction : glam::Vec3,
    up : glam::Vec3,
    // deltaTime : f32,
    roll : f32,
    speedMultiplier : HashMap<i32, f32>,
    fov : f32
}

impl Player {
    pub fn moveForward(&mut self, delta : f32) {
        self.pos += self.direction * delta * 3.5 * self.get_speed(delta);
        println!("Pos : {},{},{}", self.pos.x, self.pos.y, self.pos.z);
        println!("Direction : {},{},{}", self.direction.x, self.direction.y, self.direction.z);
    }

    pub fn moveRight(&mut self, delta : f32) {
        let right : glam::Vec3 = glam::Vec3::normalize(glam::Vec3::cross(self.direction, self.up));
        // println!("Moving right: %f\n", glam::length(right));
        self.pos = self.pos + right * delta * 3.5 * self.get_speed(delta);
        println!("Pos : {},{},{}", self.pos.x, self.pos.y, self.pos.z);
    }

    pub fn moveUp(&mut self, delta : f32) {
        // Rotate baseUp around front vector (XZ plane) by roll angle to get the rolled-up vector
        println!("Length rolledUp: {}", glam::Vec3::length(self.up));
        // Move along local-up
        self.pos += self.up * delta * 2.5 * self.get_speed(delta);
    }

    pub fn get_speed(&self, delta : f32) -> f32{
        let mut speed = delta;
        for elt in self.speedMultiplier.values() {
            speed *= elt;
        }
        speed
    }

    pub fn addFov(&mut self, fov : f32) {
        self.fov -=fov;
        if (self.fov < 30.0) {
            self.fov = 30.0;
        } else if (self.fov > 140.0) {
            self.fov = 140.0;
        }
    }

    pub fn get_fov(&self) -> f32 {
        self.fov
    }

    pub fn new(x : f32, y : f32, z : f32) -> Self {
        println!("Creating player at {},{},{}\n", x, y, z);
        Self {
            pos : glam::vec3(x, y, z),
            speedMultiplier: HashMap::new(),
            up: glam::vec3(0.0,1.0,0.0),
            direction: glam::vec3(1.0,0.0,-1.0),
            fov: 140_f32,
            roll: 0_f32,
        }
    }

    pub fn get_coords(&self) -> glam::Vec3 {
        self.pos
    }

    pub fn get_direction(&self) -> glam::Vec3 {
        self.direction
    }

    pub fn makeRoll(&mut self, angle : f32) {
        self.roll += angle;
        self.computeUp_angle(angle);
    }

    pub fn get_up(&self) -> glam::Vec3 {
        self.up
    }

    pub fn moveCamera(&mut self, x_offset : f32, y_offset : f32) {
        // 1. Rotate around the UP vector (Yaw)
        let yaw_rotation = glam::Mat4::from_axis_angle(self.up, x_offset.to_radians());
        self.direction = yaw_rotation.transform_vector3(self.direction);

        // 2. Rotate around the RIGHT vector (Pitch)
        // Compute the right vector on the fly
        let right = self.direction.cross(self.up).normalize();
        let pitch_rotation = glam::Mat4::from_axis_angle(right, -y_offset.to_radians());
        self.direction = pitch_rotation.transform_vector3(self.direction);

        // 3. Re-compute the up vector
        self.compute_up();
    }

    pub fn compute_up(&mut self){
        // Rotate the standard world up (0,1,0) by the roll around the current direction
        let roll_rotation = glam::Mat4::from_axis_angle(self.direction, self.roll.to_radians());
        let new_up = roll_rotation.transform_vector3(glam::Vec3::Y);
        self.up = new_up.normalize();
    }
    pub fn computeUp_angle(&mut self, angle : f32){
        let rotation = glam::Mat4::from_axis_angle(self.direction, angle.to_radians());
        // transform_vector3 handles the Vec4 conversion and truncation for you
        self.up = rotation.transform_vector3(self.up).normalize();
        //    println!("up: %f %f %f\n", self.up.x, self.up.y, self.up.z);
    }

    pub fn addSpeedMultiplier(&mut self,key : i32, multi : f32) {
        self.speedMultiplier.insert(key,multi);
    }


    pub fn removeSpeedMultiplier(&mut self, key : i32) {
        self.speedMultiplier.remove(&key);
    }
}







