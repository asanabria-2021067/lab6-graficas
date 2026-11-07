use raylib::prelude::*;

/// Lo que recibe cada fragment shader.
/// (world_position e intensity vienen del renderer de clase; no todos los shaders los usan)
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Fragment {
    pub position: Vector2,        // píxel en pantalla
    pub depth: f32,
    pub normal: Vector3,          // normal interpolada en el mundo
    pub world_position: Vector3,
    pub object_position: Vector3, // posición en la esfera original (se usa para los patrones)
    pub intensity: f32,           // iluminación difusa básica del renderer: max(n·l, 0)
}
