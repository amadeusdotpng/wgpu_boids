pub struct Boid {
    pos: [f32; 2],
    rot: f32,
}

pub type BoidUniform = [[f32; 3]; 3];
impl Boid {
    pub fn new(x: f32, y: f32, rot: f32) -> Self {
        Boid { pos: [x, y], rot, }
    }

    pub fn into_matrix(&self) -> BoidUniform {
        let (rot_sin, rot_cos) = f32::sin_cos(self.rot);
        let [x, y]  = self.pos;
        [[ rot_cos, rot_sin, 0.],
         [-rot_sin, rot_cos, 0.],
         [x       , y      , 1.]]
    }
}

impl Default for Boid {
    fn default() -> Self {
        Self { pos: [0., 0.], rot: 0., }
    }
}
