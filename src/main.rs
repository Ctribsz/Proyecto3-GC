use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;
use rand::Rng;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;
mod planet;
use rayon::prelude::*;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use triangle::triangle;
use shaders::{vertex_shader, fragment_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType};
use planet::Planet;

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise: FastNoiseLite,
}

pub struct Spaceship {
    pub position: Vec3,
    pub scale: f32,
    pub rotation: Vec3,
    pub model: Obj, // El modelo .obj cargado
    pub shader_index: u32, // Shader que usará la nave
}

impl Spaceship {
    pub fn new(model_path: &str, position: Vec3, scale: f32, rotation: Vec3, shader_index: u32) -> Self {
        Spaceship {
            position,
            scale,
            rotation,
            model: Obj::load("assets/models/ship.obj").expect("Failed to load spaceship model"),
            shader_index,
        }
    }

    pub fn update_position(&mut self, direction: Vec3) {
        self.position += direction;
    }

    pub fn get_model_matrix(&self) -> Mat4 {
        create_model_matrix(self.position, self.scale, self.rotation)
    }
}

fn create_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, cos_x, -sin_x, 0.0,
        0.0, sin_x, cos_x, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y, 0.0, sin_y, 0.0,
        0.0, 1.0, 0.0, 0.0,
        -sin_y, 0.0, cos_y, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z, cos_z, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0, 0.0, translation.x,
        0.0, scale, 0.0, translation.y,
        0.0, 0.0, scale, translation.z,
        0.0, 0.0, 0.0, 1.0,
    );

    transform_matrix * rotation_matrix
}

fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 60.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    current_shader: u32,
    framebuffer_width: usize,
    framebuffer_height: usize,
) {
    let transformed_vertices: Vec<_> = vertex_array.par_iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    let triangles: Vec<_> = transformed_vertices.par_chunks(3)
        .filter_map(|chunk| {
            if chunk.len() == 3 {
                Some([chunk[0].clone(), chunk[1].clone(), chunk[2].clone()])
            } else {
                None
            }
        })
        .collect();

    for tri in &triangles {
        let fragments = triangle(&tri[0], &tri[1], &tri[2]);

        for fragment in fragments {
            let x = fragment.position.x as usize;
            let y = fragment.position.y as usize;

            if x < framebuffer_width && y < framebuffer_height {
                let index = y * framebuffer_width + x;
                framebuffer.set_color_at_index(
                    index,
                    fragment_shader(&fragment, uniforms, current_shader).to_hex(),
                    fragment.depth,
                );
            }
        }
    }
}

fn generate_stars(count: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            (
                rng.gen_range(0..width),
                rng.gen_range(0..height),
            )
        })
        .collect()
}

fn draw_stars(framebuffer: &mut Framebuffer, stars: &[(usize, usize)]) {
    framebuffer.set_current_color(0xFFFFFF); // Blanco para las estrellas
    for &(x, y) in stars {
        let index = y * framebuffer.width + x;
        framebuffer.set_color_at_index(index, 0xFFFFFF, 1.0);
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Sistema Solar con Nave Espacial",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    let mut camera = Camera::new(
        Vec3::new(0.0, 10.0, 30.0),  // Eleva la cámara en el eje Y
        Vec3::new(0.0, 0.0, 0.0),    // Sigue apuntando al centro
        Vec3::new(0.0, 1.0, 0.0),    // Mantén el eje "arriba"
    );    

    let stars = generate_stars(500, framebuffer_width, framebuffer_height);

    let mut planets = vec![
        Planet::new("Sol", 4.0, 0.0, 0.0, 0.0, 0xFFFF00, 0),
        Planet::new("Mercurio", 0.5, 2.0, 0.04, 0.1, 0xffc300, 1),
        Planet::new("Venus", 1.0, 3.5, 0.03, 0.08, 0xe24e42, 2),
        Planet::new("Tierra", 1.2, 5.0, 0.02, 0.07, 0x0077be, 3),
        Planet::new("Marte", 0.8, 6.8, 0.01, 0.05, 0xd95d39, 4),
        Planet::new("Júpiter", 4.0, 12.0, 0.005, 0.03, 0xfff9a6, 5),
        Planet::new("Saturno", 3.5, 16.0, 0.004, 0.02, 0xc49c48, 6),
    ];

    let mut spaceship = Spaceship::new(
        "assets/models/ship.obj", // Ruta de tu modelo de nave
        Vec3::new(5.5, 1.5, 0.0),      // Cerca de la Tierra, en su órbita
        0.05,                           // Escala pequeña
        Vec3::new(0.0, 0.0, 0.0),      // Rotación inicial
        7,                             // Shader para la nave
    );

    let rotation = Vec3::new(0.0, 0.0, 0.0);
    let mut time = 0;
    let planet_obj = Obj::load("assets/models/sphere.obj").expect("Failed to load obj");
    let mut current_shader = 0;

    while window.is_open() {
        framebuffer.clear();
        draw_stars(&mut framebuffer, &stars);

        let view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
        let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);

        // Renderizar los planetas
        for planet in &mut planets {
            planet.update_position();
            let model_matrix = create_model_matrix(planet.get_position(), planet.radius, rotation);

            let uniforms = Uniforms {
                model_matrix,
                view_matrix,
                projection_matrix,
                viewport_matrix,
                time,
                noise: create_noise(),
            };

            render(
                &mut framebuffer,
                &uniforms,
                &planet_obj.get_vertex_array(),
                planet.shader_index,
                framebuffer_width,
                framebuffer_height,
            );
        }

        // Renderizar la nave espacial
        let spaceship_uniforms = Uniforms {
            model_matrix: spaceship.get_model_matrix(),
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            noise: create_noise(),
        };

        render(
            &mut framebuffer,
            &spaceship_uniforms,
            &spaceship.model.get_vertex_array(),
            spaceship.shader_index,
            framebuffer_width,
            framebuffer_height,
        );

        // Actualizar el buffer
        window
            .update_with_buffer(framebuffer.get_active_buffer(), framebuffer_width, framebuffer_height)
            .unwrap();

        framebuffer.switch_buffers();
        time += 1;
        std::thread::sleep(frame_delay);

        // Opcional: Control de la nave con teclas
        if window.is_key_down(Key::Left) {
            spaceship.update_position(Vec3::new(-0.1, 0.0, 0.0));
        }
        if window.is_key_down(Key::Right) {
            spaceship.update_position(Vec3::new(0.1, 0.0, 0.0));
        }
        if window.is_key_down(Key::Up) {
            spaceship.update_position(Vec3::new(0.0, 0.1, 0.0));
        }
        if window.is_key_down(Key::Down) {
            spaceship.update_position(Vec3::new(0.0, -0.1, 0.0));
        }
    }
}