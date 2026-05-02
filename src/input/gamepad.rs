use gilrs::{Axis, EventType, Gilrs};

use super::controls::StickState;

/// Assignment of a gilrs axis to a drone control.
#[derive(Clone)]
pub struct AxisAssignment {
    pub axis: Axis,
    pub inverted: bool,
}

/// Configurable mapping from gamepad axes to drone controls.
pub struct AxisMapping {
    pub throttle: AxisAssignment,
    pub yaw: AxisAssignment,
    pub pitch: AxisAssignment,
    pub roll: AxisAssignment,
}

impl Default for AxisMapping {
    /// Mode 2 defaults (most common RC layout).
    fn default() -> Self {
        Self {
            throttle: AxisAssignment { axis: Axis::LeftStickY, inverted: false },
            yaw: AxisAssignment { axis: Axis::LeftStickX, inverted: false },
            pitch: AxisAssignment { axis: Axis::RightStickY, inverted: true },
            roll: AxisAssignment { axis: Axis::RightStickX, inverted: false },
        }
    }
}

impl AxisMapping {
    /// Get assignment by index (0=throttle, 1=yaw, 2=pitch, 3=roll).
    pub fn get_mut(&mut self, index: usize) -> &mut AxisAssignment {
        match index {
            0 => &mut self.throttle,
            1 => &mut self.yaw,
            2 => &mut self.pitch,
            3 => &mut self.roll,
            _ => panic!("invalid axis index"),
        }
    }
}

/// All axes we check when listening for stick input.
pub const ALL_AXES: [Axis; 6] = [
    Axis::LeftStickX,
    Axis::LeftStickY,
    Axis::RightStickX,
    Axis::RightStickY,
    Axis::LeftZ,
    Axis::RightZ,
];

/// Manages gamepad/radio controller input via gilrs.
pub struct GamepadInput {
    gilrs: Gilrs,
    pub connected: bool,
    pub name: String,
    pub sticks: StickState,
    pub mapping: AxisMapping,
}

impl GamepadInput {
    pub fn new() -> Option<Self> {
        let gilrs = Gilrs::new().ok()?;

        let mut connected = false;
        let mut name = String::new();
        for (_id, gamepad) in gilrs.gamepads() {
            connected = true;
            name = gamepad.name().to_string();
            break;
        }

        Some(Self {
            gilrs,
            connected,
            name,
            sticks: StickState::default(),
            mapping: AxisMapping::default(),
        })
    }

    fn read_axis(&self, gamepad: &gilrs::Gamepad, assignment: &AxisAssignment) -> f32 {
        let val = gamepad
            .axis_data(assignment.axis)
            .map(|a| a.value())
            .unwrap_or(0.0);
        if assignment.inverted { -val } else { val }
    }

    /// Poll gamepad events and update stick state.
    pub fn poll(&mut self) -> bool {
        while let Some(event) = self.gilrs.next_event() {
            match event.event {
                EventType::Connected => {
                    let gp = self.gilrs.gamepad(event.id);
                    self.connected = true;
                    self.name = gp.name().to_string();
                }
                EventType::Disconnected => {
                    self.connected = false;
                    self.name.clear();
                }
                _ => {}
            }
        }

        if !self.connected {
            return false;
        }

        if let Some((_id, gamepad)) = self.gilrs.gamepads().next() {
            if !gamepad.is_connected() {
                self.connected = false;
                return false;
            }

            let raw_throttle = self.read_axis(&gamepad, &self.mapping.throttle.clone());
            self.sticks.throttle = (raw_throttle * 0.5 + 0.5).clamp(0.0, 1.0);
            self.sticks.yaw = self.read_axis(&gamepad, &self.mapping.yaw.clone());
            self.sticks.pitch = self.read_axis(&gamepad, &self.mapping.pitch.clone());
            self.sticks.roll = self.read_axis(&gamepad, &self.mapping.roll.clone());

            true
        } else {
            false
        }
    }

    /// Find the axis with the largest deflection (for axis mapping detection).
    /// Returns Some(axis) if any axis exceeds the threshold.
    pub fn detect_axis(&self) -> Option<Axis> {
        if let Some((_id, gamepad)) = self.gilrs.gamepads().next() {
            let mut best_axis = None;
            let mut best_val = 0.5; // threshold

            for &axis in &ALL_AXES {
                if let Some(data) = gamepad.axis_data(axis) {
                    let val = data.value().abs();
                    if val > best_val {
                        best_val = val;
                        best_axis = Some(axis);
                    }
                }
            }

            best_axis
        } else {
            None
        }
    }
}
