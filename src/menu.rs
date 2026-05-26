use macroquad::prelude::*;

pub struct Menu {
    t: f32,
    selected: usize, // 0=Play, 1=Quit
}

impl Menu {
    pub fn new() -> Self { Self { t: 0.0, selected: 0 } }

    /// Returns true when Play is confirmed
    pub fn update_draw(&mut self) -> bool {
        self.t += get_frame_time();
        let sw = screen_width();
        let sh = screen_height();

        // ── Animated background ──────────────────────────────────────────────
        for i in 0..20 {
            let x = ((i as f32 * 137.5 + self.t * 15.0) % sw).abs();
            let y = ((i as f32 * 97.3  + self.t * 8.0)  % sh).abs();
            let a = ((self.t + i as f32).sin() * 0.5 + 0.5) * 80.0;
            draw_circle(x, y, 2.0, Color::from_rgba(80, 40, 120, a as u8));
        }

        // ── Title ────────────────────────────────────────────────────────────
        let pulse = ((self.t * 2.0).sin() * 4.0) as f32;
        let title = "DUNGEON CRAWLER";
        let tsz   = 60.0 + pulse;
        let tw    = measure_text(title, None, tsz as u16, 1.0).width;
        // Shadow
        draw_text(title, sw/2.0 - tw/2.0 + 3.0, sh*0.28 + 3.0, tsz, Color::from_rgba(0,0,0,180));
        draw_text(title, sw/2.0 - tw/2.0,        sh*0.28,        tsz, Color::from_rgba(220,160,40,255));

        let sub = "A roguelike dungeon adventure";
        let sw2 = measure_text(sub, None, 22, 1.0).width;
        draw_text(sub, sw/2.0 - sw2/2.0, sh*0.38, 22.0, Color::from_rgba(160,140,180,200));

        // ── Navigation ───────────────────────────────────────────────────────
        if is_key_pressed(KeyCode::Up)   || is_key_pressed(KeyCode::W) { if self.selected > 0 { self.selected -= 1; } }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) { if self.selected < 1 { self.selected += 1; } }

        let items = [("  PLAY GAME  ", Color::from_rgba(80,200,100,255)),
                     ("    QUIT     ", Color::from_rgba(200,80,80,255))];

        for (i, (label, color)) in items.iter().enumerate() {
            let selected = i == self.selected;
            let by = sh * 0.55 + i as f32 * 80.0;
            let bw = 260.0_f32; let bh = 52.0_f32;
            let bx = sw/2.0 - bw/2.0;

            // Button shadow + bg
            draw_rectangle(bx+4.0, by+4.0, bw, bh, Color::from_rgba(0,0,0,120));
            let bg = if selected { Color::from_rgba(color.r as u8, color.g as u8, color.b as u8, 60) }
                     else        { Color::from_rgba(20,20,30,200) };
            draw_rectangle(bx, by, bw, bh, bg);
            // Border
            let border = if selected { *color } else { Color::from_rgba(80,70,100,255) };
            draw_rectangle_lines(bx, by, bw, bh, if selected { 3.0 } else { 1.5 }, border);
            // Arrow indicator
            if selected {
                draw_text("▶", bx - 28.0, by + bh*0.68, 26.0, *color);
                draw_text("◀", bx + bw + 4.0, by + bh*0.68, 26.0, *color);
            }
            // Label
            let lw = measure_text(label, None, 28, 1.0).width;
            let lcolor = if selected { *color } else { Color::from_rgba(160,150,180,255) };
            draw_text(label, sw/2.0 - lw/2.0, by + bh*0.68, 28.0, lcolor);
        }

        // Controls hint
        let hint = "W/S or ↑/↓ to navigate  •  ENTER to select";
        let hw = measure_text(hint, None, 16, 1.0).width;
        draw_text(hint, sw/2.0 - hw/2.0, sh*0.88, 16.0, Color::from_rgba(120,110,140,200));

        // Controls info box
        let info = "WASD: Move   Arrow Keys: Shoot   R: Restart";
        let iw = measure_text(info, None, 15, 1.0).width;
        draw_text(info, sw/2.0 - iw/2.0, sh*0.93, 15.0, Color::from_rgba(100,100,130,180));

        // Confirm
        let confirm = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        if confirm {
            if self.selected == 0 { return true; }
            if self.selected == 1 { std::process::exit(0); }
        }
        false
    }
}
