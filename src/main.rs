mod capture;
mod fragment;
mod framebuffer;
mod math;
mod obj;
mod pipeline;
mod scene;
mod shaders;
mod triangle;
mod uniforms;
mod vertex;

use framebuffer::Framebuffer;
use noise::OpenSimplex;
use raylib::prelude::*;
use scene::Scene;
use uniforms::Body;

const MODEL_PATH: &str = "assets/sphere.obj";
const WIDTH: i32 = 800;
const HEIGHT: i32 = 800;

fn main() {
    let model = obj::load(MODEL_PATH).unwrap_or_else(|e| panic!("{e}"));
    let noise = OpenSimplex::new(7);

    // `cargo run --release -- capture` genera las imágenes del README sin abrir ventana.
    if std::env::args().any(|a| a == "capture") {
        capture::run(&model, &noise);
        return;
    }

    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Lab 6 - Shaders: Solmira, Tessara y Orvane")
        .build();
    rl.set_target_fps(60);

    let mut render_scale: usize = 2; // 1 = resolución completa, 2 = mitad (más rápido)
    let mut fb = Framebuffer::new(WIDTH as usize / render_scale, HEIGHT as usize / render_scale);
    let make_texture = |rl: &mut RaylibHandle, fb: &Framebuffer| {
        let img = Image::gen_image_color(fb.width as i32, fb.height as i32, Color::BLACK);
        rl.load_texture_from_image(&thread, &img).expect("textura")
    };
    let mut texture = make_texture(&mut rl, &fb);

    let mut scene = Scene::new(Body::Rocky);
    let mut paused = false;
    let mut spinning = true;
    let mut light_orbit = false;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time().min(0.1);

        // ---------- Entrada ----------
        use KeyboardKey::*;
        let pressed = |k| rl.is_key_pressed(k);
        let down = |k| rl.is_key_down(k);
        if pressed(KEY_ONE) { scene.switch_to(Body::Star); }
        if pressed(KEY_TWO) { scene.switch_to(Body::Rocky); }
        if pressed(KEY_THREE) { scene.switch_to(Body::GasGiant); }
        if pressed(KEY_FOUR) { scene.switch_to(Body::Ball); }
        if pressed(KEY_FIVE) { scene.switch_to(Body::Orb); }
        if pressed(KEY_SPACE) { paused = !paused; }
        if pressed(KEY_R) { spinning = !spinning; }
        if pressed(KEY_L) { light_orbit = !light_orbit; }
        let p = &mut scene.params;
        if pressed(KEY_B) { p.bump = !p.bump; }
        if pressed(KEY_C) { p.clouds = !p.clouds; }
        if pressed(KEY_X) { p.cloud_shadows = !p.cloud_shadows; }
        if pressed(KEY_Z) { p.specular = !p.specular; }
        if pressed(KEY_N) { p.city_lights = !p.city_lights; }
        if down(KEY_RIGHT_BRACKET) { p.cloud_coverage = (p.cloud_coverage + dt * 0.3).min(1.0); }
        if down(KEY_LEFT_BRACKET) { p.cloud_coverage = (p.cloud_coverage - dt * 0.3).max(0.0); }
        if down(KEY_PERIOD) { p.bump_strength = (p.bump_strength + dt * 0.1).min(0.4); }
        if down(KEY_COMMA) { p.bump_strength = (p.bump_strength - dt * 0.1).max(0.0); }
        if down(KEY_EQUAL) { p.sea_level += dt * 0.1; }
        if down(KEY_MINUS) { p.sea_level -= dt * 0.1; }
        if down(KEY_NINE) { p.storm_strength = (p.storm_strength - dt * 3.0).max(0.0); }
        if down(KEY_ZERO) { p.storm_strength = (p.storm_strength + dt * 3.0).min(12.0); }
        if down(KEY_U) { p.star_speed = (p.star_speed - dt).max(0.0); }
        if down(KEY_I) { p.star_speed = (p.star_speed + dt).min(5.0); }
        if down(KEY_O) { p.star_activity = (p.star_activity - dt * 0.4).max(0.0); }
        if down(KEY_P) { p.star_activity = (p.star_activity + dt * 0.4).min(1.0); }
        if down(KEY_T) { p.turbulence = (p.turbulence - dt * 0.1).max(0.0); }
        if down(KEY_Y) { p.turbulence = (p.turbulence + dt * 0.1).min(0.5); }
        if down(KEY_F) { p.orb_ior = (p.orb_ior - dt * 0.3).max(1.0); }
        if down(KEY_G) { p.orb_ior = (p.orb_ior + dt * 0.3).min(2.2); }
        if down(KEY_V) { p.core_relief = (p.core_relief - dt * 0.2).max(0.0); }
        if down(KEY_M) { p.core_relief = (p.core_relief + dt * 0.2).min(0.45); }
        if pressed(KEY_J) { p.ball_stripes = (p.ball_stripes - 2.0).max(2.0); }
        if pressed(KEY_SEMICOLON) { p.ball_stripes = (p.ball_stripes + 2.0).min(24.0); }
        if down(KEY_Q) { scene.light_angle -= dt * 1.2; }
        if down(KEY_E) { scene.light_angle += dt * 1.2; }
        if down(KEY_LEFT) { scene.cam_yaw -= dt * 1.2; }
        if down(KEY_RIGHT) { scene.cam_yaw += dt * 1.2; }
        if down(KEY_UP) { scene.cam_pitch = (scene.cam_pitch + dt).min(1.4); }
        if down(KEY_DOWN) { scene.cam_pitch = (scene.cam_pitch - dt).max(-1.4); }
        if down(KEY_W) { scene.cam_dist = (scene.cam_dist - dt * 1.5).max(1.25); }
        if down(KEY_S) { scene.cam_dist = (scene.cam_dist + dt * 1.5).min(8.0); }
        let toggle_res = pressed(KEY_H);
        let screenshot = pressed(KEY_F12) || pressed(KEY_K);
        let wheel = rl.get_mouse_wheel_move();
        scene.cam_dist = (scene.cam_dist - wheel * 0.2).clamp(1.25, 8.0);
        if toggle_res {
            render_scale = if render_scale == 1 { 2 } else { 1 };
            fb = Framebuffer::new(WIDTH as usize / render_scale, HEIGHT as usize / render_scale);
            texture = make_texture(&mut rl, &fb);
        }

        // ---------- Actualización ----------
        if !paused { scene.time += dt; }
        if spinning { scene.spin += dt * 0.12; }
        if light_orbit { scene.light_angle += dt * 0.5; }

        // ---------- Render ----------
        let u = scene.uniforms(fb.width, fb.height, model.radius, &noise);
        pipeline::render(&mut fb, &model.vertices, &u);
        texture.update_texture(&fb.to_rgba()).expect("update texture");
        if screenshot {
            let name = format!("screenshots/captura_{:.0}.png", rl.get_time() * 1000.0);
            fb.save_png(&name);
        }

        let fps = rl.get_fps();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture_ex(&texture, Vector2::zero(), 0.0, render_scale as f32, Color::WHITE);
        let p = &scene.params;
        let on = |b: bool| if b { "si" } else { "no" };
        let info = match scene.body {
            Body::Star => format!("[O/P] actividad (manchas): {:.2}   [U/I] velocidad: {:.1}", p.star_activity, p.star_speed),
            Body::Rocky => format!(
                "[B] relieve: {}  [,/.] fuerza: {:.2}  [C] nubes: {}  [X] sombras: {}\n[ [ / ] ] cobertura: {:.2}  [Z] especular: {}  [N] ciudades: {}  [-/=] nivel mar: {:.2}",
                on(p.bump), p.bump_strength, on(p.clouds), on(p.cloud_shadows),
                p.cloud_coverage, on(p.specular), on(p.city_lights), p.sea_level
            ),
            Body::GasGiant => format!("[9/0] fuerza tormenta: {:.1}   [T/Y] turbulencia: {:.2}", p.storm_strength, p.turbulence),
            Body::Ball => format!("[J/;] rayas: {:.0}", p.ball_stripes),
            Body::Orb => format!("[F/G] refraccion: {:.2}   [V/M] relieve del nucleo: {:.2}", p.orb_ior, p.core_relief),
        };
        d.draw_text(&format!("{}   {} FPS", scene.body.name(), fps), 12, 10, 20, Color::RAYWHITE);
        d.draw_text("[1-5] cuerpo  [Espacio] pausa  [R] giro  [L] luz orbita  [Q/E] mover luz  [H] resolucion  [K] captura", 12, 36, 14, Color::LIGHTGRAY);
        d.draw_text("[Flechas] orbitar camara  [W/S o rueda] zoom", 12, 54, 14, Color::LIGHTGRAY);
        d.draw_text(&info, 12, HEIGHT - 44, 14, Color::LIGHTGRAY);
    }
}
