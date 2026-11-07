pub mod ball;
pub mod gas_giant;
pub mod noise_helpers;
pub mod orb;
pub mod rocky;
pub mod star;

use crate::fragment::Fragment;
use crate::uniforms::{Body, Uniforms};
use raylib::prelude::*;

/// Punto de entrada: el render llama esta función por cada píxel visible
/// y aquí se escoge el shader del cuerpo activo.
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    match uniforms.body {
        Body::Star => star::star(fragment, uniforms),
        Body::Rocky => rocky::rocky(fragment, uniforms),
        Body::GasGiant => gas_giant::gas_giant(fragment, uniforms),
        Body::Ball => ball::ball(fragment, uniforms),
        Body::Orb => orb::orb(fragment, uniforms),
    }
}

pub fn mix(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    a + (b - a) * t
}

pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn rgb(r: f32, g: f32, b: f32) -> Vector3 {
    Vector3::new(r, g, b)
}

/// Degradado de colores: `stops` son pares (posición, color) ordenados.
pub fn ramp(stops: &[(f32, Vector3)], t: f32) -> Vector3 {
    if t <= stops[0].0 {
        return stops[0].1;
    }
    for w in stops.windows(2) {
        let (a, b) = (w[0], w[1]);
        if t <= b.0 {
            return mix(a.1, b.1, smoothstep(a.0, b.0, t));
        }
    }
    stops[stops.len() - 1].1
}

/// Datos comunes que casi todos los shaders necesitan, en espacio del objeto.
pub struct Surface {
    pub p: Vector3, // punto sobre la esfera unitaria (se usa para los patrones)
    pub n: Vector3, // normal geométrica
    pub l: Vector3, // dirección hacia la luz
    pub v: Vector3, // dirección hacia la cámara
}

/// La sphere.obj del Canvas tiene pocas caras (192 triángulos) y normales planas por cara.
/// Si usamos esas normales la luz se ve en facetas, así que para iluminar usamos la normal
/// "ideal" de una esfera: la posición normalizada. La silueta sigue siendo la de la malla,
/// porque los vértices no se tocan. Con `false` se usan las normales del .obj.
pub const SMOOTH_SPHERE_NORMALS: bool = true;

pub fn surface(fragment: &Fragment, u: &Uniforms) -> Surface {
    let p = fragment.object_position.normalized();
    let n = if SMOOTH_SPHERE_NORMALS {
        p
    } else {
        u.world_dir_to_object(fragment.normal).normalized()
    };
    let l = u.light_in_object();
    let v = (u.camera_in_object() - p).normalized();
    Surface { p, n, l, v }
}
