use crate::input::controls::InputState;
use crate::input::gamepad::GamepadInput;
use crate::physics::drone::{DroneConfig, DroneState};
use crate::render::framebuffer::Framebuffer;
use crate::render::terminal::RenderMode;
use crate::world::scene::Scene;

#[derive(Clone, Copy, PartialEq)]
pub enum AppState {
    Menu,
    Flying,
    AxisMapping,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MenuOption {
    Play,
    AxisMapping,
    Quit,
}

impl MenuOption {
    pub fn label(self) -> &'static str {
        match self {
            MenuOption::Play => "PLAY",
            MenuOption::AxisMapping => "AXIS MAPPING",
            MenuOption::Quit => "QUIT",
        }
    }
}

pub const MENU_OPTIONS: &[MenuOption] = &[
    MenuOption::Play,
    MenuOption::AxisMapping,
    MenuOption::Quit,
];

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
    pub state: AppState,
    pub menu_selection: usize,
    // Axis mapping state
    pub axis_map_selection: usize,
    pub axis_map_listening: bool,
    pub needs_clear: bool,
    pub kitty_supported: bool,
}

impl App {
    pub fn new(scene: Scene) -> Self {
        let config = DroneConfig::default();
        let drone = DroneState::new(scene.spawn_position, scene.spawn_orientation);
        let gamepad = GamepadInput::new();
        let kitty_supported = crate::render::terminal::detect_kitty_support();

        Self {
            drone,
            config,
            scene,
            framebuffer: Framebuffer::new(160, 96),
            input: InputState::new(),
            gamepad,
            render_mode: if kitty_supported { RenderMode::Kitty } else { RenderMode::HalfBlock },
            flight_time: 0.0,
            running: true,
            state: AppState::Menu,
            menu_selection: 0,
            axis_map_selection: 0,
            axis_map_listening: false,
            needs_clear: false,
            kitty_supported,
        }
    }

    pub fn reset(&mut self) {
        self.drone = DroneState::new(self.scene.spawn_position, self.scene.spawn_orientation);
        self.flight_time = 0.0;
    }

    pub fn resize_framebuffer(&mut self, term_cols: u16, term_rows: u16) {
        let (px_per_col, px_per_row) = self.render_mode.cell_pixels();
        let fb_w = term_cols as u32 * px_per_col;
        let fb_h = term_rows as u32 * px_per_row;
        self.framebuffer.resize(fb_w, fb_h);
    }
}
