// let's move the camera and movement here for now?

use std::collections::HashSet;

use glium::winit::{event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}};
use nalgebra_glm::{cross, look_at, normalize, Vec3};

pub struct Player
{
    pub camera_pos: Vec3,
    pub camera_front: Vec3,
    pub camera_up: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
    pub speed: f32,
}

impl Player 
{
    
    pub fn new(pos: Vec3) -> Self {
        let camera_front = Vec3::new(0.0,0.0,-1.0);
        let camera_pos = Vec3::new(pos.x, pos.y, pos.z + 3.0);
        let camera_up = Vec3::new(0.0, 1.0, 0.0);
        let yaw = 0.0;
        let pitch = 0.0;
        let sensitivity = 0.1;
        let speed = 3.0;
        Player { camera_pos, camera_front, camera_up, yaw, pitch, sensitivity, speed }
    }

    pub fn handle_keyboard_inputs(&mut self, keys_pressed: &HashSet<PhysicalKey>, window_target: &ActiveEventLoop, delta_time: f32) {
        // let's handle arrow keys to move the camera ? 
        let move_speed = self.speed * delta_time;

        if keys_pressed.contains(&PhysicalKey::Code(KeyCode::KeyW)) {
           self.camera_pos +=  self.camera_front.scale(move_speed);
        }
        if keys_pressed.contains(&PhysicalKey::Code(KeyCode::KeyS)) {
            self.camera_pos  -=  self.camera_front.scale(move_speed);
        }
        if keys_pressed.contains(&PhysicalKey::Code(KeyCode::KeyA)) {
            let right = normalize(&cross(&self.camera_front, &self.camera_up));
            self.camera_pos -=  right.scale(move_speed);
        }
        if keys_pressed.contains(&PhysicalKey::Code(KeyCode::KeyD)) {
            let right = normalize(&cross(&self.camera_front, &self.camera_up));
            self.camera_pos +=  right.scale(move_speed);
        }
        if keys_pressed.contains(&PhysicalKey::Code(KeyCode::Space)) {
            self.camera_pos +=  self.camera_up.scale(move_speed);
        }
        if keys_pressed.contains(&PhysicalKey::Code(KeyCode::ShiftLeft)) {
            self.camera_pos -=  self.camera_up.scale(move_speed);
        }
        // Escape
        if keys_pressed.contains(&PhysicalKey::Code(KeyCode::Escape)) {
            window_target.exit();
        }
    }
    
   
    pub fn handle_mouse_inputs(&mut self, mut x_offset: f32, mut y_offset: f32) {
            x_offset *= self.sensitivity;
            y_offset *= self.sensitivity;

            self.yaw += x_offset;
            self.pitch += y_offset;

            if self.pitch > 89.0 {
                self.pitch = 89.0;
            }

            if self.pitch < -89.0 {
                self.pitch = -89.0;
            }


    }

    pub fn get_view_matrix(&mut self) -> [[f32; 4]; 4] {
         self.camera_front = Vec3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos()
        ).normalize();

        // View Matrix (move backwards)
        let view: [[f32;4];4] = look_at(
            &self.camera_pos,
            &(self.camera_pos + self.camera_front),
            &self.camera_up
        ).into();   

        view
    }
}