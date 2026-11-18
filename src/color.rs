use std::ops::{Add, Mul};

// Color con canales decimales entre 0.0 y 1.0. Trabajar con decimales evita
// que la luz se "corte" a mitad de los cálculos; solo se convierte a 0..255
// al final, cuando el píxel se escribe en el framebuffer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0 };

    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b }
    }

    // A partir de valores 0..255, como vienen en las imágenes.
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }

    // Empaqueta el color como 0xRRGGBB, el formato que usa minifb. Los
    // valores mayores que 1.0 (luz muy intensa) se recortan a blanco.
    pub fn to_hex(&self) -> u32 {
        let r = (self.r.clamp(0.0, 1.0) * 255.0) as u32;
        let g = (self.g.clamp(0.0, 1.0) * 255.0) as u32;
        let b = (self.b.clamp(0.0, 1.0) * 255.0) as u32;
        (r << 16) | (g << 8) | b
    }

    // Mezcla lineal: t = 0 devuelve self, t = 1 devuelve other.
    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        *self * (1.0 - t) + *other * t
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color::new(self.r + other.r, self.g + other.g, self.b + other.b)
    }
}

// Escalar: hace el color más claro u oscuro.
impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, scalar: f32) -> Color {
        Color::new(self.r * scalar, self.g * scalar, self.b * scalar)
    }
}

// Color por color: tiñe un color con otro (por ejemplo, la textura por la
// luz del sol).
impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, other: Color) -> Color {
        Color::new(self.r * other.r, self.g * other.g, self.b * other.b)
    }
}
