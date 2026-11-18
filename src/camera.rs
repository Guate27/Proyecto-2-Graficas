use nalgebra_glm::Vec3;
use std::f32::consts::PI;

// Límite para no pasar por encima de los polos: si la cámara mirara justo
// hacia abajo o hacia arriba, "derecha" dejaría de estar definida.
const PITCH_LIMIT: f32 = PI / 2.0 - 0.1;

pub struct Camera {
    // Dónde está el ojo.
    pub eye: Vec3,
    // Hacia qué punto mira (el centro del diorama).
    pub center: Vec3,
    // Qué dirección se considera "arriba".
    pub up: Vec3,
}

impl Camera {
    pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        Camera { eye, center, up }
    }

    // Convierte una dirección escrita "desde el punto de vista de la cámara"
    // (x a la derecha de la pantalla, y hacia arriba, -z hacia el frente) a
    // una dirección del mundo, usando los tres ejes propios de la cámara.
    pub fn basis_change(&self, vector: &Vec3) -> Vec3 {
        let forward = (self.center - self.eye).normalize();
        let right = forward.cross(&self.up).normalize();
        let up = right.cross(&forward).normalize();

        let rotated = vector.x * right + vector.y * up - vector.z * forward;

        rotated.normalize()
    }

    pub fn distance(&self) -> f32 {
        (self.eye - self.center).magnitude()
    }

    // Gira el ojo alrededor del centro como un satélite: yaw es la vuelta
    // horizontal y pitch la inclinación. El radio no cambia.
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let radius_vector = self.eye - self.center;
        let radius = radius_vector.magnitude();

        // Ángulos actuales a partir de la posición del ojo.
        let current_yaw = radius_vector.z.atan2(radius_vector.x);
        let radius_xz =
            (radius_vector.x * radius_vector.x + radius_vector.z * radius_vector.z).sqrt();
        let current_pitch = (-radius_vector.y).atan2(radius_xz);

        let new_yaw = (current_yaw + delta_yaw) % (2.0 * PI);
        let new_pitch = (current_pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        // Coordenadas esféricas de vuelta a x, y, z.
        self.eye = self.center
            + Vec3::new(
                radius * new_yaw.cos() * new_pitch.cos(),
                -radius * new_pitch.sin(),
                radius * new_yaw.sin() * new_pitch.cos(),
            );
    }

    // Acerca (delta negativo) o aleja (delta positivo) el ojo a lo largo de la
    // línea que lo une con el centro, sin pasar de los límites.
    pub fn zoom(&mut self, delta: f32, min_distance: f32, max_distance: f32) {
        let direction = (self.eye - self.center).normalize();
        let distance = (self.distance() + delta).clamp(min_distance, max_distance);
        self.eye = self.center + direction * distance;
    }
}
