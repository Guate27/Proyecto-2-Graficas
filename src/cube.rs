use crate::ray_intersect::{Intersect, RayIntersect};
use nalgebra_glm::Vec3;

// Un cubo alineado con los ejes queda definido por dos esquinas opuestas:
// `min` (la de valores más pequeños en x, y, z) y `max` (la de valores más
// grandes). Todo punto con min <= punto <= max en los tres ejes está adentro.
pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: u8,
}

// Resultado del método de las franjas: las distancias de entrada y salida, y
// en qué eje ocurrió cada una (para saber qué cara se tocó).
pub struct SlabHit {
    pub t_near: f32,
    pub t_far: f32,
    pub near_axis: usize,
    pub far_axis: usize,
}

impl Cube {
    // Caja con esquinas arbitrarias (no necesariamente un cubo perfecto).
    pub fn from_corners(min: Vec3, max: Vec3, material: u8) -> Self {
        Cube { min, max, material }
    }

    // Bloque de la cuadrícula: ocupa de (x, y, z) a (x + 1, y + 1, z + 1).
    pub fn from_cell(x: i32, y: i32, z: i32, material: u8) -> Self {
        let min = Vec3::new(x as f32, y as f32, z as f32);
        Cube {
            min,
            max: min + Vec3::new(1.0, 1.0, 1.0),
            material,
        }
    }

    // Método de las franjas (slabs). El cubo es el cruce de tres franjas:
    // entre sus dos paredes en x, entre piso y techo en y, y entre su cara
    // frontal y trasera en z. Para cada franja se calcula el intervalo de
    // distancias [t1, t2] en el que el rayo está dentro de ella. El rayo
    // está dentro del cubo solo donde los tres intervalos se traslapan.
    pub fn slab(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<SlabHit> {
        // t_near: la entrada más tardía a una franja (el rayo ya entró a todas).
        // t_far: la salida más temprana de una franja (el rayo ya salió de una).
        let mut t_near = f32::NEG_INFINITY;
        let mut t_far = f32::INFINITY;
        let mut near_axis = 0;
        let mut far_axis = 0;

        // axis = 0 es x, 1 es y, 2 es z.
        for axis in 0..3 {
            let origin = ray_origin[axis];
            let direction = ray_direction[axis];

            // Un rayo paralelo a la franja nunca entra ni sale de ella:
            // o va por dentro todo el tiempo, o nunca la toca.
            if direction.abs() < 1e-8 {
                if origin < self.min[axis] || origin > self.max[axis] {
                    return None;
                }
                continue;
            }

            // Despejando t en origen + t · dirección = pared, se obtiene la
            // distancia a la que el rayo cruza cada una de las dos paredes.
            let mut t1 = (self.min[axis] - origin) / direction;
            let mut t2 = (self.max[axis] - origin) / direction;

            // Si el rayo viaja hacia valores negativos del eje, cruza primero
            // la pared `max`; el intercambio deja siempre t1 = entrada, t2 = salida.
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }

            // Además de la distancia, se recuerda qué eje la produjo: la cara
            // por la que entra el rayo es la del eje que dio el t_near final.
            if t1 > t_near {
                t_near = t1;
                near_axis = axis;
            }
            if t2 < t_far {
                t_far = t2;
                far_axis = axis;
            }

            // Si ya salió de una franja antes de entrar a otra, los
            // intervalos no se traslapan: el rayo pasa de largo.
            if t_near > t_far {
                return None;
            }
        }

        // Todo el traslape está detrás de la cámara: el cubo no se ve.
        if t_far < 0.0 {
            return None;
        }

        Some(SlabHit {
            t_near,
            t_far,
            near_axis,
            far_axis,
        })
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let hit = self.slab(ray_origin, ray_direction)?;

        // Si el origen está fuera, lo visible es la entrada (t_near). Si está
        // dentro del cubo, t_near es negativo y lo visible es la salida.
        let (distance, axis, outward) = if hit.t_near > 0.0 {
            (hit.t_near, hit.near_axis, false)
        } else {
            (hit.t_far, hit.far_axis, true)
        };

        // La normal apunta hacia afuera de la cara tocada. Al entrar, esa cara
        // mira en contra del rayo; al salir, mira en el mismo sentido.
        let mut normal = Vec3::zeros();
        let sign = ray_direction[axis].signum();
        normal[axis] = if outward { sign } else { -sign };

        let point = ray_origin + ray_direction * distance;
        let (u, v) = face_uv(&point, &self.min, &self.max, axis);

        Some(Intersect {
            point,
            normal,
            distance,
            u,
            v,
            material: self.material,
        })
    }
}

// Coordenadas de textura dentro de una cara: se usan las dos coordenadas que
// cambian a lo largo de esa cara, medidas desde la esquina `min`.
pub fn face_uv(point: &Vec3, min: &Vec3, max: &Vec3, axis: usize) -> (f32, f32) {
    let size = max - min;
    let local = point - min;
    let (u, v) = match axis {
        0 => (local.z / size.z, local.y / size.y),
        1 => (local.x / size.x, local.z / size.z),
        _ => (local.x / size.x, local.y / size.y),
    };
    (u.clamp(0.0, 0.9999), v.clamp(0.0, 0.9999))
}
