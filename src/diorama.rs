use crate::color::Color;
use crate::light::Light;
use crate::material::Material;
use crate::scene::{Scene, AIR};
use crate::skybox::Skybox;
use crate::texture::Texture;
use nalgebra_glm::Vec3;

// Números de material que se guardan en cada celda de la cuadrícula.
pub const PASTO: u8 = 1;
pub const TIERRA: u8 = 2;
pub const PIEDRA: u8 = 3;
pub const ARENA: u8 = 4;
pub const TRONCO: u8 = 5;
pub const TABLONES: u8 = 6;
pub const HOJAS: u8 = 7;
pub const AGUA: u8 = 8;
pub const VIDRIO: u8 = 9;
pub const AMATISTA: u8 = 10;

// Las imágenes se buscan junto al proyecto, sin importar desde qué carpeta
// se ejecute el programa.
const ASSETS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

// Orden en que se cargan las texturas; su posición en esta lista es el
// número que usan los materiales.
const TEXTURE_FILES: [&str; 12] = [
    "pasto_arriba",  // 0
    "pasto_lado",    // 1
    "tierra",        // 2
    "piedra",        // 3
    "arena",         // 4
    "tronco_lado",   // 5
    "tronco_arriba", // 6
    "tablones",      // 7
    "hojas",         // 8
    "agua",          // 9
    "vidrio",        // 10
    "amatista",      // 11
];

#[allow(clippy::too_many_arguments)]
fn material(
    name: &'static str,
    top: usize,
    side: usize,
    bottom: usize,
    albedo: f32,
    specular: f32,
    shininess: f32,
    transparency: f32,
    reflectivity: f32,
    refractive_index: f32,
) -> Material {
    Material {
        name,
        texture_top: top,
        texture_side: side,
        texture_bottom: bottom,
        albedo,
        specular,
        shininess,
        transparency,
        reflectivity,
        refractive_index,
    }
}

fn materials() -> Vec<Material> {
    vec![
        //        nombre      arriba lado abajo albedo specular brillo transp. reflej. índice
        material("aire",       0,  0,  0, 0.00, 0.00,   1.0, 0.00, 0.00, 1.00),
        material("pasto",      0,  1,  2, 0.90, 0.05,   8.0, 0.00, 0.00, 1.00),
        material("tierra",     2,  2,  2, 0.88, 0.03,   6.0, 0.00, 0.00, 1.00),
        material("piedra",     3,  3,  3, 0.85, 0.20,  24.0, 0.00, 0.00, 1.00),
        material("arena",      4,  4,  4, 0.92, 0.08,  10.0, 0.00, 0.00, 1.00),
        material("tronco",     6,  5,  6, 0.85, 0.10,  12.0, 0.00, 0.00, 1.00),
        material("tablones",   7,  7,  7, 0.88, 0.18,  30.0, 0.00, 0.00, 1.00),
        material("hojas",      8,  8,  8, 0.82, 0.06,   8.0, 0.00, 0.00, 1.00),
        material("agua",       9,  9,  9, 0.30, 0.90, 150.0, 0.70, 0.08, 1.33),
        material("vidrio",    10, 10, 10, 0.15, 1.00, 300.0, 0.82, 0.08, 1.50),
        material("amatista",  11, 11, 11, 0.75, 1.00, 180.0, 0.00, 0.30, 1.00),
    ]
}

// Número pseudoaleatorio fijo entre 0 y 1 para cada (x, z): siempre da lo
// mismo para la misma celda, así la isla no cambia entre ejecuciones.
fn hash(x: i32, z: i32, seed: i32) -> f32 {
    let mut h = (x.wrapping_mul(374_761_393) ^ z.wrapping_mul(668_265_263) ^ seed.wrapping_mul(1_274_126_177)) as u32;
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h % 10_000) as f32 / 10_000.0
}

