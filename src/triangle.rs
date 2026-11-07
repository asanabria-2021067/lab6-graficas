//! Rasterizado con coordenadas baricéntricas e interpolación con perspectiva correcta.
use crate::fragment::Fragment;
use crate::uniforms::Uniforms;
use crate::vertex::TransformedVertex;
use raylib::prelude::*;

fn edge(a: Vector3, b: Vector3, px: f32, py: f32) -> f32 {
    (px - a.x) * (b.y - a.y) - (py - a.y) * (b.x - a.x)
}

pub fn triangle(v: &[TransformedVertex], u: &Uniforms) -> Vec<Fragment> {
    let (a, b, c) = (&v[0], &v[1], &v[2]);
    // Triángulo detrás de la cámara: lo descartamos.
    if a.inv_w <= 0.0 || b.inv_w <= 0.0 || c.inv_w <= 0.0 {
        return Vec::new();
    }
    let (sa, sb, sc) = (a.screen, b.screen, c.screen);
    let area = edge(sa, sb, sc.x, sc.y);
    if area.abs() < 1e-8 {
        return Vec::new();
    }

    let min_x = sa.x.min(sb.x).min(sc.x).floor().max(0.0) as i32;
    let max_x = sa.x.max(sb.x).max(sc.x).ceil().min(u.width as f32 - 1.0) as i32;
    let min_y = sa.y.min(sb.y).min(sc.y).floor().max(0.0) as i32;
    let max_y = sa.y.max(sb.y).max(sc.y).ceil().min(u.height as f32 - 1.0) as i32;

    let mut out = Vec::new();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let w0 = edge(sb, sc, px, py) / area;
            let w1 = edge(sc, sa, px, py) / area;
            let w2 = edge(sa, sb, px, py) / area;
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }
            let depth = w0 * sa.z + w1 * sb.z + w2 * sc.z;

            // Pesos con perspectiva correcta.
            let (p0, p1, p2) = (w0 * a.inv_w, w1 * b.inv_w, w2 * c.inv_w);
            let s = 1.0 / (p0 + p1 + p2);
            let lerp = |va: Vector3, vb: Vector3, vc: Vector3| (va * p0 + vb * p1 + vc * p2) * s;

            let normal = lerp(a.world_normal, b.world_normal, c.world_normal).normalized();
            out.push(Fragment {
                position: Vector2::new(x as f32, y as f32),
                depth,
                normal,
                world_position: lerp(a.world_position, b.world_position, c.world_position),
                object_position: lerp(a.object_position, b.object_position, c.object_position),
                intensity: normal.dot(u.light_dir).max(0.0),
            });
        }
    }
    out
}
