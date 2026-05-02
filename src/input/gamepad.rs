use gilrs::{Axis, EventType, Gilrs};

use super::controls::StickState;

/// Manages gamepad/radio controller input via gilrs.
/// Reads real analog stick values from USB HID joysticks like the RadioMaster TX16S/TX15.
pub struct GamepadInput {
    gilrs: Gilrs,
    pub connected: bool,
    pub name: String,
    /// Raw axis values from the controller [-1, 1]
    pub sticks: StickState,
}

impl GamepadInput {
    pub fn new() -> Option<Self> {
        let gilrs = Gilrs::new().ok()?;

        // Check if any gamepad is connected
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
        })
    }

    /// Poll gamepad events and update stick state.
    /// Returns true if a gamepad is active and providing input.
    pub fn poll(&mut self) -> bool {
        // Process all pending events
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

        // Read current axis values from the first connected gamepad
        if let Some((_id, gamepad)) = self.gilrs.gamepads().next() {
            if !gamepad.is_connected() {
                self.connected = false;
                return false;
            }

            // RC transmitters typically map:
            // Left stick Y = throttle (LeftStickY)
            // Left stick X = yaw (LeftStickX)
            // Right stick Y = pitch (RightStickY)
            // Right stick X = roll (RightStickX)
            //
            // Axis mapping varies by transmitter mode and USB config.
            // Mode 2 (most common):
            //   Left stick:  Y=throttle, X=yaw
            //   Right stick: Y=pitch (elevator), X=roll (aileron)

            // Read axes with deadzone applied by gilrs
            let left_x = gamepad
                .axis_data(Axis::LeftStickX)
                .map(|a| a.value())
                .unwrap_or(0.0);
            let left_y = gamepad
                .axis_data(Axis::LeftStickY)
                .map(|a| a.value())
                .unwrap_or(0.0);
            let right_x = gamepad
                .axis_data(Axis::RightStickX)
                .map(|a| a.value())
                .unwrap_or(0.0);
            let right_y = gamepad
                .axis_data(Axis::RightStickY)
                .map(|a| a.value())
                .unwrap_or(0.0);

            // Map to drone controls (Mode 2)
            // Throttle: left Y, mapped from [-1,1] to [0,1]
            self.sticks.throttle = (left_y * 0.5 + 0.5).clamp(0.0, 1.0);
            self.sticks.yaw = left_x;
            self.sticks.pitch = -right_y; // invert: stick forward = pitch forward
            self.sticks.roll = right_x;

            true
        } else {
            false
        }
    }
}
