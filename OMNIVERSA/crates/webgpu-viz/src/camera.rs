use nalgebra::{Vector3, Matrix4, Point3, Perspective3, Isometry3, Translation3, UnitQuaternion};

pub struct OrbitalCamera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
    pub distance: f32,
    pub azimuth: f32,
    pub elevation: f32,
}

impl OrbitalCamera {
    pub fn new(aspect: f32) -> Self {
        Self {
            eye: Point3::new(0.0, 0.0, 5.0),
            target: Point3::origin(),
            up: Vector3::y_axis().into_inner(),
            fov: 45.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            aspect,
            distance: 5.0,
            azimuth: 0.0,
            elevation: 0.0,
        }
    }

    pub fn update_position(&mut self) {
        let x = self.distance * self.elevation.cos() * self.azimuth.cos();
        let y = self.distance * self.elevation.sin();
        let z = self.distance * self.elevation.cos() * self.azimuth.sin();

        self.eye = Point3::new(x, y, z);
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(&self.eye, &self.target, &self.up)
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        Perspective3::new(self.aspect, self.fov, self.near, self.far).to_homogeneous()
    }

    pub fn view_proj(&self) -> Matrix4<f32> {
        self.projection_matrix() * self.view_matrix()
    }
}

pub struct CameraController {
    pub camera: OrbitalCamera,
    pub rotate_speed: f32,
    pub zoom_speed: f32,
    pub pan_speed: f32,
}

impl CameraController {
    pub fn new(aspect: f32) -> Self {
        Self {
            camera: OrbitalCamera::new(aspect),
            rotate_speed: 0.005,
            zoom_speed: 0.1,
            pan_speed: 0.01,
        }
    }

    pub fn rotate(&mut self, delta_x: f32, delta_y: f32) {
        self.camera.azimuth += delta_x * self.rotate_speed;
        self.camera.elevation += delta_y * self.rotate_speed;
        self.camera.elevation = self.camera.elevation.clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.camera.update_position();
    }

    pub fn zoom(&mut self, delta: f32) {
        self.camera.distance *= 1.0 + delta * self.zoom_speed;
        self.camera.distance = self.camera.distance.clamp(0.1, 100.0);
        self.camera.update_position();
    }
}
