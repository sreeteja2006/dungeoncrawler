use macroquad::prelude::*;

pub struct Projectile {
    pub x: f32, pub y: f32,
    pub vx: f32, pub vy: f32,
    pub alive: bool,
}

impl Projectile {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Self { x, y, vx, vy, alive: true }
    }
    pub fn update(&mut self) {
        let dt = get_frame_time();
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }
    pub fn draw(&self, texture: Option<&Texture2D>) {
        if let Some(tex) = texture {
            draw_texture_ex(tex, self.x-16.0, self.y-16.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(32.0, 32.0)), ..Default::default()
            });
        } else {
            draw_circle(self.x, self.y, 8.0, YELLOW);
        }
    }
}
