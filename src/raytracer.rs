use crate::camera::Camera;
use crate::color::Color;
use crate::ray_intersect::Intersect;
use crate::scene::Scene;
use nalgebra_glm::{dot, Vec3};
use std::f32::consts::PI;
use std::sync::Mutex;
use std::thread;

// Campo de visión vertical de la cámara.
pub const FOV: f32 = PI / 3.0;

// Cuántas veces puede rebotar o atravesar un rayo antes de rendirse.
const MAX_DEPTH: u32 = 4;

// Pequeño empujón para que los rayos secundarios no vuelvan a chocar con la
// misma superficie de la que salen (por errores de redondeo).
const BIAS: f32 = 1e-3;

const AIR_REFRACTIVE_INDEX: f32 = 1.0;

// Luz del cielo que ilumina las caras aunque no les dé el sol.
const SKY_AMBIENT: Color = Color { r: 0.46, g: 0.52, b: 0.62 };

// Espejo: el rayo sale con el mismo ángulo con el que llegó, pero del otro
// lado de la normal.
pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - normal * (2.0 * dot(incident, normal))
}

// Ley de Snell: al pasar de un medio a otro, el rayo se dobla según la razón
// entre sus índices de refracción. Devuelve None cuando el ángulo es tan
// rasante que la luz no puede salir (reflexión interna total).
pub fn refract(incident: &Vec3, normal: &Vec3, refractive_index: f32) -> Option<Vec3> {
    let cosine = dot(incident, normal).clamp(-1.0, 1.0);

    // Si el rayo va contra la normal, entra al material; si va a favor, sale.
    let (outside_index, inside_index, surface_normal) = if cosine < 0.0 {
        (AIR_REFRACTIVE_INDEX, refractive_index, *normal)
    } else {
        (refractive_index, AIR_REFRACTIVE_INDEX, -normal)
    };

    let eta = outside_index / inside_index;
    let cosine_incident = cosine.abs();
    let cosine_squared = 1.0 - eta * eta * (1.0 - cosine_incident * cosine_incident);

    if cosine_squared < 0.0 {
        return None;
    }

    Some(incident * eta + surface_normal * (eta * cosine_incident - cosine_squared.sqrt()))
}

