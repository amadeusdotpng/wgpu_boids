use winit::{
    event::*,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct Camera {
    scale_factor: f32,
    scale: [f32; 2],
    position: [f32; 2],
}

pub type CameraUniform = [[f32; 4]; 3];

impl Camera {
    pub fn new(viewport_size: winit::dpi::PhysicalSize<u32>) -> Self {
        let scale_factor = 10.0;
        let scale = [
            1.0 / viewport_size.width as f32,
            1.0 / viewport_size.height as f32,
        ];

        let position = [
            0.0,
            0.0
        ];

        Self { scale_factor, scale, position }
    }

    pub fn update_scale(&mut self, new_viewport_size: winit::dpi::PhysicalSize<u32>) {
        self.scale = [
            10.0 / new_viewport_size.width as f32,
            10.0 / new_viewport_size.height as f32,

        ];
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(keycode),
                    ..
                },
                ..
            } => match keycode {
                KeyCode::KeyW | KeyCode::ArrowUp    => { self.position[1] += 0.05; true }
                KeyCode::KeyS | KeyCode::ArrowDown  => { self.position[1] -= 0.05; true }
                KeyCode::KeyD | KeyCode::ArrowLeft  => { self.position[0] += 0.05; true }
                KeyCode::KeyA | KeyCode::ArrowRight => { self.position[0] -= 0.05; true }
                KeyCode::KeyE  => { self.scale_factor += 0.5; true }
                KeyCode::KeyQ  => { self.scale_factor -= 0.5; true }
                _ => false
            }
            _ => false
        }
    }


    pub fn into_matrix(&self) -> CameraUniform {
        let sf = self.scale_factor;
        let [sx, sy] = self.scale;
        let [px, py] = self.position;
        [[sf*sx, 0.   , 0., 0.],
         [0.   , sf*sy, 0., 0.],
         [-px  , -py  , 1., 0.]]
    }
}

