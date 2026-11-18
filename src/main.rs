mod camera;
mod color;
mod cube;
mod diorama;
mod framebuffer;
mod light;
mod material;
mod ray_intersect;
mod raytracer;
mod scene;
mod skybox;
mod texture;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nalgebra_glm::Vec3;
use std::f32::consts::PI;
use std::time::{Duration, Instant};

use crate::camera::Camera;
use crate::framebuffer::Framebuffer;
use crate::scene::Scene;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

// Rayos por lado de cada píxel cuando la cámara está quieta (2 → 2 × 2 = 4
// rayos, bordes suaves). Mientras se mueve se usa 1 para que sea fluido.
const STILL_SAMPLES: usize = 2;

const ROTATION_SPEED: f32 = PI / 90.0;
const AUTO_ROTATION_SPEED: f32 = PI / 240.0;
const ZOOM_SPEED: f32 = 0.6;
const MIN_DISTANCE: f32 = 9.0;
const MAX_DISTANCE: f32 = 45.0;

const TITLE: &str = "Diorama con Raytracing";
const CONTROLS: &str = "←→ girar · ↑↓ inclinar · W/S o rueda: zoom · Espacio: giro automático · R: reiniciar";

fn initial_camera() -> Camera {
    let center = diorama::center();
    let offset = Vec3::new(0.75, 0.55, 1.0).normalize() * 22.0;
    Camera::new(center + offset, center, Vec3::new(0.0, 1.0, 0.0))
}

fn render(framebuffer: &mut Framebuffer, scene: &Scene, camera: &Camera, samples: usize) {
    let (w, h) = (framebuffer.width, framebuffer.height);
    raytracer::render(&mut framebuffer.buffer, w, h, scene, camera, samples);
}

// Modo sin ventana: calcula varias vistas y las guarda como imágenes.
// Se usa con `cargo run --release -- --capturas`.
fn save_screenshots(scene: &Scene) {
    std::fs::create_dir_all("capturas").expect("No se pudo crear la carpeta capturas");
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let views: [(&str, f32, f32, f32); 4] = [
        ("vista_frente", 0.0, 0.0, 0.0),
        ("vista_lateral", -PI / 2.0, 0.0, 0.0),
        ("vista_atras", PI * 0.85, 0.1, -4.0),
        ("vista_cerca", 0.25, 0.25, -12.0),
    ];
    for (name, yaw, pitch, zoom) in views {
        let mut camera = initial_camera();
        camera.orbit(yaw, pitch);
        camera.zoom(zoom, MIN_DISTANCE, MAX_DISTANCE);
        let start = Instant::now();
        render(&mut framebuffer, scene, &camera, STILL_SAMPLES);
        let path = format!("capturas/{}.bmp", name);
        framebuffer.save_bmp(&path).expect("No se pudo guardar la captura");
        println!("{} ({} ms)", path, start.elapsed().as_millis());
    }
}

// Modo sin ventana: una vuelta completa alrededor del diorama, cuadro por
// cuadro, para armar un video. Se usa con `cargo run --release -- --video 240`.
fn save_video_frames(scene: &Scene, frames: usize) {
    std::fs::create_dir_all("frames").expect("No se pudo crear la carpeta frames");
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut camera = initial_camera();
    for i in 0..frames {
        // Acercamiento y alejamiento suaves mientras da la vuelta.
        let phase = i as f32 / frames as f32 * 2.0 * PI;
        let distance = 22.0 - 5.0 * phase.sin().max(0.0);
        let mut frame_camera = Camera::new(camera.eye, camera.center, camera.up);
        frame_camera.zoom(distance - frame_camera.distance(), MIN_DISTANCE, MAX_DISTANCE);
        render(&mut framebuffer, scene, &frame_camera, STILL_SAMPLES);
        framebuffer
            .save_bmp(&format!("frames/frame_{:04}.bmp", i))
            .expect("No se pudo guardar el cuadro");
        camera.orbit(2.0 * PI / frames as f32, 0.0);
        if i % 20 == 0 {
            println!("cuadro {} de {}", i, frames);
        }
    }
}

fn main() {
    let scene = match diorama::build() {
        Ok(scene) => scene,
        Err(error) => {
            eprintln!("Error al cargar el diorama: {}", error);
            std::process::exit(1);
        }
    };

    println!("Materiales del diorama:");
    for material in scene.materials.iter().skip(1) {
        println!(
            "  {:<9} albedo {:.2}  specular {:.2}  transparencia {:.2}  reflectividad {:.2}  índice {:.2}",
            material.name,
            material.albedo,
            material.specular,
            material.transparency,
            material.reflectivity,
            material.refractive_index
        );
    }

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--capturas") {
        save_screenshots(&scene);
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--video") {
        let frames = args.get(i + 1).and_then(|n| n.parse().ok()).unwrap_or(240);
        save_video_frames(&scene, frames);
        return;
    }

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);

    let mut window = Window::new(TITLE, WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    window.set_target_fps(60);

    let mut camera = initial_camera();
    let mut auto_rotate = false;
    // La primera imagen es la completa.
    let mut needs_full_render = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut moved = false;

        let orbit_keys = [
            (Key::Left, ROTATION_SPEED, 0.0),
            (Key::A, ROTATION_SPEED, 0.0),
            (Key::Right, -ROTATION_SPEED, 0.0),
            (Key::D, -ROTATION_SPEED, 0.0),
            (Key::Up, 0.0, -ROTATION_SPEED),
            (Key::Down, 0.0, ROTATION_SPEED),
        ];
        for (key, delta_yaw, delta_pitch) in orbit_keys {
            if window.is_key_down(key) {
                camera.orbit(delta_yaw, delta_pitch);
                moved = true;
            }
        }

        if window.is_key_down(Key::W) {
            camera.zoom(-ZOOM_SPEED, MIN_DISTANCE, MAX_DISTANCE);
            moved = true;
        }
        if window.is_key_down(Key::S) {
            camera.zoom(ZOOM_SPEED, MIN_DISTANCE, MAX_DISTANCE);
            moved = true;
        }
        if let Some((_, scroll)) = window.get_scroll_wheel() {
            if scroll != 0.0 {
                camera.zoom(-scroll.signum() * ZOOM_SPEED * 2.0, MIN_DISTANCE, MAX_DISTANCE);
                moved = true;
            }
        }

        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            auto_rotate = !auto_rotate;
        }
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            camera = initial_camera();
            moved = true;
        }
        if auto_rotate {
            camera.orbit(AUTO_ROTATION_SPEED, 0.0);
            moved = true;
        }

        if moved {
            let start = Instant::now();
            render(&mut framebuffer, &scene, &camera, 1);
            needs_full_render = true;
            window.set_title(&format!(
                "{} | en movimiento {} ms | {}",
                TITLE,
                start.elapsed().as_millis(),
                CONTROLS
            ));
        } else if needs_full_render {
            let start = Instant::now();
            render(&mut framebuffer, &scene, &camera, STILL_SAMPLES);
            needs_full_render = false;
            window.set_title(&format!(
                "{} | imagen suavizada {} ms | {}",
                TITLE,
                start.elapsed().as_millis(),
                CONTROLS
            ));
        } else {
            // Nada cambió: no hace falta recalcular, solo esperar un poco.
            std::thread::sleep(Duration::from_millis(10));
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
