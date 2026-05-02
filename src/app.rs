use crate::input::controls::InputState;
use crate::input::gamepad::GamepadInput;
use crate::physics::drone::{DroneConfig, DroneState};
use crate::render::framebuffer::Framebuffer;
use crate::render::terminal::RenderMode;
use crate::world::scene::Scene;

pub struct App {
    pub drone: DroneState,
    pub config: DroneConfig,
    pub scene: Scene,
    pub framebuffer: Framebuffer,
    pub input: InputState,
    pub gamepad: Option<GamepadInput>,
    pub render_mode: RenderMode,
    pub flight_time: f32,
    pub running: bool,
}

impl App {
    pub fn new(scene: Scene) -> Self {
        let config = DroneConfig::default();
        let drone = DroneState::new(scene.spawn_position, scene.spawn_orientation);
        let gamepad = GamepadInput::new();

        if let Some(ref gp) = gamepad {
            if gp.connected {
                eprintln!("Gamepad detected: {}", gp.name);
            }
        }

        Self {
            drone,
            config,
            scene,
            framebuffer: Framebuffer::new(160, 96),
            input: InputState::new(),
            gamepad,
            render_mode: RenderMode::HalfBlock,
            flight_time: 0.0,
            running: true,
        }
    }

    pub fn reset(&mut self) {
        self.drone = DroneState::new(self.scene.spawn_position, self.scene.spawn_orientation);
        self.flight_time = 0.0;
        self.input.reset_requested = false;
    }

    /// Resize framebuffer based on terminal size and render mode.
    pub fn resize_framebuffer(&mut self, term_cols: u16, term_rows: u16) {
        let (px_per_col, px_per_row) = self.render_mode.cell_pixels();
        let fb_w = term_cols as u32 * px_per_col;
        let fb_h = term_rows as u32 * px_per_row;
        self.framebuffer.resize(fb_w, fb_h);
    }
}
