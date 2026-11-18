use crate::color::Color;
use crate::texture::Texture;
use nalgebra_glm::Vec3;

// Un skybox es un cubo gigante que rodea toda la escena, con una imagen del
// cielo pegada en cada una de sus seis caras. Cuando un rayo no choca con
// ningún bloque, se mira hacia qué cara del cubo apunta y qué punto de esa
// imagen le corresponde. Como el cubo está "infinitamente lejos", solo
// importa la dirección del rayo, no su origen.
pub struct Skybox {
    // Orden: +x, -x, +y, -y, +z, -z
    faces: [Texture; 6],
}

impl Skybox {
    pub fn load(folder: &str) -> Result<Skybox, String> {
        let load = |name: &str| Texture::load_bmp(&format!("{}/{}.bmp", folder, name));
        Ok(Skybox {
            faces: [
                load("px")?,
                load("nx")?,
                load("py")?,
                load("ny")?,
                load("pz")?,
                load("nz")?,
            ],
        })
    }

    pub fn sample(&self, direction: &Vec3) -> Color {
        let (x, y, z) = (direction.x, direction.y, direction.z);
        let (ax, ay, az) = (x.abs(), y.abs(), z.abs());

        // La componente más grande decide la cara. Dividir las otras dos
        // entre ella proyecta la dirección sobre esa cara, en el rango -1..1.
        let (face, s, t) = if ax >= ay && ax >= az {
            if x > 0.0 {
                (0, z / ax, y / ax)
            } else {
                (1, -z / ax, y / ax)
            }
        } else if ay >= az {
            if y > 0.0 {
                (2, x / ay, -z / ay)
            } else {
                (3, x / ay, z / ay)
            }
        } else if z > 0.0 {
            (4, -x / az, y / az)
        } else {
            (5, x / az, y / az)
        };

        // De -1..1 a 0..1, que es lo que espera la textura.
        self.faces[face].sample_bilinear((s + 1.0) * 0.5, (t + 1.0) * 0.5)
    }
}
