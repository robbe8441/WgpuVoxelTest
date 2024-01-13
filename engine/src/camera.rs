use winit::event::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn default(config: &wgpu::SurfaceConfiguration) -> Self {
        Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 40.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

pub struct CameraController {
    sensitivity: f32,
    input_position: [f64; 2],
    zoom_dis : f32,
}

impl CameraController {
    pub fn new(sensitivity: f32) -> Self {
        Self {
            sensitivity,
            input_position : [0.0, 0.0],
            zoom_dis : 2.0,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) {
        match event {

            WindowEvent::CursorMoved { position, .. } => {
                let senv = self.sensitivity as f64;
                self.input_position = [position.x.to_radians() * senv, position.y.to_radians() * senv];
            }

            WindowEvent::MouseWheel {delta, ..} => {
                match delta {
                    MouseScrollDelta::LineDelta(_x, y) => {
                        self.zoom_dis = (self.zoom_dis - y / 10.0).max(1.0);
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let x_input = self.input_position[0];
        let y_input = self.input_position[1];

        let x = y_input.sin() * x_input.cos();
        let y = -y_input.cos();
        let z = y_input.sin() * x_input.sin();

        camera.eye = cgmath::Point3::new(x as f32, y as f32, z as f32) * self.zoom_dis;
    }
}
