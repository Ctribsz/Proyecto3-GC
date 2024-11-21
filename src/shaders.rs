use nalgebra_glm::{Vec3, Vec4, Mat3, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

    let w = transformed.w;
    let transformed_position = Vec4::new(
        transformed.x / w,
        transformed.y / w,
        transformed.z / w,
        1.0
    );

    let screen_position = uniforms.viewport_matrix * transformed_position;

    let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

    let transformed_normal = normal_matrix * vertex.normal;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: transformed_normal
    }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, current_shader: u32) -> Color {
    match current_shader {
        0 => sun_shader(fragment, uniforms),             // Sol dinámico con manchas solares
        1 => ripple_shader(fragment, uniforms),          // Shader de ondas
        2 => earth_clouds(fragment, uniforms),           // Shader Tierra con nubes
        3 => moon_shader_bright_craters(fragment, uniforms),  // Luna
        4 => dynamic_cellular_shader(fragment, uniforms), // Patrón celular dinámico
        5 => noise_shader(fragment, uniforms),           // Ruido para superficies complejas
        6 => ripple_shader(fragment, uniforms),          // Shader adicional de ondas
        _ => default_shader(fragment, uniforms),         // Fallback shader
    }
}

fn sun_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 50.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let time = uniforms.time as f32 * 0.01;

    let noise_value = uniforms.noise.get_noise_2d(x * zoom + time, y * zoom + time);

    let bright_color = Color::new(255, 255, 102); // Amarillo brillante
    let dark_spot_color = Color::new(139, 0, 0);  // Rojo oscuro
    let base_color = Color::new(255, 69, 0);      // Superficie roja/anaranjada

    let spot_threshold = 0.6;

    let noise_color = if noise_value < spot_threshold {
        bright_color
    } else {
        dark_spot_color
    };

    let final_color = base_color.lerp(&noise_color, noise_value.clamp(0.0, 1.0));
    final_color * fragment.intensity
}

fn ripple_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.vertex_position;
    let wave_speed = 0.3;
    let wave_frequency = 10.0;
    let wave_amplitude = 0.05;
    let time = uniforms.time as f32 * wave_speed;

    let distance = (pos.x.powi(2) + pos.y.powi(2)).sqrt();
    let ripple = (wave_frequency * (distance - time)).sin() * wave_amplitude;

    let base_color = Color::new(70, 130, 180); // Azul acero
    let ripple_color = Color::new(173, 216, 230); // Azul claro

    let color_factor = ripple.clamp(0.0, 1.0);
    let final_color = base_color.lerp(&ripple_color, color_factor);

    final_color * fragment.intensity
}

fn moon_shader_bright_craters(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 50.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let t = uniforms.time as f32 * 0.1;

    let pulsate = (t * 0.5).sin() * 0.05;
    let surface_noise = uniforms.noise.get_noise_2d(x * zoom + t, y * zoom + t);

    let gray_color = Color::new(200, 200, 200);
    let bright_crater_color = Color::new(220, 220, 220);
    let dynamic_color = Color::new(250, 250, 250);

    let crater_threshold = 0.4 + pulsate;

    let base_color = if surface_noise > crater_threshold {
        gray_color
    } else if surface_noise > crater_threshold - 0.1 {
        bright_crater_color
    } else {
        dynamic_color
    };

    base_color * fragment.intensity
}

fn earth_clouds(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 80.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let t = uniforms.time as f32 * 0.1;

    let surface_noise = uniforms.noise.get_noise_2d(x * zoom + t, y * zoom);

    let ocean_color = Color::new(0, 105, 148);
    let land_color = Color::new(34, 139, 34);
    let desert_color = Color::new(210, 180, 140);
    let snow_color = Color::new(255, 250, 250);

    let snow_threshold = 0.7;
    let land_threshold = 0.4;
    let desert_threshold = 0.3;

    let base_color = if y.abs() > snow_threshold {
        snow_color
    } else if surface_noise > land_threshold {
        land_color
    } else if surface_noise > desert_threshold {
        desert_color
    } else {
        ocean_color
    };

    let cloud_zoom = 100.0;
    let cloud_noise = uniforms.noise.get_noise_2d(x * cloud_zoom + t * 0.5, y * cloud_zoom + t * 0.5);

    let cloud_color = Color::new(255, 255, 255);
    let sky_gradient = Color::new(135, 206, 250);

    let cloud_intensity = cloud_noise.clamp(0.4, 0.7) - 0.4;
    let final_color = if cloud_noise > 0.6 {
        base_color.lerp(&cloud_color, cloud_intensity * 0.5)
    } else {
        base_color.lerp(&sky_gradient, 0.1)
    };

    final_color * fragment.intensity
}

fn dynamic_cellular_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 30.0;
    let flow_speed = 0.1;
    let time = uniforms.time as f32 * flow_speed;
    
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;

    let cell_noise_value = uniforms.noise.get_noise_3d(x * zoom, y * zoom, time).abs();

    let energy_color_1 = Color::new(255, 69, 0);
    let energy_color_2 = Color::new(255, 140, 0);
    let energy_color_3 = Color::new(255, 215, 0);
    let energy_color_4 = Color::new(255, 255, 153);

    let final_color = if cell_noise_value < 0.2 {
        energy_color_1
    } else if cell_noise_value < 0.5 {
        energy_color_2
    } else if cell_noise_value < 0.8 {
        energy_color_3
    } else {
        energy_color_4
    };

    final_color * fragment.intensity
}

fn noise_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.vertex_position;
    let radius = 0.1;

    let speed = 0.2;
    let time = uniforms.time as f32 * 0.01;

    let light_dir = Vec3::new(1.0, 1.0, 1.0).normalize();
    let normal = fragment.normal.normalize();
    let intensity = normal.dot(&light_dir).max(0.0);

    let mut circle_mask = 0.0;
    for i in -3..=3 {
        for j in -3..=3 {
            let offset_x = (i as f32 * 0.3) + (time * speed);
            let offset_y = (j as f32 * 0.3) + (time * speed * 0.5);

            let dist_to_circle = ((pos.x - offset_x).powi(2) + (pos.y - offset_y).powi(2)).sqrt();

            if dist_to_circle < radius {
                circle_mask = 1.0;
                break;
            }
        }
        if circle_mask == 1.0 {
            break;
        }
    }

    if circle_mask > 0.5 {
        Color::new(0, 0, 0) * intensity
    } else {
        Color::new(255, 255, 255) * (0.5 + intensity * 0.5)
    }
}

fn default_shader(fragment: &Fragment, _uniforms: &Uniforms) -> Color {
    fragment.color
}

pub fn switch_shader(current_shader: &mut u32, total_shaders: u32) {
    *current_shader = (*current_shader + 1) % total_shaders;
}