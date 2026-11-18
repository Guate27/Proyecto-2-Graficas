use nalgebra_glm::Vec3;

// Todo lo que se necesita saber de un choque entre un rayo y un bloque.
#[derive(Debug, Clone, Copy)]
pub struct Intersect {
    // Punto exacto del choque en el mundo.
    pub point: Vec3,
    // Vector de largo 1 perpendicular a la cara golpeada, apuntando hacia
    // afuera del bloque. Dice hacia dónde "mira" esa cara.
    pub normal: Vec3,
    // Distancia t desde el origen del rayo.
    #[allow(dead_code)]
    pub distance: f32,
    // Coordenadas de textura (0..1) dentro de la cara golpeada.
    pub u: f32,
    pub v: f32,
    // Índice del material en la lista de materiales de la escena.
    pub material: u8,
}

// Todo objeto de la escena responde a la misma pregunta: dado un rayo,
// ¿lo toca? Si lo toca, devuelve los datos del choque; si no, None.
pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect>;
}
