use crate::color::Color;
use std::fs;

// Imagen cargada en memoria como una lista de colores, fila por fila,
// empezando por la fila de arriba.
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pixels: Vec<Color>,
}

// Lee números del archivo en formato little-endian (el byte menos
// significativo va primero), que es como los guarda el formato BMP.
fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    read_u32(bytes, offset) as i32
}

impl Texture {
    // Lector de BMP sin compresión de 24 o 32 bits por píxel.
    //
    // Estructura del archivo:
    //   byte 0-1   "BM"
    //   byte 10    dónde empiezan los píxeles
    //   byte 18    ancho
    //   byte 22    alto (positivo = filas de abajo hacia arriba)
    //   byte 28    bits por píxel
    //   byte 30    compresión (0 = ninguna)
    // Cada píxel se guarda como azul, verde, rojo, y cada fila se rellena
    // con ceros hasta medir un múltiplo de 4 bytes.
    pub fn load_bmp(path: &str) -> Result<Texture, String> {
        let bytes = fs::read(path).map_err(|e| format!("No se pudo leer {}: {}", path, e))?;

        if bytes.len() < 54 || &bytes[0..2] != b"BM" {
            return Err(format!("{} no es un archivo BMP", path));
        }

        let data_offset = read_u32(&bytes, 10) as usize;
        let width = read_i32(&bytes, 18);
        let raw_height = read_i32(&bytes, 22);
        let bits_per_pixel = read_u16(&bytes, 28);
        let compression = read_u32(&bytes, 30);

        if compression != 0 || (bits_per_pixel != 24 && bits_per_pixel != 32) {
            return Err(format!("{}: solo se aceptan BMP de 24 o 32 bits sin compresión", path));
        }

        let width = width as usize;
        let height = raw_height.unsigned_abs() as usize;
        let bottom_up = raw_height > 0;
        let bytes_per_pixel = (bits_per_pixel / 8) as usize;
        let row_size = (width * bytes_per_pixel + 3) / 4 * 4;

        let mut pixels = Vec::with_capacity(width * height);

        for row in 0..height {
            // Si el archivo va de abajo hacia arriba, la primera fila de la
            // imagen es la última del archivo.
            let file_row = if bottom_up { height - 1 - row } else { row };
            let start = data_offset + file_row * row_size;

            for x in 0..width {
                let i = start + x * bytes_per_pixel;
                pixels.push(Color::from_rgb(bytes[i + 2], bytes[i + 1], bytes[i]));
            }
        }

        Ok(Texture { width, height, pixels })
    }

    fn get(&self, x: usize, y: usize) -> Color {
        self.pixels[y * self.width + x]
    }

    // Muestreo "al vecino más cercano": toma el píxel exacto que cae en (u, v),
    // sin suavizar. Es lo que da el aspecto de pixel art de Minecraft.
    // u va de 0 (izquierda) a 1 (derecha) y v de 0 (abajo) a 1 (arriba).
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let u = u.rem_euclid(1.0);
        let v = v.rem_euclid(1.0);
        let x = ((u * self.width as f32) as usize).min(self.width - 1);
        let y = (((1.0 - v) * self.height as f32) as usize).min(self.height - 1);
        self.get(x, y)
    }

    // Muestreo bilineal: mezcla los cuatro píxeles más cercanos según qué tan
    // cerca esté el punto de cada uno. Se usa en el cielo, donde los bordes
    // de píxel se notarían demasiado.
    pub fn sample_bilinear(&self, u: f32, v: f32) -> Color {
        let fx = (u.clamp(0.0, 1.0) * self.width as f32 - 0.5).max(0.0);
        let fy = ((1.0 - v.clamp(0.0, 1.0)) * self.height as f32 - 0.5).max(0.0);

        let x0 = (fx as usize).min(self.width - 1);
        let y0 = (fy as usize).min(self.height - 1);
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);
        let tx = fx - x0 as f32;
        let ty = fy - y0 as f32;

        let top = self.get(x0, y0).lerp(&self.get(x1, y0), tx);
        let bottom = self.get(x0, y1).lerp(&self.get(x1, y1), tx);
        top.lerp(&bottom, ty)
    }
}
