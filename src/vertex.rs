use raylib::prelude::*;

/// Vértice tal como viene del .obj (espacio del objeto).
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vector3,
    pub normal: Vector3,
}

/// Vértice después del vertex shader.
#[derive(Clone, Copy, Debug)]
pub struct TransformedVertex {
    pub screen: Vector3,          // x, y en píxeles; z = profundidad NDC
    pub inv_w: f32,               // 1/w para interpolar con perspectiva correcta
    pub world_position: Vector3,
    pub world_normal: Vector3,
    pub object_position: Vector3,
}
