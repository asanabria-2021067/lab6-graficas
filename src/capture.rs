//! Modo captura: genera las imágenes y cuadros de GIF que usa el README.
//! Uso: cargo run --release -- capture [filtro]
use crate::framebuffer::Framebuffer;
use crate::obj::Model;
use crate::pipeline;
use crate::scene::Scene;
use crate::uniforms::Body;
use noise::OpenSimplex;
use std::f32::consts::PI;

fn render(scene: &Scene, size: usize, model: &Model, noise: &OpenSimplex) -> Framebuffer {
    let mut fb = Framebuffer::new(size, size);
    let u = scene.uniforms(size, size, model.radius, noise);
    pipeline::render(&mut fb, &model.vertices, &u);
    fb
}

fn still(name: &str, scene: Scene, size: usize, m: &Model, nz: &OpenSimplex) {
    render(&scene, size, m, nz).save_png(&format!("docs/{name}.png"));
    println!("docs/{name}.png");
}

/// Dos renders lado a lado (antes | después) en una sola imagen.
fn pair(name: &str, a: Scene, b: Scene, size: usize, m: &Model, nz: &OpenSimplex) {
    let fa = render(&a, size, m, nz);
    let fb = render(&b, size, m, nz);
    let (ia, ib) = (fa.to_rgba(), fb.to_rgba());
    let mut out = image::RgbaImage::new(size as u32 * 2 + 6, size as u32);
    for y in 0..size {
        for x in 0..size {
            let i = (y * size + x) * 4;
            out.put_pixel(x as u32, y as u32, image::Rgba([ia[i], ia[i + 1], ia[i + 2], 255]));
            out.put_pixel((x + size + 6) as u32, y as u32, image::Rgba([ib[i], ib[i + 1], ib[i + 2], 255]));
        }
    }
    let _ = std::fs::create_dir_all("docs");
    out.save(format!("docs/{name}.png")).unwrap();
    println!("docs/{name}.png");
}

/// Cuadros para un GIF; `f` recibe el índice del cuadro y su avance de 0 a 1.
fn frames(name: &str, count: usize, fps: u32, size: usize, m: &Model, nz: &OpenSimplex, f: impl Fn(usize, f32) -> Scene) {
    let dir = format!("docs/frames/{name}");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    std::fs::write(format!("{dir}/fps.txt"), fps.to_string()).unwrap();
    for i in 0..count {
        let s = f(i, i as f32 / count as f32);
        render(&s, size, m, nz).save_png(&format!("{dir}/{i:03}.png"));
    }
    println!("{dir} ({count} cuadros)");
}

