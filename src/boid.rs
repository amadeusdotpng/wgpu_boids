#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Boid {
    pos: [f32; 2],
    rot: f32,
    _padding: [f32; 1],
}

impl Boid {
    pub fn new(x: f32, y: f32, rot: f32) -> Self {
        Boid { pos: [x, y], rot, _padding: [0.0]}
    }
}

impl Default for Boid {
    fn default() -> Self {
        Self { pos: [0., 0.], rot: 0., _padding: [0.0] }
    }
}
