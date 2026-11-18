use crate::cube::{face_uv, Cube};
use crate::light::Light;
use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::skybox::Skybox;
use crate::texture::Texture;
use nalgebra_glm::{dot, Vec3};

// Material 0 se reserva para "aire": una celda vacía.
pub const AIR: u8 = 0;

// El diorama es una cuadrícula 3D de celdas de 1×1×1, como en Minecraft.
// Cada celda guarda el número de su material (0 si está vacía).
pub struct Scene {
    pub size: [i32; 3],
    cells: Vec<u8>,
    // Caja que encierra toda la cuadrícula; un rayo que no la toca no puede
    // chocar con ningún bloque.
    bounds: Cube,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub skybox: Skybox,
    pub light: Light,
}

impl Scene {
    pub fn new(
        size: [i32; 3],
        materials: Vec<Material>,
        textures: Vec<Texture>,
        skybox: Skybox,
        light: Light,
    ) -> Self {
        let cell_count = (size[0] * size[1] * size[2]) as usize;
        let bounds = Cube::from_corners(
            Vec3::zeros(),
            Vec3::new(size[0] as f32, size[1] as f32, size[2] as f32),
            AIR,
        );
        Scene {
            size,
            cells: vec![AIR; cell_count],
            bounds,
            materials,
            textures,
            skybox,
            light,
        }
    }

    fn inside_grid(&self, x: i32, y: i32, z: i32) -> bool {
        x >= 0 && y >= 0 && z >= 0 && x < self.size[0] && y < self.size[1] && z < self.size[2]
    }

    fn index(&self, x: i32, y: i32, z: i32) -> usize {
        ((y * self.size[2] + z) * self.size[0] + x) as usize
    }

    // Material de la celda; fuera de la cuadrícula todo es aire.
    pub fn get(&self, x: i32, y: i32, z: i32) -> u8 {
        if self.inside_grid(x, y, z) {
            self.cells[self.index(x, y, z)]
        } else {
            AIR
        }
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, material: u8) {
        if self.inside_grid(x, y, z) {
            let i = self.index(x, y, z);
            self.cells[i] = material;
        }
    }

    pub fn is_transparent(&self, material: u8) -> bool {
        material != AIR && self.materials[material as usize].transparency > 0.0
    }

    // Un bloque opaco tapa la luz por completo (sirve para las sombras de
    // contacto en las esquinas).
    fn is_opaque(&self, x: i32, y: i32, z: i32) -> bool {
        let m = self.get(x, y, z);
        m != AIR && !self.is_transparent(m)
    }

    // Busca el primer bloque que toca un rayo.
    //
    // Revisar los cientos de bloques uno por uno con el método de las franjas
    // sería muy lento. En su lugar, el rayo "camina" por la cuadrícula celda
    // por celda, siempre a la siguiente celda que cruza (algoritmo DDA, como
    // en el raycaster), y solo se calcula la intersección exacta con el cubo
    // de la primera celda ocupada que encuentra.
    //
    // Si el rayo empieza dentro de agua o vidrio, las celdas de ese mismo
    // material se atraviesan sin chocar (no hay pared entre dos bloques de
    // agua) y el choque es la superficie por donde sale.
    pub fn trace(&self, origin: &Vec3, direction: &Vec3) -> Option<Intersect> {
        let bounds_hit = self.bounds.slab(origin, direction)?;
        let t_start = bounds_hit.t_near.max(0.0);

        // Celda donde empieza el recorrido.
        let start = origin + direction * t_start;
        let mut cell = [0i32; 3];
        for axis in 0..3 {
            cell[axis] = (start[axis].floor() as i32).clamp(0, self.size[axis] - 1);
        }

        // Medio en el que viaja el rayo: agua o vidrio si nace dentro de uno.
        let start_material = self.get(cell[0], cell[1], cell[2]);
        let medium = if t_start == 0.0 && self.is_transparent(start_material) {
            start_material
        } else {
            AIR
        };

        // Para cada eje: hacia dónde se avanza (+1 o -1), cuánto t cuesta
        // cruzar una celda completa, y en qué t se cruza la siguiente pared.
        let mut step = [0i32; 3];
        let mut t_delta = [f32::INFINITY; 3];
        let mut t_max = [f32::INFINITY; 3];
        for axis in 0..3 {
            let d = direction[axis];
            if d > 1e-8 {
                step[axis] = 1;
                t_delta[axis] = 1.0 / d;
                t_max[axis] = ((cell[axis] + 1) as f32 - origin[axis]) / d;
            } else if d < -1e-8 {
                step[axis] = -1;
                t_delta[axis] = -1.0 / d;
                t_max[axis] = (cell[axis] as f32 - origin[axis]) / d;
            }
        }

        // Eje por el que se entró a la celda actual (None en la celda inicial
        // cuando el rayo nace dentro de la cuadrícula).
        let mut entry_axis = if t_start > 0.0 {
            Some(bounds_hit.near_axis)
        } else {
            None
        };
        let mut t = t_start;

        loop {
            let material = self.get(cell[0], cell[1], cell[2]);

            if material != medium && entry_axis.is_some() {
                if material != AIR {
                    // Entra a un bloque distinto: intersección exacta con su cubo.
                    let cube = Cube::from_cell(cell[0], cell[1], cell[2], material);
                    if let Some(hit) = cube.ray_intersect(origin, direction) {
                        return Some(hit);
                    }
                } else if let Some(axis) = entry_axis {
                    // Sale del agua o el vidrio hacia el aire: la superficie es
                    // la pared que acaba de cruzar. Su normal apunta hacia el
                    // aire, en el mismo sentido en que avanza el rayo.
                    let mut normal = Vec3::zeros();
                    normal[axis] = step[axis] as f32;
                    let point = origin + direction * t;
                    let mut previous = cell;
                    previous[axis] -= step[axis];
                    let min = Vec3::new(previous[0] as f32, previous[1] as f32, previous[2] as f32);
                    let (u, v) = face_uv(&point, &min, &(min + Vec3::new(1.0, 1.0, 1.0)), axis);
                    return Some(Intersect {
                        point,
                        normal,
                        distance: t,
                        u,
                        v,
                        material: medium,
                    });
                }
            }

            // Avanza a la celda vecina cuya pared esté más cerca.
            let axis = if t_max[0] < t_max[1] {
                if t_max[0] < t_max[2] { 0 } else { 2 }
            } else if t_max[1] < t_max[2] {
                1
            } else {
                2
            };

            if t_max[axis] > bounds_hit.t_far {
                return None;
            }

            t = t_max[axis];
            t_max[axis] += t_delta[axis];
            cell[axis] += step[axis];
            entry_axis = Some(axis);

            if !self.inside_grid(cell[0], cell[1], cell[2]) {
                return None;
            }
        }
    }

