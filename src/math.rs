//! Matemática básica: matrices 4x4 y rotaciones.
use raylib::prelude::*;

pub type Mat4 = [[f32; 4]; 4];

pub fn identity() -> Mat4 {
    let mut m = [[0.0; 4]; 4];
    for (i, row) in m.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    m
}

pub fn mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut r = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            r[i][j] = (0..4).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    r
}

/// Multiplica la matriz por (v, w) y devuelve el vector homogéneo.
pub fn transform(m: &Mat4, v: Vector3, w: f32) -> [f32; 4] {
    let p = [v.x, v.y, v.z, w];
    let mut r = [0.0; 4];
    for (i, out) in r.iter_mut().enumerate() {
        *out = (0..4).map(|k| m[i][k] * p[k]).sum();
    }
    r
}

pub fn translation(t: Vector3) -> Mat4 {
    let mut m = identity();
    m[0][3] = t.x;
    m[1][3] = t.y;
    m[2][3] = t.z;
    m
}

pub fn scale(s: f32) -> Mat4 {
    let mut m = identity();
    m[0][0] = s;
    m[1][1] = s;
    m[2][2] = s;
    m
}

pub fn rotation_y(a: f32) -> Mat4 {
    let (s, c) = a.sin_cos();
    [
        [c, 0.0, s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn rotation_z(a: f32) -> Mat4 {
    let (s, c) = a.sin_cos();
    [
        [c, -s, 0.0, 0.0],
        [s, c, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Modelo = Traslación * Inclinación(Z) * Giro(Y) * Escala
pub fn model_matrix(t: Vector3, s: f32, tilt: f32, spin: f32) -> Mat4 {
    mul(
        &translation(t),
        &mul(&rotation_z(tilt), &mul(&rotation_y(spin), &scale(s))),
    )
}

pub fn look_at(eye: Vector3, target: Vector3, up: Vector3) -> Mat4 {
    let f = (target - eye).normalized();
    let r = f.cross(up).normalized();
    let u = r.cross(f);
    [
        [r.x, r.y, r.z, -r.dot(eye)],
        [u.x, u.y, u.z, -u.dot(eye)],
        [-f.x, -f.y, -f.z, f.dot(eye)],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fovy / 2.0).tan();
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (far + near) / (near - far), 2.0 * far * near / (near - far)],
        [0.0, 0.0, -1.0, 0.0],
    ]
}

pub fn rotate_y(v: Vector3, a: f32) -> Vector3 {
    let (s, c) = a.sin_cos();
    Vector3::new(c * v.x + s * v.z, v.y, -s * v.x + c * v.z)
}

pub fn rotate_z(v: Vector3, a: f32) -> Vector3 {
    let (s, c) = a.sin_cos();
    Vector3::new(c * v.x - s * v.y, s * v.x + c * v.y, v.z)
}

/// Rotación de Rodrigues: gira `v` alrededor de `axis` (unitario) un ángulo `a`.
pub fn rotate_axis(v: Vector3, axis: Vector3, a: f32) -> Vector3 {
    let (s, c) = a.sin_cos();
    v * c + axis.cross(v) * s + axis * (axis.dot(v) * (1.0 - c))
}
