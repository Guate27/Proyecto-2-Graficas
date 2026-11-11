mod framebuffer;

use minifb::{Key, Window, WindowOptions};
use std::time::Duration;

use crate::framebuffer::Framebuffer;

// Tamaño de la ventana. Es moderado a propósito: en raytracing cada píxel
// cuesta caro en términos de rendimiento, así que menos píxeles = más fluidez.
const WIDTH: usize = 800;
const HEIGHT: usize = 600;

// Azul cielo para el fondo de la escena.
const SKY_COLOR: u32 = 0x87CEEB;

fn main() {
    // ~60 cuadros por segundo: esperar 16 ms entre cuadro y cuadro.
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(SKY_COLOR);

    let mut window = Window::new(
        "Proyecto 2 - Diorama con Raytracing",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    // Se pinta el fondo una sola vez; la imagen no cambia entre cuadros.
    framebuffer.clear();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
