use nalgebra::{Isometry3, Matrix4, Perspective3, UnitQuaternion, Vector3};

use crate::physics::drone::{DroneConfig, DroneState};

/// Compute the FPV camera view matrix from drone state.
/// Camera is at drone position, oriented by drone quaternion,
/// then tilted upward by camera_tilt_deg around the body X axis.
pub fn fpv_view_matrix(drone: &DroneState, config: &DroneConfig) -> Matrix4<f32> {
    let camera_tilt = UnitQuaternion::from_axis_angle(
        &Vector3::x_axis(),
        config.camera_tilt_deg.to_radians(),
    );

    let camera_orientation = drone.orientation * camera_tilt;

    let isometry = Isometry3::from_parts(
        nalgebra::Translation3::from(drone.position.coords),
        camera_orientation,
    );

    isometry.inverse().to_homogeneous()
}

/// Build a perspective projection matrix.
pub fn projection_matrix(fov_deg: f32, aspect: f32) -> Matrix4<f32> {
    Perspective3::new(aspect, fov_deg.to_radians(), 0.05, 200.0).to_homogeneous()
}

