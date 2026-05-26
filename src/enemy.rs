use macroquad::prelude::*;
use crate::Resources;
use crate::EnemyTextures;

#[derive(PartialEq, Clone, Copy)]
pub enum EnemyState { Idle, Chasing, Dead }

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyType { Skeleton1, Skeleton2, Vampire }

pub struct Enemy {
    pub x: f32, pub y: f32,
    pub width: f32, pub height: f32,
    pub speed: f32, pub hp: i32,
    pub state: EnemyState,
    pub enemy_type: EnemyType,
    pub alive: bool,
    pub frame: i32,
    pub frame_timer: f32,
    pub facing_left: bool,
}

impl Enemy {
    pub fn new(x: f32, y: f32, enemy_type: EnemyType) -> Self {
        let (speed, hp) = match enemy_type {
            EnemyType::Skeleton1 => (80.0, 3),
            EnemyType::Skeleton2 => (100.0, 4),
            EnemyType::Vampire   => (130.0, 5),
        };
        Self { x, y, width: 64.0, height: 64.0, speed, hp,
               state: EnemyState::Idle, enemy_type, alive: true,
               frame: 0, frame_timer: 0.0, facing_left: false }
    }

    pub fn draw(&self, res: &Resources) {
        let textures: &EnemyTextures = match self.enemy_type {
            EnemyType::Skeleton1 => &res.skeleton1,
            EnemyType::Skeleton2 => &res.skeleton2,
            EnemyType::Vampire   => &res.vampire,
        };
        let tex_opt = if self.state == EnemyState::Chasing { &textures.run } else { &textures.idle };

        if let Some(tex) = tex_opt {
            // Frames are square; height = frame height (single row sheets)
            let fh = tex.height();
            let fw = fh; // square frames
            let frame_count = ((tex.width() / fw) as i32).max(1);
            let col = self.frame % frame_count;

            draw_texture_ex(tex, self.x, self.y, WHITE, DrawTextureParams {
                dest_size: Some(vec2(self.width, self.height)),
                source: Some(Rect::new(col as f32 * fw, 0.0, fw, fh)),
                flip_x: self.facing_left,
                ..Default::default()
            });
        } else {
            let color = match self.enemy_type {
                EnemyType::Skeleton1 => GREEN,
                EnemyType::Skeleton2 => Color::from_rgba(0, 180, 0, 255),
                EnemyType::Vampire   => RED,
            };
            draw_rectangle(self.x, self.y, self.width, self.height, color);
        }
    }

    pub fn update(&mut self, player_x: f32, player_y: f32) {
        if self.state == EnemyState::Dead { return; }
        let dt = get_frame_time();
        let anim_rate = if self.state == EnemyState::Chasing { 0.1 } else { 0.15 };

        self.frame_timer += dt;
        if self.frame_timer > anim_rate { self.frame = self.frame.wrapping_add(1); self.frame_timer = 0.0; }

        let dx = player_x - self.x;
        let dy = player_y - self.y;
        let dist = (dx*dx + dy*dy).sqrt();

        match self.state {
            EnemyState::Idle => {
                if dist < 300.0 {
                    self.state = EnemyState::Chasing;
                    // FIX: set facing immediately on aggro so sprite doesn't lag one frame
                    if dist > 0.0 { self.facing_left = dx < 0.0; }
                }
            }
            EnemyState::Chasing => {
                if dist > 0.0 { self.facing_left = dx < 0.0; }
                if dist > 400.0 { self.state = EnemyState::Idle; }
            }
            EnemyState::Dead => {}
        }
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.hp -= damage;
        if self.hp <= 0 { self.state = EnemyState::Dead; self.alive = false; }
    }
}
