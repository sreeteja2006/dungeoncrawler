use macroquad::prelude::*;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub speed: f32,
    pub invulnerable_timer: f32,
    pub frame: i32,
    pub frame_timer: f32,
    pub facing_left: bool,
    pub moving: bool,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, width: 64.0, height: 64.0, hp: 3, max_hp: 3, speed: 250.0,
               invulnerable_timer: 0.0, frame: 0, frame_timer: 0.0,
               facing_left: false, moving: false }
    }

    pub fn draw(&self, texture: Option<&Texture2D>) {
        let blink = self.invulnerable_timer > 0.0 && (get_time() * 15.0) as i32 % 2 == 0;
        let color = if blink { Color::from_rgba(255, 255, 255, 100) } else { WHITE };

        if let Some(tex) = texture {
            // Dungeon_Character.png: 16x16 frames, row 0 = idle, row 1 = walk
            let fw = 16.0_f32;
            let fh = 16.0_f32;
            let total_frames = (tex.width() / fw).max(1.0) as i32;
            let walk_rows = (tex.height() / fh) as i32;

            // Use row 1 for walking if sheet has multiple rows, else row 0
            let row = if self.moving && walk_rows > 1 { 1 } else { 0 };
            let col = self.frame % total_frames;

            draw_texture_ex(tex, self.x, self.y, color, DrawTextureParams {
                dest_size: Some(vec2(self.width, self.height)),
                source: Some(Rect::new(col as f32 * fw, row as f32 * fh, fw, fh)),
                flip_x: self.facing_left,
                ..Default::default()
            });
        } else {
            draw_rectangle(self.x, self.y, self.width, self.height, if blink { SKYBLUE } else { BLUE });
        }
    }
}
