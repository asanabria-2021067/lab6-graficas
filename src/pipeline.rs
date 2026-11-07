//! Vertex shader y función render (rasterizado + depth test + fragment shader).
use crate::framebuffer::Framebuffer;
use crate::math::{self, Mat4};
use crate::uniforms::Uniforms;
use crate::vertex::{TransformedVertex, Vertex};
use crate::{fragment::Fragment, shaders, triangle};
use raylib::prelude::*;
use rayon::prelude::*;

pub fn vertex_shader(v: &Vertex, u: &Uniforms, mvp: &Mat4) -> TransformedVertex {
    let clip = math::transform(mvp, v.position, 1.0);
    let w = clip[3];
    let inv_w = if w.abs() > 1e-6 { 1.0 / w } else { 0.0 };
    let ndc = Vector3::new(clip[0] * inv_w, clip[1] * inv_w, clip[2] * inv_w);
    let screen = Vector3::new(
        (ndc.x + 1.0) * 0.5 * u.width as f32,
        (1.0 - ndc.y) * 0.5 * u.height as f32,
        ndc.z,
    );
    let wp = math::transform(&u.model, v.position, 1.0);
    TransformedVertex {
        screen,
        inv_w,
        world_position: Vector3::new(wp[0], wp[1], wp[2]),
        world_normal: u.object_dir_to_world(v.normal).normalized(),
        object_position: v.position,
    }
}

/// Igual que el render de clase, pero en dos pasos:
/// 1) se rasterizan los triángulos y el depth test guarda el fragmento más cercano de cada píxel;
/// 2) el fragment shader se corre una sola vez por píxel visible, en paralelo (rayon).
/// Así los shaders pesados (mucho ruido) no se calculan para fragmentos que luego se tapan.
pub fn render(fb: &mut Framebuffer, vertices: &[Vertex], u: &Uniforms) {
    fb.clear();
    let mvp = math::mul(&u.projection, &math::mul(&u.view, &u.model));
    let transformed: Vec<_> = vertices.iter().map(|v| vertex_shader(v, u, &mvp)).collect();

    let mut visible: Vec<Option<Fragment>> = vec![None; fb.width * fb.height];
    for tri in transformed.chunks_exact(3) {
        for fragment in triangle::triangle(tri, u) {
            let index = fragment.position.y as usize * fb.width + fragment.position.x as usize;
            if fragment.depth < fb.depths[index] {
                fb.depths[index] = fragment.depth;
                visible[index] = Some(fragment);
            }
        }
    }

    fb.colors
        .par_iter_mut()
        .zip(visible.par_iter())
        .for_each(|(color, frag)| {
            if let Some(f) = frag {
                *color = shaders::fragment_shader(f, u);
            }
        });
}
