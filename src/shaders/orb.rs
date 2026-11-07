//! Lumen: orbe de cristal con un núcleo rocoso y nebulosas adentro.
//! El interior se dibuja siguiendo el rayo refractado hacia dentro de la esfera
//! (ray marching), así el núcleo se desplaza respecto a la superficie cuando la cámara
//! se mueve: eso es el parallax dependiente de la vista.
use super::noise_helpers::{fbm, fbm4, ridged};
use super::{mix, ramp, rgb, smoothstep, surface};
use crate::math::rotate_y;
use crate::{fragment::Fragment, uniforms::Uniforms};
use noise::OpenSimplex;
use raylib::prelude::*;

const STEPS: usize = 24;

/// Campo de altura procedural del núcleo: 0..1. El núcleo tiene radio base `core_radius`
/// y esta altura lo levanta hasta `core_relief`.
fn core_height(nz: &OpenSimplex, q: Vector3, time: f32) -> f32 {
    let dir = rotate_y(q.normalized(), time * 0.15);
    ridged(nz, dir * 2.2, 3)
}

/// Ley de Snell: dirección del rayo que entra al vidrio.
fn refract(i: Vector3, n: Vector3, eta: f32) -> Vector3 {
    let cosi = -i.dot(n);
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
    if k < 0.0 {
        return i; // reflexión total (casi no pasa al entrar)
    }
    (i * eta + n * (eta * cosi - k.sqrt())).normalized()
}

pub fn orb(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    let s = surface(fragment, u);
    let (p, n, l, v) = (s.p, s.n, s.l, s.v);
    let nz = &u.noise;
    let pr = &u.params;
    let t = u.time;

    // ---------- 1. Rayo refractado hacia dentro ----------
    let d = refract(v * -1.0, n, 1.0 / pr.orb_ior);
    let chord = (-2.0 * p.dot(d)).max(0.0); // largo del rayo dentro de la esfera
    let dt = chord / STEPS as f32;

    // ---------- 2. Ray marching: nebulosa + núcleo ----------
    let mut acc = rgb(0.0, 0.0, 0.0);
    let mut trans = 1.0f32; // cuánta luz sigue pasando
    for i in 0..STEPS {
        let q = p + d * (dt * (i as f32 + 0.5));
        let r = q.length();

        // ¿Ya chocamos con el núcleo? Su superficie es radio base + altura del ruido.
        let h = core_height(nz, q, t);
        if r < pr.core_radius + h * pr.core_relief {
            // Afinamos el choque con bisección entre el paso anterior y este (bordes más limpios).
            let (mut a, mut b) = (dt * (i as f32 - 0.5).max(0.0), dt * (i as f32 + 0.5));
            for _ in 0..4 {
                let mid = (a + b) * 0.5;
                let qm = p + d * mid;
                if qm.length() < pr.core_radius + core_height(nz, qm, t) * pr.core_relief { b = mid; } else { a = mid; }
            }
            let q = p + d * b;
            let h = core_height(nz, q, t);
            let normal = q.normalized();
            let lit = normal.dot(l).max(0.0) * 0.7 + 0.3;
            let veins = smoothstep(0.78, 0.95, h); // crestas brillantes
            let core = ramp(
                &[(0.0, rgb(0.10, 0.04, 0.20)), (0.5, rgb(0.30, 0.10, 0.45)), (1.0, rgb(0.55, 0.30, 0.75))],
                h,
            ) * lit
                + rgb(0.25, 0.95, 0.80) * (veins * 0.8);
            acc = acc + core * trans;
            trans = 0.0;
            break;
        }

        // Nebulosa: densidad de ruido 4D, más fuerte en una capa intermedia.
        let shell = smoothstep(0.3, 0.5, r) * (1.0 - smoothstep(0.85, 1.0, r));
        let dens = smoothstep(0.1, 0.55, fbm4(nz, q * 2.6, t * 0.12, 2)) * shell * 0.26;
        let neb = mix(rgb(0.15, 0.45, 0.95), rgb(0.95, 0.35, 0.75), smoothstep(-0.3, 0.3, q.y));
        acc = acc + neb * (dens * trans);
        trans *= 1.0 - dens;
    }
    acc = acc + rgb(0.01, 0.015, 0.04) * trans; // fondo oscuro del vidrio

    // ---------- 3. Superficie del vidrio ----------
    // Grabados finos fijos en la superficie: sirven de referencia para notar el parallax.
    let etch = smoothstep(0.93, 0.98, ridged(nz, p * 3.5 + rgb(7.0, 1.0, 3.0), 2)) * 0.18;
    let ndv = n.dot(v).max(0.0);
    let fresnel = 0.04 + 0.96 * (1.0 - ndv).powf(5.0); // reflejo en los bordes
    let half = (l + v).normalized();
    let spec = n.dot(half).max(0.0).powf(220.0) * 2.0;
    let rim_tint = rgb(0.45, 0.6, 1.0);
    let rim_noise = fbm(nz, p * 4.0, 2) * 0.5 + 0.5;

    acc * (1.0 - fresnel) + rim_tint * (fresnel * (0.5 + rim_noise * 0.5)) + rgb(0.85, 0.92, 1.0) * etch + rgb(1.0, 1.0, 1.0) * spec
}
