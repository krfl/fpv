use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Smoothed analog stick state derived from binary keyboard input.
pub struct StickState {
    pub throttle: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl Default for StickState {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
        }
    }
}

/// Shared state between the input thread and main thread.
pub struct SharedInput {
    pub keys_last_press: HashMap<KeyCode, Instant>,
    pub has_release_support: bool,
    /// One-shot key presses consumed by the game loop each frame.
    pub pressed_keys: Vec<KeyCode>,
}

impl SharedInput {
    fn new() -> Self {
        Self {
            keys_last_press: HashMap::new(),
            has_release_support: false,
            pressed_keys: Vec::new(),
        }
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        match event.kind {
            KeyEventKind::Press => {
                self.keys_last_press.insert(event.code, Instant::now());
                self.pressed_keys.push(event.code);
            }
            KeyEventKind::Repeat => {
                self.keys_last_press.insert(event.code, Instant::now());
            }
            KeyEventKind::Release => {
                self.has_release_support = true;
                self.keys_last_press.remove(&event.code);
            }
        }
    }
}

const KEY_TIMEOUT_MS: u128 = 120;

/// Main-thread input state.
pub struct InputState {
    pub shared: Arc<Mutex<SharedInput>>,
    pub sticks: StickState,
    /// One-shot key presses from this frame (consumed each frame).
    pub pressed: Vec<KeyCode>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(SharedInput::new())),
            sticks: StickState::default(),
            pressed: Vec::new(),
        }
    }

    /// Drain one-shot key presses from the input thread.
    pub fn sync_from_shared(&mut self) {
        let mut shared = self.shared.lock().unwrap();
        self.pressed.clear();
        self.pressed.append(&mut shared.pressed_keys);
    }

    /// Check if a key was pressed this frame (one-shot).
    pub fn was_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    fn is_key_held(&self, key: KeyCode, shared: &SharedInput) -> bool {
        if let Some(&last_press) = shared.keys_last_press.get(&key) {
            if shared.has_release_support {
                true
            } else {
                last_press.elapsed().as_millis() < KEY_TIMEOUT_MS
            }
        } else {
            false
        }
    }

    /// Update stick positions based on held keys.
    pub fn update_sticks(&mut self, dt: f32) {
        let ramp_up = 3.0;
        let ramp_down = 4.0;
        let throttle_rate = 0.75;

        let (w_held, s_held, a_held, d_held, i_held, k_held, j_held, l_held);
        {
            let shared = self.shared.lock().unwrap();
            w_held = self.is_key_held(KeyCode::Char('w'), &shared)
                || self.is_key_held(KeyCode::Up, &shared);
            s_held = self.is_key_held(KeyCode::Char('s'), &shared)
                || self.is_key_held(KeyCode::Down, &shared);
            a_held = self.is_key_held(KeyCode::Char('a'), &shared);
            d_held = self.is_key_held(KeyCode::Char('d'), &shared);
            i_held = self.is_key_held(KeyCode::Char('i'), &shared);
            k_held = self.is_key_held(KeyCode::Char('k'), &shared);
            j_held = self.is_key_held(KeyCode::Char('j'), &shared);
            l_held = self.is_key_held(KeyCode::Char('l'), &shared);
        }

        if w_held {
            self.sticks.throttle = (self.sticks.throttle + throttle_rate * dt).min(1.0);
        }
        if s_held {
            self.sticks.throttle = (self.sticks.throttle - throttle_rate * dt).max(0.0);
        }

        self.sticks.yaw =
            Self::update_axis(self.sticks.yaw, a_held, d_held, ramp_up, ramp_down, dt);

        self.sticks.pitch = Self::update_axis(
            self.sticks.pitch,
            i_held,
            k_held,
            ramp_up * 2.0,
            ramp_down * 2.0,
            dt,
        );

        self.sticks.roll =
            Self::update_axis(self.sticks.roll, j_held, l_held, ramp_up, ramp_down, dt);
    }

    fn update_axis(
        current: f32,
        negative: bool,
        positive: bool,
        ramp_up: f32,
        ramp_down: f32,
        dt: f32,
    ) -> f32 {
        let target = match (negative, positive) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };

        if (target - current).abs() < 0.001 {
            return target;
        }

        let rate = if target.abs() > current.abs() || target.signum() != current.signum() {
            ramp_up
        } else {
            ramp_down
        };

        let delta = (target - current).signum() * rate * dt;
        let new_val = current + delta;

        if (current < target && new_val > target) || (current > target && new_val < target) {
            target
        } else {
            new_val.clamp(-1.0, 1.0)
        }
    }
}

/// Spawn the input polling thread.
pub fn spawn_input_thread(shared: Arc<Mutex<SharedInput>>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        use crossterm::event::{self, Event};
        use std::time::Duration;

        loop {
            if event::poll(Duration::from_millis(5)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    let mut shared = shared.lock().unwrap();
                    shared.handle_key_event(key);
                }
            }
        }
    })
}
