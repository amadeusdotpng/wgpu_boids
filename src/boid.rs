#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Boid {
    pos: [f32; 2],
    vel: [f32; 2],
}

impl Boid {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Self { pos: [x, y], vel: [vx, vy]}
    }
}

impl Default for Boid {
    fn default() -> Self {
        Self { pos: [0., 0.], vel: [0., 0.] }
    }
}