// Aproximación de Schlick a las ecuaciones de Fresnel: qué fracción de la
// luz se refleja en una superficie transparente. Mirando el agua de frente
// casi todo la atraviesa; mirándola de lado, se vuelve un espejo.
fn fresnel(cosine: f32, from_index: f32, to_index: f32) -> f32 {
    let mut cosine = cosine;
    if from_index > to_index {
        let ratio = from_index / to_index;
        let sine_squared = ratio * ratio * (1.0 - cosine * cosine);
        if sine_squared > 1.0 {
            return 1.0;
        }
        cosine = (1.0 - sine_squared).sqrt();
    }
    let r0 = ((from_index - to_index) / (from_index + to_index)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

// Color propio de una superficie iluminada (sin reflejos ni transparencias):
// textura × (luz del cielo + luz del sol) + brillo especular.
fn shade(scene: &Scene, hit: &Intersect, ray_direction: &Vec3, facing: &Vec3) -> Color {
    let material = &scene.materials[hit.material as usize];
    let texture = &scene.textures[material.texture_for(&hit.normal)];
    let base = texture.sample(hit.u, hit.v);
    let light = &scene.light;

    // Luz del cielo, más fuerte en las caras de arriba que en las de abajo
    // (como en Minecraft), y apagada en las esquinas por la oclusión.
    let face_light = 0.8 + 0.2 * facing.y;
    let occlusion = 0.35 + 0.65 * scene.ambient_occlusion(hit);
    let ambient = base * SKY_AMBIENT * (face_light * occlusion);

    // Difusa (Lambert): la cara recibe más sol cuanto más directo le llega.
    let sun_angle = dot(facing, &light.direction).max(0.0);
    if sun_angle <= 0.0 {
        return ambient;
    }
    let visibility = scene.sun_visibility(&(hit.point + facing * BIAS));
    let sunlight = light.intensity * visibility;
    let diffuse = base * light.color * (material.albedo * sun_angle * sunlight);

    // Especular (Phong): brillo donde el reflejo del sol apunta a la cámara.
    let sun_reflection = reflect(&-light.direction, facing);
    let toward_camera = -ray_direction;
    let specular_intensity = dot(&toward_camera, &sun_reflection)
        .max(0.0)
        .powf(material.shininess);
    let specular = light.color * (specular_intensity * material.specular * sunlight);

    ambient + diffuse + specular
}

// Color que ve un rayo. Si choca con un bloque, combina el color propio de
// la superficie con lo que se ve reflejado en ella y lo que se ve a través
// de ella, según las proporciones de su material. Si no choca, ve el cielo.
pub fn cast_ray(scene: &Scene, ray_origin: &Vec3, ray_direction: &Vec3, depth: u32) -> Color {
    let Some(hit) = scene.trace(ray_origin, ray_direction) else {
        return scene.skybox.sample(ray_direction);
    };

    let material = &scene.materials[hit.material as usize];

    // `facing` es la normal del lado por el que llega el rayo: igual a la
    // normal si el rayo entra al bloque, invertida si está saliendo de él.
    let entering = dot(ray_direction, &hit.normal) < 0.0;
    let facing = if entering { hit.normal } else { -hit.normal };

    let local = shade(scene, &hit, ray_direction, &facing);

    if depth >= MAX_DEPTH {
        return local;
    }

    // Reparto de la luz entre reflejo, refracción y color propio.
    let mut reflect_weight = material.reflectivity;
    let mut refract_weight = 0.0;
    let mut refracted = None;

    if material.transparency > 0.0 {
        let (from, to) = if entering {
            (AIR_REFRACTIVE_INDEX, material.refractive_index)
        } else {
            (material.refractive_index, AIR_REFRACTIVE_INDEX)
        };
        let reflected_part = fresnel(-dot(ray_direction, &facing), from, to);
        reflect_weight += material.transparency * reflected_part;
        refract_weight = material.transparency * (1.0 - reflected_part);
        refracted = refract(ray_direction, &hit.normal, material.refractive_index);

        // Reflexión interna total: lo que no puede salir se refleja.
        if refracted.is_none() {
            reflect_weight += refract_weight;
            refract_weight = 0.0;
        }
    }

    let own_weight = (1.0 - reflect_weight - refract_weight).max(0.0);
    let mut color = local * own_weight;

    if reflect_weight > 0.01 {
        let direction = reflect(ray_direction, &facing).normalize();
        let origin = hit.point + facing * BIAS;
        color = color + cast_ray(scene, &origin, &direction, depth + 1) * reflect_weight;
    }

    if let Some(direction) = refracted {
        if refract_weight > 0.01 {
            let direction = direction.normalize();
            let origin = hit.point - facing * BIAS;
            // La luz que atraviesa agua o vidrio toma un poco de su color.
            let texture = &scene.textures[material.texture_for(&hit.normal)];
            let tint = Color::WHITE.lerp(&texture.sample(hit.u, hit.v), 0.35);
            color = color
                + cast_ray(scene, &origin, &direction, depth + 1) * tint * refract_weight;
        }
    }

    color
}

// Dirección del rayo que pasa por el punto (x + offset_x, y + offset_y) del
// píxel; con offsets de 0.5 pasa por su centro.
fn primary_ray(
    camera: &Camera,
    x: usize,
    y: usize,
    offset_x: f32,
    offset_y: f32,
    width: usize,
    height: usize,
) -> Vec3 {
    let w = width as f32;
    let h = height as f32;
    let aspect_ratio = w / h;
    let perspective_scale = (FOV / 2.0).tan();

    let screen_x = (2.0 * (x as f32 + offset_x)) / w - 1.0;
    let screen_y = -(2.0 * (y as f32 + offset_y)) / h + 1.0;

    let screen_x = screen_x * aspect_ratio * perspective_scale;
    let screen_y = screen_y * perspective_scale;

    let direction = Vec3::new(screen_x, screen_y, -1.0).normalize();
    camera.basis_change(&direction)
}

// Color final de un píxel. Con `samples` = 1 se lanza un solo rayo por el
// centro; con `samples` = 2 se lanzan 2 × 2 rayos repartidos dentro del
// píxel y se promedian, lo que suaviza los bordes dentados (antialiasing).
fn pixel_color(
    scene: &Scene,
    camera: &Camera,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    samples: usize,
) -> Color {
    let mut sum = Color::new(0.0, 0.0, 0.0);
    for sy in 0..samples {
        for sx in 0..samples {
            let offset_x = (sx as f32 + 0.5) / samples as f32;
            let offset_y = (sy as f32 + 0.5) / samples as f32;
            let direction = primary_ray(camera, x, y, offset_x, offset_y, width, height);
            let color = cast_ray(scene, &camera.eye, &direction, 0);
            // Se recorta antes de promediar para que un brillo muy intenso no
            // "manche" a los rayos vecinos.
            sum = sum + Color::new(color.r.min(1.0), color.g.min(1.0), color.b.min(1.0));
        }
    }
    sum * (1.0 / (samples * samples) as f32)
}

// Calcula la imagen completa repartiendo el trabajo entre todos los núcleos
// del procesador. La imagen se divide en franjas de pocas filas; cada hilo
// toma la siguiente franja libre, la pinta y pide otra, hasta que no quedan.
pub fn render(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    scene: &Scene,
    camera: &Camera,
    samples: usize,
) {
    const ROWS_PER_CHUNK: usize = 4;

    let threads = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let chunks = Mutex::new(buffer.chunks_mut(width * ROWS_PER_CHUNK).enumerate());

    thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                // Solo se bloquea el tiempo justo para tomar la siguiente franja.
                let next = chunks.lock().unwrap().next();
                let Some((chunk_index, chunk)) = next else {
                    break;
                };

                for (i, pixel) in chunk.iter_mut().enumerate() {
                    let x = i % width;
                    let y = chunk_index * ROWS_PER_CHUNK + i / width;
                    if y >= height {
                        break;
                    }
                    *pixel = pixel_color(scene, camera, x, y, width, height, samples).to_hex();
                }
            });
        }
    });
}
