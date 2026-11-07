use crate::math::{self, Mat4};
use noise::OpenSimplex;
use raylib::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Body {
    Star,
    Rocky,
    GasGiant,
    Ball,
    Orb,
}

impl Body {
    pub fn name(&self) -> &'static str {
        match self {
            Body::Star => "Solmira (estrella)",
            Body::Rocky => "Tessara (planeta rocoso)",
            Body::GasGiant => "Orvane (gigante gaseoso)",
            Body::Ball => "Albiceleste (balon, extra)",
            Body::Orb => "Lumen (orbe de cristal, extra)",
        }
    }
}

/// Parámetros que se pueden cambiar en vivo con el teclado.
#[derive(Clone, Copy, Debug)]
pub struct Params {
    // Planeta rocoso
    pub sea_level: f32,      // umbral del ruido que separa océano y tierra
    pub cloud_coverage: f32, // 0 = cielo despejado, 1 = todo nublado
    pub cloud_height: f32,   // altura de la capa de nubes (afecta qué tan lejos cae la sombra)
    pub bump_strength: f32,  // fuerza del relieve
    pub bump: bool,
    pub clouds: bool,
    pub cloud_shadows: bool,
    pub specular: bool,
    pub city_lights: bool,
    // Estrella
    pub star_activity: f32,  // cantidad de manchas solares
    pub star_speed: f32,     // qué tan rápido hierve el plasma
    // Gigante gaseoso
    pub storm_strength: f32, // cuánto tuerce la tormenta las bandas
    pub storm_radius: f32,
    pub band_count: f32,
    pub turbulence: f32,     // cuánto se deforman las bandas
    pub flow_speed: f32,     // ciclos por segundo del flujo de las bandas
    // Balón
    pub ball_stripes: f32,   // cantidad de rayas (celestes + blancas)
    // Orbe
    pub orb_ior: f32,        // índice de refracción del vidrio (1 = sin refracción)
    pub core_radius: f32,    // radio base del núcleo
    pub core_relief: f32,    // cuánto levanta el campo de altura al núcleo
}

impl Default for Params {
    fn default() -> Self {
        Self {
            sea_level: 0.04,
            cloud_coverage: 0.48,
            cloud_height: 0.06,
            bump_strength: 0.09,
            bump: true,
            clouds: true,
            cloud_shadows: true,
            specular: true,
            city_lights: true,
            star_activity: 0.5,
            star_speed: 1.0,
            storm_strength: 5.0,
            storm_radius: 0.34,
            band_count: 9.0,
            turbulence: 0.14,
            flow_speed: 0.08,
            ball_stripes: 10.0,
            orb_ior: 1.45,
            core_radius: 0.28,
            core_relief: 0.18,
        }
    }
}

pub struct Uniforms {
    pub model: Mat4,
    pub view: Mat4,
    pub projection: Mat4,
    pub width: usize,
    pub height: usize,
    pub time: f32,
    pub light_dir: Vector3,  // dirección HACIA la luz, en el mundo
    pub camera_pos: Vector3,
    pub translation: Vector3,
    pub size: f32,           // radio de la esfera en el mundo
    pub tilt: f32,
    pub spin: f32,
    pub noise: OpenSimplex,
    pub body: Body,
    pub params: Params,
}

impl Uniforms {
    /// objeto -> mundo: primero el giro (Y), luego la inclinación (Z)
    pub fn object_dir_to_world(&self, d: Vector3) -> Vector3 {
        math::rotate_z(math::rotate_y(d, self.spin), self.tilt)
    }
    /// mundo -> objeto (la inversa)
    pub fn world_dir_to_object(&self, d: Vector3) -> Vector3 {
        math::rotate_y(math::rotate_z(d, -self.tilt), -self.spin)
    }
    /// Dirección de la luz vista desde la esfera (espacio del objeto).
    pub fn light_in_object(&self) -> Vector3 {
        self.world_dir_to_object(self.light_dir).normalized()
    }
    /// Cámara en el espacio del objeto, medida en radios de la esfera.
    pub fn camera_in_object(&self) -> Vector3 {
        self.world_dir_to_object(self.camera_pos - self.translation) * (1.0 / self.size)
    }
}
