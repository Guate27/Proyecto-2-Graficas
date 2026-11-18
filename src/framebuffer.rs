use std::fs;

// El framebuffer es la "hoja de papel" donde se pinta la imagen antes de
// mostrarla en la ventana. Cada píxel es un número u32 con el color 0xRRGGBB.
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],
        }
    }

    // Guarda la imagen como BMP de 24 bits (el mismo formato que se lee para
    // las texturas): filas de abajo hacia arriba, cada píxel en orden azul,
    // verde, rojo, y cada fila rellena hasta un múltiplo de 4 bytes.
    pub fn save_bmp(&self, path: &str) -> std::io::Result<()> {
        let row_size = (self.width * 3 + 3) / 4 * 4;
        let data_size = row_size * self.height;
        let file_size = 54 + data_size;

        let mut bytes = Vec::with_capacity(file_size);
        bytes.extend_from_slice(b"BM");
        bytes.extend_from_slice(&(file_size as u32).to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&54u32.to_le_bytes());
        bytes.extend_from_slice(&40u32.to_le_bytes());
        bytes.extend_from_slice(&(self.width as i32).to_le_bytes());
        bytes.extend_from_slice(&(self.height as i32).to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&24u16.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&(data_size as u32).to_le_bytes());
        bytes.extend_from_slice(&[0u8; 16]);

        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let pixel = self.buffer[y * self.width + x];
                bytes.push(pixel as u8);
                bytes.push((pixel >> 8) as u8);
                bytes.push((pixel >> 16) as u8);
            }
            bytes.resize(bytes.len() + row_size - self.width * 3, 0);
        }

        fs::write(path, bytes)
    }
}