fn fill_box(scene: &mut Scene, min: [i32; 3], max: [i32; 3], material: u8) {
    for y in min[1]..=max[1] {
        for z in min[2]..=max[2] {
            for x in min[0]..=max[0] {
                scene.set(x, y, z, material);
            }
        }
    }
}

pub const SIZE: [i32; 3] = [20, 17, 20];
pub const GROUND: i32 = 6;

// Centro del diorama: hacia donde mira la cámara.
pub fn center() -> Vec3 {
    Vec3::new(10.0, 7.5, 10.0)
}

pub fn build() -> Result<Scene, String> {
    let mut textures = Vec::new();
    for name in TEXTURE_FILES {
        textures.push(Texture::load_bmp(&format!("{}/textures/{}.bmp", ASSETS, name))?);
    }
    let skybox = Skybox::load(&format!("{}/skybox", ASSETS))?;

    // El sol coincide con el que aparece pintado en el skybox.
    let sun = Light::new(Vec3::new(0.7, 0.62, -0.3), Color::new(1.0, 0.96, 0.88), 1.1);

    let mut scene = Scene::new(SIZE, materials(), textures, skybox, sun);

    build_island(&mut scene);
    build_pond(&mut scene);
    build_house(&mut scene);
    build_plaza(&mut scene);
    build_tree(&mut scene, 4, 14, 2.4);
    build_tree(&mut scene, 15, 5, 2.1);

    Ok(scene)
}

// Altura del pasto en cada columna: plano a la altura GROUND con una loma
// donde va el árbol principal.
fn ground_height(x: i32, z: i32) -> i32 {
    let d = ((x as f32 + 0.5 - 4.5).powi(2) + (z as f32 + 0.5 - 14.5).powi(2)).sqrt();
    if d <= 1.4 {
        GROUND + 2
    } else if d <= 2.6 {
        GROUND + 1
    } else {
        GROUND
    }
}

// Isla flotante: piedra que va haciendo grande hacia abajo, con dos
// capas de tierra y pasto encima.
fn build_island(scene: &mut Scene) {
    let c = 9.5;
    for z in 0..SIZE[2] {
        for x in 0..SIZE[0] {
            let dx = (x as f32 + 0.5 - c).abs();
            let dz = (z as f32 + 0.5 - c).abs();
            let r = dx.max(dz);

            // Silueta cuadrada con esquinas recortadas y bordes irregulares.
            if r > 7.0 || dx + dz > 11.5 || (r > 6.0 && hash(x, z, 1) < 0.22) {
                continue;
            }

            let top = ground_height(x, z);
            let bottom = ((r * 0.8 + hash(x, z, 2) * 1.8) as i32).clamp(0, top - 2);

            for y in bottom..=top {
                let material = if y == top {
                    PASTO
                } else if y >= top - 2 {
                    TIERRA
                } else if hash(x * 7 + y, z, 3) < 0.12 {
                    TIERRA
                } else {
                    PIEDRA
                };
                scene.set(x, y, z, material);
            }
        }
    }
}

// Estanque con fondo de arena, un arroyo que llega a la orilla de la isla y
// una cascada que cae por el borde.
fn build_pond(scene: &mut Scene) {
    let (cx, cz) = (12.5, 12.0);
    for z in 0..SIZE[2] {
        for x in 0..SIZE[0] {
            let ex = (x as f32 + 0.5 - cx) / 2.8;
            let ez = (z as f32 + 0.5 - cz) / 2.3;
            let e = ex * ex + ez * ez;
            if scene.get(x, GROUND, z) == AIR {
                continue;
            }
            if e <= 1.0 {
                scene.set(x, GROUND, z, AGUA);
                scene.set(x, GROUND - 1, z, AGUA);
                scene.set(x, GROUND - 2, z, ARENA);
            } else if e <= 1.9 {
                scene.set(x, GROUND, z, ARENA);
            }
        }
    }

    // Arroyo hacia la orilla +x.
    let stream_z = 12;
    for x in 15..=16 {
        scene.set(x, GROUND, stream_z, AGUA);
        scene.set(x, GROUND - 1, stream_z, ARENA);
        scene.set(x, GROUND, stream_z - 1, ARENA);
        scene.set(x, GROUND, stream_z + 1, ARENA);
    }

    // Cascada: columna de agua por fuera de la isla.
    for y in 0..=GROUND {
        scene.set(17, y, stream_z, AGUA);
    }
}

