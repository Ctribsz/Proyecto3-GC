use nalgebra_glm::Vec3;

pub struct Planet {
    pub name: String,
    pub radius: f32,
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    pub rotation_speed: f32,
    pub color: u32,
    pub current_angle: f32,
    pub shader_index: u32, // Nuevo campo para el índice del shader
}

impl Planet {
    pub fn new(
        name: &str,
        radius: f32,
        orbit_radius: f32,
        orbit_speed: f32,
        rotation_speed: f32,
        color: u32,
        shader_index: u32, // Nuevo parámetro
    ) -> Self {
        Planet {
            name: name.to_string(),
            radius,
            orbit_radius,
            orbit_speed,
            rotation_speed,
            color,
            current_angle: 0.0,
            shader_index, // Inicializa el índice del shader
        }
    }

    pub fn update_position(&mut self) {
        self.current_angle += self.orbit_speed;
        if self.current_angle > 2.0 * std::f32::consts::PI {
            self.current_angle -= 2.0 * std::f32::consts::PI;
        }
    }

    pub fn get_position(&self) -> Vec3 {
        Vec3::new(
            self.orbit_radius * self.current_angle.cos(),
            0.0,
            self.orbit_radius * self.current_angle.sin(),
        )
    }
}