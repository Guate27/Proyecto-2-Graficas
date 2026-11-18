use nalgebra_glm::Vec3;

// Describe de qué está hecho un bloque: qué imagen lleva en cada cara y cómo
// responde a la luz.
#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub name: &'static str,

    // Índices dentro de la lista de texturas de la escena. Un bloque como el
    // pasto lleva una imagen arriba, otra a los lados y otra abajo.
    pub texture_top: usize,
    pub texture_side: usize,
    pub texture_bottom: usize,

    // Qué fracción de la luz del sol devuelve como color propio (difuso).
    pub albedo: f32,
    // Qué tan fuerte es el brillo blanco que aparece cuando el reflejo del
    // sol coincide con la dirección de la cámara.
    pub specular: f32,
    // Qué tan pequeño y concentrado es ese brillo: valores altos dan un
    // punto pequeño (vidrio, agua); valores bajos, una mancha amplia.
    pub shininess: f32,
    // Fracción de la luz que atraviesa el bloque (0 = opaco).
    pub transparency: f32,
    // Fracción de la luz que rebota como en un espejo (0 = nada).
    pub reflectivity: f32,
    // Qué tanto se dobla la luz al entrar: 1.0 aire, 1.33 agua, 1.5 vidrio.
    pub refractive_index: f32,
}

impl Material {
    // Elige la textura según hacia dónde mira la cara que se golpeó.
    pub fn texture_for(&self, normal: &Vec3) -> usize {
        if normal.y > 0.5 {
            self.texture_top
        } else if normal.y < -0.5 {
            self.texture_bottom
        } else {
            self.texture_side
        }
    }
}
