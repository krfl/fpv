use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

use crate::physics::drone::DroneState;
use crate::input::controls::StickState;

#[allow(dead_code)]
pub struct Hud<'a> {
    pub drone: &'a DroneState,
    pub sticks: &'a StickState,
    pub flight_time: f32,
    pub gamepad_name: Option<&'a str>,
}

impl Widget for Hud<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let white = Style::default().fg(Color::White).bg(Color::Reset);
        let green = Style::default().fg(Color::Green).bg(Color::Reset);
        let yellow = Style::default().fg(Color::Yellow).bg(Color::Reset);

        // Flight timer (top center)
        let secs = self.flight_time as u32;
        let mins = secs / 60;
        let timer = format!("{:02}:{:02}", mins, secs % 60);
        let tx = area.x + area.width / 2 - 2;
        buf.set_string(tx, area.y, &timer, white);

        // Throttle (bottom left)
        let throttle_pct = (self.sticks.throttle * 100.0) as u32;
        let thr_str = format!("THR {:3}%", throttle_pct);
        buf.set_string(area.x + 1, area.y + area.height - 2, &thr_str, green);

        // Speed (bottom right)
        let speed = self.drone.velocity.norm();
        let spd_str = format!("{:.1} m/s", speed);
        let sx = area.x + area.width - spd_str.len() as u16 - 1;
        buf.set_string(sx, area.y + area.height - 2, &spd_str, yellow);

        // Altitude (bottom center)
        let alt_str = format!("ALT {:.1}m", self.drone.position.y);
        let ax = area.x + area.width / 2 - alt_str.len() as u16 / 2;
        buf.set_string(ax, area.y + area.height - 2, &alt_str, white);

        // Crashed indicator
        if self.drone.crashed {
            let crash_msg = ">>> CRASHED - Press R to reset <<<";
            let cx = area.x + area.width / 2 - crash_msg.len() as u16 / 2;
            let cy = area.y + area.height / 2;
            let red = Style::default().fg(Color::Red).bg(Color::Reset);
            buf.set_string(cx, cy, crash_msg, red);
        }

        // Controls help / gamepad indicator (top left)
        if let Some(name) = self.gamepad_name {
            let label = if name.len() > 20 { &name[..20] } else { name };
            buf.set_string(area.x + 1, area.y, label, Style::default().fg(Color::Cyan));
        } else {
            buf.set_string(area.x + 1, area.y, "W/S", Style::default().fg(Color::DarkGray));
            buf.set_string(area.x + 5, area.y, "thr", Style::default().fg(Color::DarkGray));
            buf.set_string(area.x + 1, area.y + 1, "IJKL", Style::default().fg(Color::DarkGray));
            buf.set_string(area.x + 6, area.y + 1, "fly", Style::default().fg(Color::DarkGray));
        }
    }
}
