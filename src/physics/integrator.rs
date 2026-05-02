use nalgebra::{Quaternion, UnitQuaternion, Vector3};

/// Integrate orientation quaternion given angular velocity (body frame) and timestep.
/// Uses first-order quaternion integration with renormalization.
pub fn integrate_quaternion(
    q: &UnitQuaternion<f32>,
    omega: &Vector3<f32>,
    dt: f32,
) -> UnitQuaternion<f32> {
    // q_dot = 0.5 * q * omega_quat
    let omega_quat = Quaternion::new(0.0, omega.x, omega.y, omega.z);
    let q_dot = q.as_ref() * omega_quat * 0.5;
    let new_q = Quaternion::new(
        q.w + q_dot.w * dt,
        q.i + q_dot.i * dt,
        q.j + q_dot.j * dt,
        q.k + q_dot.k * dt,
    );
    UnitQuaternion::new_normalize(new_q)
}
