//! Cargador sencillo de archivos .obj (v, vn, f). Triangula caras con abanico.
use crate::vertex::Vertex;
use raylib::prelude::*;
use std::fs;

pub struct Model {
    pub vertices: Vec<Vertex>, // ya en triángulos: cada 3 vértices = 1 triángulo
    pub radius: f32,           // radio máximo, para normalizar el tamaño
}

pub fn load(path: &str) -> Result<Model, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("No se pudo leer {path}: {e}"))?;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut vertices = Vec::new();

    let parse3 = |it: &mut std::str::SplitWhitespace| -> Vector3 {
        let mut n = [0.0f32; 3];
        for v in n.iter_mut() {
            *v = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        }
        Vector3::new(n[0], n[1], n[2])
    };

    for line in text.lines() {
        let mut it = line.split_whitespace();
        match it.next() {
            Some("v") => positions.push(parse3(&mut it)),
            Some("vn") => normals.push(parse3(&mut it)),
            Some("f") => {
                let corners: Vec<Vertex> = it
                    .map(|tok| {
                        let mut parts = tok.split('/');
                        let idx = |s: Option<&str>, len: usize| -> Option<usize> {
                            let i: i64 = s.filter(|s| !s.is_empty())?.parse().ok()?;
                            Some(if i < 0 { (len as i64 + i) as usize } else { (i - 1) as usize })
                        };
                        let pi = idx(parts.next(), positions.len()).unwrap_or(0);
                        let _ti = parts.next();
                        let ni = idx(parts.next(), normals.len());
                        let position = positions[pi];
                        let normal = ni
                            .and_then(|i| normals.get(i).copied())
                            .unwrap_or_else(|| position.normalized());
                        Vertex { position, normal }
                    })
                    .collect();
                for i in 1..corners.len().saturating_sub(1) {
                    vertices.push(corners[0]);
                    vertices.push(corners[i]);
                    vertices.push(corners[i + 1]);
                }
            }
            _ => {}
        }
    }

    // Centro y radio, para que cualquier esfera quede de radio 1.
    let center = positions.iter().fold(Vector3::zero(), |a, p| a + *p) * (1.0 / positions.len().max(1) as f32);
    for v in vertices.iter_mut() {
        v.position = v.position - center;
    }
    let radius = vertices.iter().map(|v| v.position.length()).fold(0.0f32, f32::max).max(1e-6);
    Ok(Model { vertices, radius })
}