pub fn run(m: &Model, nz: &OpenSimplex) {
    let filter = std::env::args().skip_while(|a| a != "capture").nth(1).unwrap_or_default();
    let want = |n: &str| filter.is_empty() || n.contains(filter.as_str());
    let big = 640;
    let gif = 320; // GIFs de evidencia
    let anim = 300; // GIFs de la galería principal

    let star = Scene { time: 3.0, ..Scene::new(Body::Star) };
    let rocky = Scene { time: 4.0, spin: 0.4, ..Scene::new(Body::Rocky) };
    let gas = Scene { time: 2.0, ..Scene::new(Body::GasGiant) };
    let ball = Scene::new(Body::Ball);
    let orb = Scene { time: 2.0, ..Scene::new(Body::Orb) };

    // Vista cercana del planeta rocoso con la luz rasante (resalta relieve y sombras).
    let mut close = rocky;
    close.cam_dist = 2.25;
    close.cam_yaw = 0.25;
    close.cam_pitch = 0.25;
    close.light_angle = -0.75;

    // Cámara mirando de frente a la tormenta.
    let mut storm_view = gas;
    {
        let u = gas.uniforms(8, 8, m.radius, nz);
        let (lat0, lon0) = (-0.38f32, 0.6f32);
        let c = raylib::prelude::Vector3::new(lat0.cos() * lon0.sin(), lat0.sin(), lat0.cos() * lon0.cos());
        let w = u.object_dir_to_world(c);
        storm_view.cam_yaw = w.x.atan2(w.z);
        storm_view.cam_pitch = w.y.asin();
        storm_view.cam_dist = 2.2;
        storm_view.light_angle = storm_view.cam_yaw - 0.5;
    }

    // ---------- Imágenes principales ----------
    if want("estrella") { still("estrella", star, big, m, nz); }
    if want("rocoso") { still("rocoso", rocky, big, m, nz); }
    if want("gaseoso") { still("gaseoso", gas, big, m, nz); }

    if want("balon") { still("balon", ball, big, m, nz); still("balon_atras", Scene { cam_yaw: std::f32::consts::PI, light_angle: std::f32::consts::PI - 0.6, ..ball }, 420, m, nz); }
    if want("orbe") { still("orbe", orb, big, m, nz); }
    if want("parallax") {
        // Mismo tiempo, cámara en dos ángulos: los grabados de la superficie se mueven con la esfera
        // pero el núcleo de adentro se desplaza distinto (profundidad).
        let a = Scene { cam_yaw: -0.45, ..orb };
        let b = Scene { cam_yaw: 0.45, ..orb };
        pair("orbe_parallax", a, b, 420, m, nz);
        let mut flat = orb; flat.params.orb_ior = 1.0; flat.params.core_relief = 0.0;
        pair("orbe_relieve", flat, orb, 420, m, nz);
    }

    // ---------- Comparaciones de parámetros ----------
    if want("manchas") {
        let mut a = star; a.params.star_activity = 0.1;
        let mut b = star; b.params.star_activity = 0.9;
        pair("estrella_actividad", a, b, 420, m, nz);
    }
    if want("relieve") {
        let mut a = close; a.params.bump = false; a.params.clouds = false;
        let mut b = close; b.params.clouds = false;
        pair("rocoso_relieve", a, b, 420, m, nz);
    }
    if want("sombras") {
        let mut b = close; b.params.cloud_coverage = 0.45; b.light_angle = -0.6;
        let mut a = b; a.params.cloud_shadows = false;
        pair("rocoso_sombras", a, b, 420, m, nz);
    }
    if want("cobertura") {
        let mut a = rocky; a.params.cloud_coverage = 0.3;
        let mut b = rocky; b.params.cloud_coverage = 0.6;
        pair("rocoso_cobertura", a, b, 420, m, nz);
    }
    if want("especular") {
        let mut a = rocky; a.params.specular = false; a.params.clouds = false; a.light_angle = 0.35;
        let mut b = rocky; b.params.clouds = false; b.light_angle = 0.35;
        pair("rocoso_especular", a, b, 420, m, nz);
    }
    if want("noche") {
        let mut a = rocky; a.light_angle = 2.3; a.params.city_lights = false;
        let mut b = rocky; b.light_angle = 2.3;
        pair("rocoso_noche", a, b, 420, m, nz);
    }
    if want("tormenta") {
        let mut a = storm_view; a.params.storm_strength = 0.0;
        let b = storm_view;
        pair("gaseoso_tormenta", a, b, 420, m, nz);
    }
    if want("bandas") {
        let mut a = gas; a.params.turbulence = 0.0;
        let mut b = gas; b.params.turbulence = 0.3;
        pair("gaseoso_turbulencia", a, b, 420, m, nz);
    }

    // ---------- Galería: un GIF por cuerpo (30 fps, la cámara recorre 90° y pasa el tiempo) ----------
    if want("anim") {
        frames("anim_estrella", 90, 30, anim, m, nz, |_, k| Scene {
            time: 3.0 + k * 9.0,
            cam_yaw: star.cam_yaw + k * PI / 2.0,
            ..star
        });
        frames("anim_rocoso", 90, 30, anim, m, nz, |_, k| Scene {
            time: 4.0 + k * 12.0,
            cam_yaw: rocky.cam_yaw + k * PI / 2.0,
            light_angle: rocky.light_angle + k * PI / 2.0, // la luz acompaña a la cámara
            ..rocky
        });
        frames("anim_gaseoso", 90, 30, anim, m, nz, |_, k| Scene {
            time: 2.0 + k * 12.0,
            cam_yaw: gas.cam_yaw - PI / 4.0 + k * PI / 2.0,
            light_angle: gas.light_angle - PI / 4.0 + k * PI / 2.0,
            ..gas
        });
    }

    if want("anim") {
        frames("anim_balon", 90, 30, anim, m, nz, |_, k| Scene { cam_yaw: k * 2.0 * PI, light_angle: -0.6 + k * 2.0 * PI, ..ball });
        frames("anim_orbe", 90, 30, anim, m, nz, |_, k| Scene { time: 2.0 + k * 8.0, cam_yaw: (k * 2.0 * PI).sin() * 0.6, ..orb });
    }
    if want("gif_parallax") {
        // Tiempo congelado: lo único que cambia es la cámara.
        frames("orbe_parallax", 60, 20, gif, m, nz, |_, k| Scene { cam_yaw: (k * 2.0 * PI).sin() * 0.6, cam_pitch: 0.05 + (k * 2.0 * PI).cos() * 0.15, ..orb });
    }

    // ---------- GIFs de evidencia de cada efecto ----------
    if want("gif_estrella") {
        // cámara y esfera quietas: solo cambia el material
        frames("estrella", 60, 20, gif, m, nz, |_, k| Scene { time: 3.0 + k * 10.0, ..star });
    }
    if want("gif_nubes") {
        // La esfera NO gira: solo pasa el tiempo. Los continentes quedan quietos y las nubes se mueven.
        frames("rocoso_nubes", 60, 20, gif, m, nz, |_, k| Scene { time: 4.0 + k * 30.0, ..rocky });
    }
    if want("gif_luz") {
        frames("rocoso_luz", 72, 20, gif, m, nz, |_, k| Scene { light_angle: rocky.light_angle + k * 2.0 * PI, ..rocky });
    }
    if want("gif_relieve") {
        frames("rocoso_relieve", 60, 20, gif, m, nz, |_, k| {
            let mut s = close;
            s.params.clouds = false;
            s.light_angle = -0.75 + (k * 2.0 * PI).sin() * 0.7;
            s
        });
    }
    if want("gif_sombras") {
        frames("rocoso_sombras", 60, 20, gif, m, nz, |_, k| {
            let mut s = close;
            s.params.cloud_coverage = 0.45;
            s.light_angle = -0.85 + (k * 2.0 * PI).sin() * 0.55;
            s
        });
    }
    if want("gif_gaseoso") {
        frames("gaseoso", 75, 20, gif, m, nz, |_, k| Scene { time: 2.0 + k * 25.0, ..gas });
    }
}