// Casa de tablones con esquinas de tronco, ventanas de vidrio y techo de
// piedra escalonado.
fn build_house(scene: &mut Scene) {
    let (x0, x1, z0, z1) = (3, 8, 3, 8);
    let (floor, wall_top) = (GROUND, GROUND + 3);

    fill_box(scene, [x0, floor, z0], [x1, floor, z1], TABLONES);

    for y in floor + 1..=wall_top {
        for z in z0..=z1 {
            for x in x0..=x1 {
                let on_x = x == x0 || x == x1;
                let on_z = z == z0 || z == z1;
                if on_x && on_z {
                    scene.set(x, y, z, TRONCO);
                } else if on_x || on_z {
                    scene.set(x, y, z, TABLONES);
                } else {
                    scene.set(x, y, z, AIR);
                }
            }
        }
    }

    // Puerta en la pared del frente (z = z1).
    scene.set(6, floor + 1, z1, AIR);
    scene.set(6, floor + 2, z1, AIR);

    // Ventanas.
    scene.set(4, floor + 2, z1, VIDRIO);
    for z in 4..=6 {
        scene.set(x1, floor + 2, z, VIDRIO);
    }
    for x in 4..=6 {
        scene.set(x, floor + 2, z0, VIDRIO);
    }

    // Un bloque de amatista dentro de la casa, visible por las ventanas.
    scene.set(5, floor + 1, 5, AMATISTA);

    // Techo en forma de pirámide escalonada.
    for (level, inset) in (0..4).zip([-1, 0, 1, 2]) {
        let y = wall_top + 1 + level;
        fill_box(
            scene,
            [x0 + inset, y, z0 + inset],
            [x1 - inset, y, z1 - inset],
            if level == 0 { TABLONES } else { PIEDRA },
        );
    }
}

// Plaza de piedra frente a la casa con un monumento de amatista y un
// camino hasta la puerta.
fn build_plaza(scene: &mut Scene) {
    for z in 9..=12 {
        scene.set(6, GROUND, z, PIEDRA);
    }
    for z in 12..=15 {
        for x in 6..=9 {
            if scene.get(x, GROUND, z) == PASTO {
                scene.set(x, GROUND, z, PIEDRA);
            }
        }
    }
    fill_box(scene, [7, GROUND + 1, 13], [8, GROUND + 2, 14], AMATISTA);
    scene.set(7, GROUND + 3, 13, AMATISTA);
}

fn build_tree(scene: &mut Scene, x: i32, z: i32, radius: f32) {
    let ground = (0..SIZE[1]).rev().find(|&y| scene.get(x, y, z) != AIR).unwrap_or(GROUND);
    let trunk_top = ground + 4;

    for y in ground + 1..=trunk_top {
        scene.set(x, y, z, TRONCO);
    }

    let (cx, cy, cz) = (x as f32 + 0.5, trunk_top as f32 + 0.5, z as f32 + 0.5);
    let r = radius.ceil() as i32;
    for y in trunk_top - 1..=trunk_top + r {
        for dz in -r..=r {
            for dx in -r..=r {
                let (px, pz) = (x + dx, z + dz);
                let d = ((px as f32 + 0.5 - cx).powi(2)
                    + (y as f32 + 0.5 - cy).powi(2) * 1.3
                    + (pz as f32 + 0.5 - cz).powi(2))
                .sqrt();
                if d <= radius && scene.get(px, y, pz) == AIR && hash(px * 3 + y, pz, 9) > 0.08 {
                    scene.set(px, y, pz, HOJAS);
                }
            }
        }
    }
}
