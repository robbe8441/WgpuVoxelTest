use cgmath::{Vector3, InnerSpace};
use winit::{event::*, keyboard::PhysicalKey};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
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
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

pub struct CameraController {
    sensitivity: f32,
    mouse_input: [f64; 2],
    zoom_dis: f32,
    move_direction: cgmath::Vector3<f32>,

    w_key : bool,
    a_key : bool,
    s_key : bool,
    d_key : bool,
    space_key : bool,
    c_key : bool,

}

fn get_key_dir(key1:bool, key2:bool) -> f32 {
    let val1 = if key1 {1.0} else {0.0};
    let val2 = if key2 {-1.0} else {0.0};
    val1 + val2
}




impl CameraController {
    pub fn new(sensitivity: f32) -> Self {
        Self {
            sensitivity,
            mouse_input: [0.0, 0.0],
            zoom_dis: 2.0,
            move_direction : cgmath::Vector3::new(0.0, 0.0, 0.0),

            w_key : false,
            a_key : false,
            s_key : false,
            d_key : false,
            space_key : false,
            c_key : false,
        }
    }

    pub fn process_events(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta, .. } => {
                let senv = self.sensitivity as f64;
                self.mouse_input = [
                    self.mouse_input[0] + delta.0.to_radians() * senv,
                    self.mouse_input[1] + delta.1.to_radians() * senv,
                ];
            }

            DeviceEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_x, y) => {
                    self.zoom_dis = (self.zoom_dis - y / 10.0).max(1.0);
                }
                _ => {}
            },

            DeviceEvent::Key(RawKeyEvent { physical_key, state }) => {
                let change = if state == &ElementState::Pressed {true} else {false};
                use winit::keyboard::KeyCode;


                match physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.w_key = change,
                    PhysicalKey::Code(KeyCode::KeyS) => self.s_key = change,
                    PhysicalKey::Code(KeyCode::KeyA) => self.a_key = change,
                    PhysicalKey::Code(KeyCode::KeyD) => self.d_key = change,
                    PhysicalKey::Code(KeyCode::Space) => self.space_key = change,
                    PhysicalKey::Code(KeyCode::KeyC) => self.c_key = change,
                    _ => {}
                }

                self.move_direction = Vector3::new(
                    get_key_dir(self.a_key, self.d_key), 
                    get_key_dir(self.space_key, self.c_key), 
                    get_key_dir(self.w_key, self.s_key));
            }
            _ => {}
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let x_input = self.mouse_input[0];
        let y_input = self.mouse_input[1];

        let x = y_input.sin() * x_input.cos();
        let y = -y_input.cos();
        let z = y_input.sin() * x_input.sin();

        let look_v = Vector3::new(x as f32, y as f32, z as f32);
        let right_v = look_v.cross(Vector3::unit_y());

        let pos = (look_v * self.move_direction.z)
            + (right_v * -self.move_direction.x)
            + (Vector3::unit_y() * self.move_direction.y);


        camera.eye += pos / 100.0;
        camera.target = camera.eye + Vector3::new(x as f32, y as f32, z as f32);
    }
}
