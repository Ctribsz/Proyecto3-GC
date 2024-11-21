use nalgebra_glm::{Vec3, rotate_vec3};
use std::f32::consts::PI;

pub struct Camera {
    pub eye: Vec3,
    pub center: Vec3,
    pub up: Vec3,
    pub has_changed: bool,
}

impl Camera {

    pub fn rotate_around_target(&mut self, angle: f32, distance: f32) {
        // Calcula la nueva posición de la cámara rotando alrededor del centro (target).
        self.eye.x = self.center.x + distance * angle.cos();
        self.eye.z = self.center.z + distance * angle.sin();
    }
    
    pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        Camera {
            eye,
            center,
            up,
            has_changed: true,
        }
    }

    // Orbita alrededor del punto central, ajustando yaw y pitch
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let radius_vector = self.eye - self.center;
        let radius = radius_vector.magnitude();

        let current_yaw = radius_vector.z.atan2(radius_vector.x);
        let radius_xz = (radius_vector.x.powi(2) + radius_vector.z.powi(2)).sqrt();
        let current_pitch = (-radius_vector.y).atan2(radius_xz);

        let new_yaw = (current_yaw + delta_yaw) % (2.0 * PI);
        let new_pitch = (current_pitch + delta_pitch).clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);

        self.eye = self.center + Vec3::new(
            radius * new_yaw.cos() * new_pitch.cos(),
            -radius * new_pitch.sin(),
            radius * new_yaw.sin() * new_pitch.cos()
        );

        self.has_changed = true;
    }

    // Cambia el centro de la cámara moviéndolo en la dirección especificada
    pub fn move_center(&mut self, direction: Vec3) {
        let movement = direction.normalize() * 0.1; // Adjust the factor for movement speed
        self.center += movement;
        self.eye += movement;

        self.has_changed = true;
    }

    // Acerca o aleja la cámara hacia el punto central
    pub fn zoom(&mut self, delta: f32) {
        let direction = (self.center - self.eye).normalize();
        self.eye += direction * delta;
        self.has_changed = true;
    }

    // Verifica si la cámara ha cambiado y resetea el estado
    pub fn check_if_changed(&mut self) -> bool {
        if self.has_changed {
            self.has_changed = false;
            true
        } else {
            false
        }
    }
}
