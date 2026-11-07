//! Orvane: gigante gaseoso con bandas que fluyen como líquido y una gran tormenta
//! que tuerce las bandas a su alrededor.
use super::noise_helpers::{fbm, fbm4};
use super::{mix, ramp, rgb, smoothstep, surface};
use crate::math::{rotate_axis, rotate_y};
use crate::{fragment::Fragment, uniforms::Uniforms};
use raylib::prelude::*;
use std::f32::consts::PI;

pub fn gas_giant(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    let s = surface(fragment, u);
    let (p, n, l, v) = (s.p, s.n, s.l, s.v);
    let nz = &u.noise;
    let pr = &u.params;
    let t = u.time;

    // ---------- 1. Tormenta: torsión localizada ----------
    // Centro de la tormenta en latitud -22°. Cada punto cercano se gira alrededor del eje
    // que pasa por el centro; el giro es mayor cerca del centro y 0 en el borde.
    // Como las bandas se calculan DESPUÉS con el punto girado, las bandas se curvan.
    let (lat0, lon0) = (-0.38f32, 0.6f32);
    let c = Vector3::new(lat0.cos() * lon0.sin(), lat0.sin(), lat0.cos() * lon0.cos());
    let d = p.dot(c).clamp(-1.0, 1.0).acos(); // distancia angular al centro
    let falloff = (1.0 - d / pr.storm_radius).clamp(0.0, 1.0);
    let twist = pr.storm_strength * falloff * falloff;
    let ps = rotate_axis(p, c, twist);

    // ---------- 2. Flujo de las bandas (flow map) ----------
    // Cada franja de latitud se arrastra hacia un lado distinto (vientos alternos).
    // Si giráramos el ruido para siempre, las bandas se estirarían hasta volverse rayitas,
    // así que usamos la técnica de "flow map": dos copias del patrón avanzan desfasadas
    // medio ciclo, cada una se reinicia al terminar su ciclo, y se mezclan para ocultar el salto.
    let jet = (ps.y * pr.band_count * 0.5 * PI).sin();
    let cycle = t * pr.flow_speed;
    let phase0 = cycle.fract();
    let phase1 = (cycle + 0.5).fract();
    let (val0, r0) = bands(nz, ps, jet * (phase0 - 0.5) * 0.6, t, pr);
    let (val1, r1) = bands(nz, ps, jet * (phase1 - 0.5) * 0.6, t, pr);
    let blend = ((phase0 - 0.5) * 2.0).abs(); // 1 cuando la copia 0 se reinicia
    let val = val0 + (val1 - val0) * blend;
    let r = r0 + (r1 - r0) * blend;

    let mut color = ramp(
        &[
            (0.0, rgb(0.32, 0.18, 0.24)),
            (0.25, rgb(0.62, 0.30, 0.22)),
            (0.45, rgb(0.84, 0.60, 0.40)),
            (0.65, rgb(0.95, 0.87, 0.70)),
            (0.82, rgb(0.62, 0.55, 0.68)),
            (1.0, rgb(0.92, 0.88, 0.82)),
        ],
        val,
    );

    // ---------- 3. Ojo y collar de la tormenta ----------
    let core = smoothstep(pr.storm_radius * 0.6, pr.storm_radius * 0.2, d);
    let collar = smoothstep(pr.storm_radius * 0.75, pr.storm_radius * 0.58, d)
        * (1.0 - smoothstep(pr.storm_radius * 0.58, pr.storm_radius * 0.45, d));
    let storm_col = mix(rgb(0.55, 0.12, 0.06), rgb(0.95, 0.45, 0.18), r * 0.5 + 0.5);
    color = mix(color, rgb(0.97, 0.90, 0.78), collar * 0.55 * (pr.storm_strength / 5.0).min(1.0));
    color = mix(color, storm_col, core * (pr.storm_strength / 5.0).min(1.0));

    // Polos más oscuros y azulados (neblina).
    color = mix(color, rgb(0.28, 0.30, 0.42), smoothstep(0.72, 0.95, p.y.abs()) * 0.7);

    // ---------- 4. Luz: difusa "envuelta" (suave, como una atmósfera gruesa) ----------
    let wrap = ((n.dot(l) + 0.2) / 1.2).max(0.0);
    let rim = (1.0 - n.dot(v).max(0.0)).powf(2.5);
    color * (0.02 + wrap * 1.05) + rgb(0.9, 0.7, 0.5) * (rim * wrap * 0.25)
}

/// Patrón de bandas para una posición ya torcida por la tormenta.
/// `shift` es cuánto se arrastra el ruido sobre el eje Y. Devuelve (valor de banda, ruido r).
fn bands(nz: &noise::OpenSimplex, ps: Vector3, shift: f32, t: f32, pr: &crate::uniforms::Params) -> (f32, f32) {
    let pf = rotate_y(ps, shift);
    // Domain warping en dos pasos: q deforma a r, r deforma la latitud.
    let q = Vector3::new(
        fbm4(nz, pf * 1.2 + Vector3::new(1.0, 0.0, 4.0), t * 0.04, 3),
        fbm4(nz, pf * 1.2 + Vector3::new(6.0, 2.0, 0.0), t * 0.04, 3),
        0.0,
    );
    let r = fbm4(nz, pf * 2.0 + q * 2.6, t * 0.07, 3);
    let y = ps.y + r * pr.turbulence + q.x * pr.turbulence * 0.5;

    // Senos sobre la latitud ya deformada + vetas finas estiradas horizontalmente.
    let b1 = (y * pr.band_count * PI).sin() * 0.5 + 0.5;
    let b2 = (y * pr.band_count * 2.3 * PI + q.y * 3.0).sin() * 0.5 + 0.5;
    let streaks = fbm(nz, Vector3::new(pf.x * 3.0, pf.y * 38.0, pf.z * 3.0) + q, 3) * 0.5 + 0.5;
    (b1 * 0.62 + b2 * 0.25 + streaks * 0.13, r)
}
