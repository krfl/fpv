/// Betaflight-style rate curve.
/// Converts a stick deflection [-1, 1] to a normalized rate [-1, 1]
/// that gets multiplied by max_rate_deg_s in the physics step.
///
/// - `rc_rate`: Overall scaling (typically 0.5-2.0)
/// - `expo`: Softens center stick (0.0 = linear, 1.0 = full cubic)
/// - `super_rate`: Increases rate at stick extremes (0.0-1.0)
pub fn apply_rate_curve(stick: f32, rc_rate: f32, expo: f32, super_rate: f32) -> f32 {
    let stick_abs = stick.abs();

    // Expo: blend between linear and cubic
    let expo_value = stick * (1.0 - expo + expo * stick_abs * stick_abs);

    // Super rate: increases sensitivity at stick extremes
    let super_factor = 1.0 / (1.0 - stick_abs * super_rate).max(0.01);

    expo_value * rc_rate * super_factor
}
