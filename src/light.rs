use crate::color::Color;
use nalgebra_glm::Vec3;

// Luz direccional: el sol está tan lejos que todos sus rayos llegan
// paralelos, así que basta con una dirección (no hace falta una posición).
pub struct Light {
    // Dirección que va desde cualquier punto de la escena HACIA el sol.
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl Light {
    pub fn new(direction: Vec3, color: Color, intensity: f32) -> Self {
        Light {
            direction: direction.normalize(),
            color,
            intensity,
        }
    }
}
