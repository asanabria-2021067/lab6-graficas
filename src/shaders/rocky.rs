//! Tessara: planeta rocoso con océanos, biomas, casquetes de hielo, relieve (bump mapping),
//! nubes que se mueven solas, sombras de nubes, reflejos en el océano y ciudades en la noche.
use super::noise_helpers::{fbm, ridged, sample};
use super::{mix, rgb, smoothstep, surface};
use crate::math::rotate_y;
use crate::{fragment::Fragment, uniforms::Uniforms};
use noise::OpenSimplex;
use raylib::prelude::*;

/// Forma de los continentes: fBm deformado por otro fBm (domain warping) para que
/// las costas se vean orgánicas y no como manchas redondas.
fn continents(nz: &OpenSimplex, p: Vector3) -> f32 {
    let q = p * 1.3;
    let warp = Vector3::new(
        fbm(nz, q + Vector3::new(3.1, 0.0, 0.0), 3),
        fbm(nz, q + Vector3::new(0.0, 7.4, 0.0), 3),
        fbm(nz, q + Vector3::new(0.0, 0.0, 1.9), 3),
    );
    fbm(nz, q + warp * 0.9, 6)
}

/// Campo de altura del relieve fino (montañas). `mountains` dice cuánta cordillera hay
/// en esta zona; se calcula una vez afuera porque cambia lento.
fn detail_height(nz: &OpenSimplex, p: Vector3, mountains: f32) -> f32 {
    ridged(nz, p * 5.0, 4) * mountains + fbm(nz, p * 18.0, 2) * 0.12
}

/// Bump mapping: derivamos la altura en dos direcciones tangentes (diferencias finitas)
/// e inclinamos la normal en contra de la pendiente. Los vértices no se tocan.
fn bump_normal(nz: &OpenSimplex, p: Vector3, n: Vector3, mountains: f32, strength: f32) -> Vector3 {
    let up = if n.y.abs() < 0.99 { Vector3::new(0.0, 1.0, 0.0) } else { Vector3::new(1.0, 0.0, 0.0) };
    let t = up.cross(n).normalized();
    let b = n.cross(t);
    let e = 0.002;
    let h0 = detail_height(nz, p, mountains);
    let dt = (detail_height(nz, p + t * e, mountains) - h0) / e;
    let db = (detail_height(nz, p + b * e, mountains) - h0) / e;
    (n - t * (dt * strength) - b * (db * strength)).normalized()
}

/// Máscara de nubes (0 = sin nube, 1 = nube densa). Las nubes giran sobre el eje Y
/// más rápido que el terreno y además se deforman con el tiempo.
pub fn cloud_mask(nz: &OpenSimplex, p: Vector3, time: f32, coverage: f32) -> f32 {
    let pc = rotate_y(p, time * 0.07);
    let swirl = fbm(nz, pc * 1.8 + Vector3::new(0.0, time * 0.03, 0.0), 3);
    let q = pc * 3.0 + Vector3::new(swirl, swirl * 0.4, -swirl) * 1.2;
    // estiramos en X/Z para que las nubes salgan alargadas, como frentes
    let c = fbm(nz, Vector3::new(q.x, q.y * 1.8, q.z), 5) * 0.5 + 0.5;
    let start = 1.0 - coverage;
    smoothstep(start - 0.04, start + 0.09, c)
}

