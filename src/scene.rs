//! Estado de la escena (cámara, luz, giro) y construcción de los uniforms.
use crate::math;
use crate::uniforms::{Body, Params, Uniforms};
use noise::OpenSimplex;
use raylib::prelude::*;

#[derive(Clone, Copy)]
pub struct Scene {
    pub body: Body,
    pub time: f32,
    pub spin: f32,        // giro de la esfera (no cuenta como animación del material)
    pub light_angle: f32, // ángulo de la luz alrededor del eje Y
    pub cam_yaw: f32,
    pub cam_pitch: f32,
    pub cam_dist: f32,
    pub params: Params,
}

impl Scene {
    /// Cada cuerpo tiene su propio encuadre: distancia de cámara, ángulo y luz.
    /// La estrella se ve de frente y grande, el rocoso con la luz de lado para que se note
    /// la línea día/noche, y el gaseoso mirando hacia su tormenta.
    pub fn new(body: Body) -> Self {
        let (cam_yaw, cam_pitch, cam_dist, light_angle) = match body {
            Body::Star => (0.0, 0.10, 3.0, 0.0),
            Body::Rocky => (0.35, 0.22, 3.1, -0.75),
            Body::GasGiant => (0.75, -0.14, 2.85, 0.15),
            Body::Ball => (0.0, 0.12, 3.0, -0.6),
            Body::Orb => (0.0, 0.05, 3.0, -0.8),
        };
        Self {
            body,
            time: 0.0,
            spin: 0.0,
            light_angle,
            cam_yaw,
            cam_pitch,
            cam_dist,
            params: Params::default(),
        }
    }

    /// Cambia de cuerpo usando su encuadre, pero conserva el tiempo y los parámetros.
    pub fn switch_to(&mut self, body: Body) {
        let keep = (self.time, self.params);
        *self = Scene::new(body);
        self.time = keep.0;
        self.params = keep.1;
    }

    pub fn camera_pos(&self) -> Vector3 {
        Vector3::new(
            self.cam_dist * self.cam_pitch.cos() * self.cam_yaw.sin(),
            self.cam_dist * self.cam_pitch.sin(),
            self.cam_dist * self.cam_pitch.cos() * self.cam_yaw.cos(),
        )
    }

    pub fn light_dir(&self) -> Vector3 {
        Vector3::new(self.light_angle.sin(), 0.25, self.light_angle.cos()).normalized()
    }

    pub fn uniforms(&self, width: usize, height: usize, model_radius: f32, noise: &OpenSimplex) -> Uniforms {
        let size = 1.0;
        let tilt = match self.body {
            Body::Star => 0.12,
            Body::Rocky => 0.38,
            Body::GasGiant => 0.18,
            Body::Ball | Body::Orb => 0.0,
        };
        let translation = Vector3::zero();
        let camera_pos = self.camera_pos();
        Uniforms {
            model: math::model_matrix(translation, size / model_radius, tilt, self.spin),
            view: math::look_at(camera_pos, translation, Vector3::new(0.0, 1.0, 0.0)),
            projection: math::perspective(45f32.to_radians(), width as f32 / height as f32, 0.1, 100.0),
            width,
            height,
            time: self.time,
            light_dir: self.light_dir(),
            camera_pos,
            translation,
            size,
            tilt,
            spin: self.spin,
            noise: noise.clone(),
            body: self.body,
            params: self.params,
        }
    }
}
