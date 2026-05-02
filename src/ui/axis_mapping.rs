use crate::input::gamepad::GamepadInput;
use crate::render::font;
use crate::render::framebuffer::{Color, Framebuffer};

/// Render the axis mapping screen into the framebuffer.
pub fn render_axis_mapping(
    fb: &mut Framebuffer,
    selection: usize,
    listening: bool,
    gamepad: Option<&GamepadInput>,
) {
    fb.color.fill(Color::new(10, 10, 25));

    let w = fb.width as i32;
    let h = fb.height as i32;
    if w < 16 || h < 16 {
        return;
    }

    // Widest line: "> THROTTLE: RIGHTSTICKX INV" ~28 chars
    let scale: u32 = ((w as u32) / (28 * 8)).min(3).max(1);
    let char_w = 8 * scale as i32;
    let line_h = 8 * scale as i32 + 4;

    // Title
    let title = "AXIS MAPPING";
    let title_px = title.len() as i32 * char_w;
    font::draw_string(fb, title, (w - title_px) / 2, h / 8, Color::new(255, 200, 50), None, scale);

    let controls = ["THROTTLE", "YAW", "PITCH", "ROLL"];
    let gamepad_ref = gamepad.filter(|gp| gp.connected);

    let start_y = h / 4;
    for (i, control) in controls.iter().enumerate() {
        let y = start_y + i as i32 * line_h;
        let is_selected = i == selection;
        let is_listening_this = is_selected && listening;

        let display = if is_listening_this {
            format!("{}: MOVE STICK...", control)
        } else if let Some(gp) = gamepad_ref {
            let assignment = match i {
                0 => &gp.mapping.throttle,
                1 => &gp.mapping.yaw,
                2 => &gp.mapping.pitch,
                3 => &gp.mapping.roll,
                _ => unreachable!(),
            };
            let inv = if assignment.inverted { " INV" } else { "" };
            format!("{}: {:?}{}", control, assignment.axis, inv)
        } else {
            format!("{}: NO GAMEPAD", control)
        };

        let prefix = if is_selected { "> " } else { "  " };
        let text = format!("{}{}", prefix, display);
        let text_px = text.len() as i32 * char_w;

        let color = if is_listening_this {
            Color::new(255, 255, 80)
        } else if is_selected {
            Color::new(255, 255, 255)
        } else {
            Color::new(90, 90, 110)
        };

        let bg = if is_selected {
            Some(Color::new(40, 40, 60))
        } else {
            None
        };

        font::draw_string(fb, &text, (w - text_px) / 2, y, color, bg, scale);
    }

    // Instructions
    let hint_scale = scale.max(2) - 1;
    let hint = "ENTER MAP  V INVERT  ESC BACK";
    let hint_px = hint.len() as i32 * 8 * hint_scale as i32;
    font::draw_string(
        fb,
        hint,
        (w - hint_px) / 2,
        h - hint_scale as i32 * 10,
        Color::new(50, 50, 70),
        None,
        hint_scale,
    );
}