pub fn rocky(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    let s = surface(fragment, u);
    let (p, n, l, v) = (s.p, s.n, s.l, s.v);
    let nz = &u.noise;
    let pr = &u.params;

    // ---------- 1. Regiones de la superficie ----------
    let h = continents(nz, p);
    let land = smoothstep(pr.sea_level - 0.008, pr.sea_level + 0.008, h);
    let elevation = (h - pr.sea_level).max(0.0);
    let mountains = smoothstep(0.04, 0.22, elevation);
    let ridge = ridged(nz, p * 5.0, 3) * mountains;

    let lat = p.y.abs();
    let moisture = fbm(nz, p * 2.4 + Vector3::new(8.0, 2.0, 5.0), 3) * 0.5 + 0.5;
    let cold = lat + elevation * 1.4 + sample(nz, p * 4.0) * 0.06;

    // Tierra: desierto -> pradera -> bosque según humedad; tundra y nieve según el frío.
    let desert = rgb(0.80, 0.62, 0.38);
    let grass = rgb(0.36, 0.52, 0.22);
    let forest = rgb(0.10, 0.30, 0.16);
    let tundra = rgb(0.52, 0.50, 0.42);
    let rock = rgb(0.40, 0.33, 0.28);
    let snow = rgb(0.94, 0.96, 1.0);
    let mut land_col = mix(desert, grass, smoothstep(0.38, 0.55, moisture));
    land_col = mix(land_col, forest, smoothstep(0.58, 0.75, moisture));
    land_col = mix(land_col, tundra, smoothstep(0.55, 0.72, cold));
    land_col = mix(land_col, rock, smoothstep(0.35, 0.75, ridge));
    land_col = mix(land_col, snow, smoothstep(0.62, 0.8, ridge) * smoothstep(0.35, 0.55, cold));

    // Océano: turquesa en la costa, azul profundo lejos de ella.
    let depth = (pr.sea_level - h).max(0.0);
    let ocean_col = mix(rgb(0.08, 0.50, 0.58), rgb(0.02, 0.09, 0.26), smoothstep(0.0, 0.18, depth));

    // Casquetes polares: cubren tierra y mar por encima de cierta latitud (borde con ruido).
    let ice = smoothstep(0.80, 0.86, lat + fbm(nz, p * 3.5, 3) * 0.12);
    let ice_col = rgb(0.90, 0.95, 1.0);

    let mut albedo = mix(ocean_col, land_col, land);
    albedo = mix(albedo, ice_col, ice);

    // ---------- 2. Normal con relieve (bump mapping) ----------
    let ndl_geo = n.dot(l);
    let shading_n = if pr.bump && land > 0.0 && ice < 1.0 {
        let bumped = bump_normal(nz, p, n, mountains + 0.25, pr.bump_strength);
        mix(n, bumped, land * (1.0 - ice)).normalized()
    } else {
        n
    };

    // ---------- 3. Nubes y sus sombras ----------
    let clouds = if pr.clouds { cloud_mask(nz, p, u.time, pr.cloud_coverage) } else { 0.0 };
    let shadow = if pr.clouds && pr.cloud_shadows && ndl_geo > -0.1 {
        // Rayo desde el suelo hacia la luz: ¿dónde cruza la capa de nubes (radio 1 + altura)?
        // |p + t·l|² = r²  ->  t = -(p·l) + sqrt((p·l)² + r² - 1)
        let r = 1.0 + pr.cloud_height;
        let pl = p.dot(l);
        let t = -pl + (pl * pl + r * r - 1.0).sqrt();
        let above = (p + l * t).normalized();
        1.0 - cloud_mask(nz, above, u.time, pr.cloud_coverage) * 0.85
    } else {
        1.0
    };

    // ---------- 4. Iluminación distinta por región ----------
    // el relieve no puede iluminar más allá de la línea día/noche de la esfera
    let diffuse = shading_n.dot(l).max(0.0) * smoothstep(-0.05, 0.1, ndl_geo);
    let ambient = 0.025;
    let mut color = albedo * (ambient + diffuse * shadow * 1.05);

    if pr.specular {
        // Blinn-Phong: océano muy brillante y concentrado, hielo un poco, tierra nada (mate).
        let half = (l + v).normalized();
        let ndh = n.dot(half).max(0.0);
        let ocean = (1.0 - land) * (1.0 - ice);
        let spec = ndh.powf(90.0) * 1.1 * ocean + ndh.powf(14.0) * 0.18 * ice;
        let lit = smoothstep(0.0, 0.08, ndl_geo);
        color = color + rgb(1.0, 0.95, 0.85) * (spec * lit * shadow);
    }

    // ---------- 5. Lado nocturno: luces de ciudades ----------
    let day = smoothstep(-0.12, 0.18, ndl_geo);
    if pr.city_lights {
        let coast = 1.0 - smoothstep(0.06, 0.22, elevation); // ciudades cerca de la costa
        let habitable = land * coast * (1.0 - ice) * (1.0 - smoothstep(0.6, 0.75, cold));
        let clusters = smoothstep(-0.05, 0.35, fbm(nz, p * 7.0 + Vector3::new(4.0, 4.0, 4.0), 2));
        let dots = smoothstep(0.2, 0.6, sample(nz, p * 70.0));
        let city = habitable * clusters * (0.35 + dots * 0.9);
        color = color + rgb(1.0, 0.72, 0.32) * (city * (1.0 - day) * (1.0 - clouds * 0.85) * 1.6);
    }

    // Nubes encima de todo: se iluminan con la normal sin relieve.
    let cloud_light = 0.03 + ndl_geo.max(0.0) * 1.05;
    color = mix(color, rgb(0.97, 0.98, 1.0) * cloud_light, clouds * 0.92);

    // Atmósfera: halo azul en el borde del lado iluminado.
    let rim = (1.0 - n.dot(v).max(0.0)).powf(3.0);
    color + rgb(0.30, 0.55, 1.0) * (rim * smoothstep(-0.25, 0.5, ndl_geo) * 0.7)
}
