use std::io::Stdout;
use std::time::{Duration, Instant};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::App;
use crate::input::controls::StickState;
use crate::physics::drone::{physics_step, DroneState};
use crate::render::camera;
use crate::render::font;
use crate::render::framebuffer::{Color, Framebuffer};
use crate::render::rasterizer;
use crate::render::terminal::{self, FpvView, RenderMode};

const PHYSICS_HZ: f32 = 240.0;
const PHYSICS_DT: f32 = 1.0 / PHYSICS_HZ;
const MAX_FRAME_TIME: f32 = 0.1;
const TARGET_FPS: u64 = 24;

pub fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> std::io::Result<()> {
    let mut accumulator = 0.0f32;
    let mut last_time = Instant::now();
    let frame_duration = Duration::from_millis(1000 / TARGET_FPS);

    // Spawn dedicated input thread
    let input_shared = app.input.shared.clone();
    let input_handle = crate::input::controls::spawn_input_thread(input_shared);

    loop {
        let now = Instant::now();
        let frame_time = now.duration_since(last_time).as_secs_f32().min(MAX_FRAME_TIME);
        last_time = now;
        accumulator += frame_time;

        // Sync input state from the input thread
        app.input.sync_from_shared();

        if !app.running || app.input.quit_requested {
            if app.render_mode == RenderMode::Kitty {
                let _ = terminal::cleanup_kitty();
            }
            break;
        }

        // Handle reset
        if app.input.reset_requested {
            app.reset();
            app.input.clear_reset();
        }

        // Handle render mode toggle
        if app.input.render_mode_toggle {
            if app.render_mode == RenderMode::Kitty {
                let _ = terminal::cleanup_kitty();
            }
            app.render_mode = app.render_mode.toggle();
            app.input.clear_render_toggle();
            if app.render_mode == RenderMode::Kitty {
                terminal::reset_kitty_frame_counter();
            }
            let size = terminal.size()?;
            app.resize_framebuffer(size.width, size.height);
            terminal.clear()?;
        }

        // Update stick values: gamepad takes priority over keyboard
        let use_gamepad = if let Some(ref mut gp) = app.gamepad {
            gp.poll()
        } else {
            false
        };

        if use_gamepad {
            // Copy gamepad stick values directly (real analog input)
            let gp = app.gamepad.as_ref().unwrap();
            app.input.sticks.throttle = gp.sticks.throttle;
            app.input.sticks.yaw = gp.sticks.yaw;
            app.input.sticks.pitch = gp.sticks.pitch;
            app.input.sticks.roll = gp.sticks.roll;
        } else {
            // Fall back to keyboard with smoothing
            app.input.update_sticks(frame_time);
        }

        // Fixed timestep physics
        while accumulator >= PHYSICS_DT {
            physics_step(&mut app.drone, &app.config, &app.input.sticks, &app.scene.colliders, PHYSICS_DT);
            if !app.drone.crashed {
                app.flight_time += PHYSICS_DT;
            }
            accumulator -= PHYSICS_DT;
        }

        // Get terminal size
        let size = terminal.size()?;
        app.resize_framebuffer(size.width, size.height);

        // Rasterize scene
        let view = camera::fpv_view_matrix(&app.drone, &app.config);
        let aspect = app.framebuffer.width as f32 / app.framebuffer.height as f32;
        let proj = camera::projection_matrix(120.0, aspect);
        let view_proj = proj * view;

        let sky = Color::new(40, 60, 120);
        app.framebuffer.clear(sky);

        for mesh in &app.scene.meshes {
            rasterizer::rasterize_mesh(&mut app.framebuffer, mesh, &view_proj);
        }

        // Pixel HUD rendered into framebuffer
        render_pixel_hud(
            &mut app.framebuffer,
            &app.input.sticks,
            &app.drone,
            app.flight_time,
            app.gamepad.as_ref().filter(|gp| gp.connected).map(|gp| gp.name.as_str()),
        );

        // Render based on mode
        if app.render_mode == RenderMode::Kitty {
            terminal::render_kitty_frame(
                &app.framebuffer,
                size.width,
                size.height,
            )?;
        } else {
            terminal.draw(|frame| {
                let area = frame.area();
                let fpv = FpvView {
                    framebuffer: &app.framebuffer,
                    mode: app.render_mode,
                };
                frame.render_widget(fpv, area);
            })?;
        }

        // Frame rate limiting
        let elapsed = now.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    // Wait for input thread to finish
    let _ = input_handle.join();

    Ok(())
}

/// Render HUD directly into the framebuffer as pixels (for kitty mode).
fn render_pixel_hud(
    fb: &mut Framebuffer,
    sticks: &StickState,
    drone: &DroneState,
    flight_time: f32,
    gamepad_name: Option<&str>,
) {
    let w = fb.width as i32;
    let h = fb.height as i32;

    // Auto-scale font based on framebuffer size
    let scale = if w >= 900 { 3 } else if w >= 450 { 2 } else { 1 };
    let char_w = 8 * scale as i32;
    let pad = scale as i32 * 2;

    let white = Color::new(240, 240, 240);
    let green = Color::new(80, 255, 80);
    let yellow = Color::new(255, 255, 80);
    let gray = Color::new(160, 160, 160);
    let red = Color::new(255, 60, 60);
    let cyan = Color::new(80, 220, 255);
    let bg = Some(Color::new(15, 15, 15));

    // Flight timer (top center)
    let secs = flight_time as u32;
    let timer = format!("{:02}:{:02}", secs / 60, secs % 60);
    let tx = w / 2 - (timer.len() as i32 * char_w) / 2;
    font::draw_string(fb, &timer, tx, pad, white, bg, scale);

    // Controls / gamepad name (top left)
    if let Some(name) = gamepad_name {
        let label: String = name.chars().take(16).collect();
        font::draw_string(fb, &label, pad, pad, cyan, bg, scale);
    } else {
        font::draw_string(fb, "W/S THR", pad, pad, gray, bg, scale);
        font::draw_string(fb, "IJKL FLY", pad, pad + 8 * scale as i32 + 2, gray, bg, scale);
    }

    // Bottom row
    let bot_y = h - 8 * scale as i32 - pad;

    // Throttle (bottom left)
    let throttle_pct = (sticks.throttle * 100.0) as u32;
    let thr_str = format!("THR {:3}%", throttle_pct);
    font::draw_string(fb, &thr_str, pad, bot_y, green, bg, scale);

    // Altitude (bottom center)
    let alt_str = format!("ALT {:.1}M", drone.position.y);
    let ax = w / 2 - (alt_str.len() as i32 * char_w) / 2;
    font::draw_string(fb, &alt_str, ax, bot_y, white, bg, scale);

    // Speed (bottom right)
    let spd_str = format!("{:.1} M/S", drone.velocity.norm());
    let sx = w - spd_str.len() as i32 * char_w - pad;
    font::draw_string(fb, &spd_str, sx, bot_y, yellow, bg, scale);

    // Crashed indicator
    if drone.crashed {
        let msg = "CRASHED - PRESS R";
        let cx = w / 2 - (msg.len() as i32 * char_w) / 2;
        let cy = h / 2 - 4 * scale as i32;
        font::draw_string(fb, msg, cx, cy, red, Some(Color::new(40, 0, 0)), scale);
    }
}