    // Cuánta luz del sol llega a un punto: 1.0 a pleno sol, 0.0 en sombra.
    // Se lanza un rayo hacia el sol; un bloque opaco en el camino la tapa por
    // completo, y el agua o el vidrio dejan pasar una parte.
    pub fn sun_visibility(&self, point: &Vec3) -> f32 {
        let direction = self.light.direction;
        let mut origin = *point;
        let mut transmission = 1.0;

        for _ in 0..8 {
            let Some(hit) = self.trace(&origin, &direction) else {
                return transmission;
            };
            let material = &self.materials[hit.material as usize];
            if material.transparency <= 0.0 {
                return 0.0;
            }
            // Se cobra la transparencia una vez por cuerpo: al salir de él.
            if dot(&direction, &hit.normal) > 0.0 {
                transmission *= material.transparency;
            }
            origin = hit.point + direction * 1e-3;
        }

        transmission
    }

    // Oclusión ambiental: las esquinas donde se juntan bloques reciben menos
    // luz del cielo, porque los bloques vecinos la tapan. Para la cara
    // golpeada se revisan las celdas que la rodean y se oscurece cada una de
    // sus cuatro esquinas según cuántos vecinos ocupados tenga; dentro de la
    // cara se mezcla suavemente entre las cuatro esquinas.
    pub fn ambient_occlusion(&self, hit: &Intersect) -> f32 {
        let n = hit.normal;
        let axis = if n.x.abs() > 0.5 { 0 } else if n.y.abs() > 0.5 { 1 } else { 2 };
        let (a1, a2) = match axis {
            0 => (2, 1),
            1 => (0, 2),
            _ => (0, 1),
        };

        // Celda de aire que está justo frente a la cara.
        let mut layer = [0i32; 3];
        for i in 0..3 {
            layer[i] = (hit.point[i] + n[i] * 0.5).floor() as i32;
        }

        let occupied = |du: i32, dv: i32| -> f32 {
            let mut c = layer;
            c[a1] += du;
            c[a2] += dv;
            if self.is_opaque(c[0], c[1], c[2]) { 1.0 } else { 0.0 }
        };

        let corner = |su: i32, sv: i32| -> f32 {
            let side1 = occupied(su, 0);
            let side2 = occupied(0, sv);
            if side1 > 0.0 && side2 > 0.0 {
                return 0.0;
            }
            (3.0 - side1 - side2 - occupied(su, sv)) / 3.0
        };

        let fu = hit.point[a1] - hit.point[a1].floor();
        let fv = hit.point[a2] - hit.point[a2].floor();

        let bottom = corner(-1, -1) * (1.0 - fu) + corner(1, -1) * fu;
        let top = corner(-1, 1) * (1.0 - fu) + corner(1, 1) * fu;
        bottom * (1.0 - fv) + top * fv
    }
}
