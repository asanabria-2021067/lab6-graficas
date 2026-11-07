//! Albiceleste: balón con rayas celeste y blanco, el Sol de Mayo de la bandera argentina,
//! tres estrellas y el número 10. Todo se dibuja con distancias en un plano tangente.
use super::noise_helpers::fbm;
use super::{mix, rgb, smoothstep, surface};
use crate::{fragment::Fragment, uniforms::Uniforms};
use raylib::prelude::*;
use std::f32::consts::PI;

/// Coordenadas 2D (u, v) de `p` en el plano tangente a la dirección `c`.
/// Sirve para dibujar figuras planas (sol, estrellas, número) sobre la esfera.
fn local_uv(p: Vector3, c: Vector3) -> (f32, f32, f32) {
    let up = Vector3::new(0.0, 1.0, 0.0);
    let t = up.cross(c).normalized();
    let b = c.cross(t);
    (p.dot(t), p.dot(b), p.dot(c))
}

/// Estrella de 5 puntas: el radio permitido cambia con el ángulo.
fn star_mask(u: f32, v: f32, size: f32) -> f32 {
    let r = (u * u + v * v).sqrt();
    let a = v.atan2(u) - PI / 2.0;
    let shape = size * (0.55 + 0.45 * (5.0 * a).cos().max(-0.2));
    smoothstep(shape + 0.006, shape - 0.006, r)
}

pub fn ball(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    let s = surface(fragment, u);
    let (p, n, l, v) = (s.p, s.n, s.l, s.v);
    let nz = &u.noise;
    let pr = &u.params;

    let white = rgb(0.95, 0.96, 0.97);
    let celeste = rgb(0.42, 0.70, 0.92);
    let gold = rgb(0.98, 0.72, 0.14);
    let navy = rgb(0.08, 0.13, 0.32);
    let seam_col = rgb(0.55, 0.58, 0.62);

    // ---------- 1. Rayas por longitud ----------
    let lon = p.x.atan2(p.z);
    // fase elegida para que al frente (lon = 0) quede el centro de una raya blanca
    let sv = -(lon * pr.ball_stripes * 0.5).cos();
    let stripe = smoothstep(-0.04, 0.04, sv);
    let cap = smoothstep(0.86, 0.88, p.y.abs()); // tapas blancas en los polos
    let mut albedo = mix(white, celeste, stripe * (1.0 - cap));

    // Costuras: bordes de las rayas y el anillo de las tapas.
    let seam_lines = 1.0 - smoothstep(0.0, 0.035, sv.abs() * (1.0 - p.y.abs()).max(0.05).recip() * 0.25);
    let seam_ring = 1.0 - smoothstep(0.0, 0.008, (p.y.abs() - 0.87).abs());
    let seam = (seam_lines * (1.0 - cap)).max(seam_ring);
    albedo = mix(albedo, seam_col, seam * 0.8);

    // ---------- 2. Sol de Mayo y tres estrellas (de frente) ----------
    let mut metal = 0.0; // dónde hay "foil" dorado (reacciona distinto a la luz)
    let (su, svv, sd) = local_uv(p, Vector3::new(0.0, 0.08, 1.0).normalized());
    if sd > 0.0 {
        let r = (su * su + svv * svv).sqrt();
        let a = svv.atan2(su);
        let badge = smoothstep(0.31, 0.30, r);
        let ring = smoothstep(0.31, 0.30, r) * smoothstep(0.285, 0.295, r);
        let disk = smoothstep(0.125, 0.115, r);
        // 32 rayos: rectos y ondulados intercalados
        let straight = smoothstep(0.88, 0.95, (a * 16.0).cos()) * smoothstep(0.27, 0.25, r);
        let wavy_a = a + (r * 70.0).sin() * 0.03;
        let wavy = smoothstep(0.90, 0.96, (wavy_a * 16.0 + PI).cos()) * smoothstep(0.24, 0.22, r);
        let sun = disk.max(straight).max(wavy) * smoothstep(0.1, 0.12, r).max(disk);
        albedo = mix(albedo, white, badge);
        albedo = mix(albedo, celeste, ring);
        albedo = mix(albedo, gold, sun);
        metal += sun;
        for (x, y) in [(-0.12, 0.38), (0.0, 0.41), (0.12, 0.38)] {
            let st = star_mask(su - x, svv - y, 0.055);
            albedo = mix(albedo, gold, st);
            metal += st;
        }
    }

    // ---------- 3. Número 10 (atrás) ----------
    let (nu, nv, nd) = local_uv(p, Vector3::new(0.0, 0.0, -1.0));
    if nd > 0.0 {
        let plate = smoothstep(0.30, 0.29, (nu * nu * 0.6 + nv * nv).sqrt());
        let one = smoothstep(0.04, 0.03, (nu + 0.12).abs()) * smoothstep(0.19, 0.18, nv.abs());
        let e = (((nu - 0.08) / 0.11).powi(2) + (nv / 0.19).powi(2)).sqrt();
        let zero = smoothstep(1.0, 0.95, e) * smoothstep(0.55, 0.6, e);
        albedo = mix(albedo, white, plate);
        albedo = mix(albedo, navy, one.max(zero));
    }

    // ---------- 4. Textura del material: granito fino como bump ----------
    let grain = |q: Vector3| fbm(nz, q * 70.0, 2);
    let up = if n.y.abs() < 0.99 { Vector3::new(0.0, 1.0, 0.0) } else { Vector3::new(1.0, 0.0, 0.0) };
    let t = up.cross(n).normalized();
    let b = n.cross(t);
    let e = 0.002;
    let g0 = grain(p);
    let k = 0.0035 * (1.0 - metal.min(1.0)) - seam * 0.01; // costuras hundidas
    let bn = (n - t * ((grain(p + t * e) - g0) / e * k) - b * ((grain(p + b * e) - g0) / e * k)).normalized();

    // ---------- 5. Luz: sintético con brillo suave; el dorado brilla como metal ----------
    let diffuse = bn.dot(l).max(0.0);
    let half = (l + v).normalized();
    let spec_soft = bn.dot(half).max(0.0).powf(40.0) * 0.35;
    let spec_gold = n.dot(half).max(0.0).powf(18.0) * 0.9;
    let m = metal.min(1.0);
    let base = albedo * (0.08 + diffuse * 0.95) * (1.0 - m * 0.4);
    base + rgb(1.0, 1.0, 1.0) * (spec_soft * (1.0 - m)) + gold * (spec_gold * m)
}
