//! Funciones de ruido sobre OpenSimplex.
use noise::{NoiseFn, OpenSimplex};
use raylib::prelude::*;

/// Ruido 3D, aprox. en [-1, 1].
pub fn sample(noise: &OpenSimplex, p: Vector3) -> f32 {
    noise.get([p.x as f64, p.y as f64, p.z as f64]) as f32
}

/// Ruido 4D: la cuarta coordenada es el tiempo, así el patrón "hierve" en su lugar
/// en vez de solo deslizarse.
pub fn sample4(noise: &OpenSimplex, p: Vector3, w: f32) -> f32 {
    noise.get([p.x as f64, p.y as f64, p.z as f64, w as f64]) as f32
}

/// fBm: suma de octavas, cada una con el doble de frecuencia y la mitad de amplitud.
pub fn fbm(noise: &OpenSimplex, p: Vector3, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 0.5, 1.0, 0.0);
    for _ in 0..octaves {
        sum += sample(noise, p * freq) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.03;
    }
    sum / norm
}

pub fn fbm4(noise: &OpenSimplex, p: Vector3, w: f32, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 0.5, 1.0, 0.0);
    for _ in 0..octaves {
        sum += sample4(noise, p * freq, w * freq.sqrt()) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.03;
    }
    sum / norm
}

/// Ruido "ridged": 1 - |ruido|, elevado al cuadrado. Da crestas finas, como cordilleras.
pub fn ridged(noise: &OpenSimplex, p: Vector3, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 0.5, 1.0, 0.0);
    for _ in 0..octaves {
        let r = 1.0 - sample(noise, p * freq).abs();
        sum += r * r * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.1;
    }
    sum / norm
}
