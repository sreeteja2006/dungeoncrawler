use crate::enemy::{Enemy, EnemyType};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::Resources;

pub enum ItemType { Health, Speed }
pub struct Item { pub x: f32, pub y: f32, pub item_type: ItemType, pub collected: bool }
pub enum PickupType { Heart }
pub struct Pickup { pub x: f32, pub y: f32, pub pickup_type: PickupType }

pub struct Room {
    pub room_type: RoomType,
    pub enemies: Vec<Enemy>,
    pub items: Vec<Item>,
    pub pickups: Vec<Pickup>,
    pub obstacles: Vec<Rect>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum RoomType { Normal, Boss, Item, Start }

const WALL: f32 = 48.0;
const MARGIN: f32 = WALL + 20.0;

impl Room {
    pub fn new(room_type: RoomType) -> Self {
        let mut enemies = Vec::new();
        let mut obstacles = Vec::new();
        let mut items = Vec::new();

        let sw = screen_width();
        let sh = screen_height();

        if room_type != RoomType::Start {
            // Generate obstacles first so enemy spawns can avoid them
            let obs_count = gen_range(5, 10);
            for _ in 0..obs_count {
                let (ow, oh) = (64.0_f32, 64.0_f32);
                // FIX: limit iterations to prevent infinite loop in edge-case tiny windows
                let mut placed = false;
                for _ in 0..200 {
                    let ox = gen_range(MARGIN, (sw - MARGIN - ow).max(MARGIN + 1.0));
                    let oy = gen_range(MARGIN, (sh - MARGIN - oh).max(MARGIN + 1.0));
                    let r = Rect::new(ox, oy, ow, oh);
                    let safe = Rect::new(sw/2.0-120.0, sh/2.0-120.0, 240.0, 240.0);
                    if !r.overlaps(&safe) { obstacles.push(Rect::new(ox, oy, ow, oh)); placed = true; break; }
                }
                let _ = placed; // allow failure silently rather than hang
            }

            // Spawn enemies avoiding obstacles and center safe zone
            let count = match room_type { RoomType::Boss => 1, _ => gen_range(3, 7) };
            for _ in 0..count {
                let ew = 64.0_f32; let eh = 64.0_f32;
                // FIX: limit iterations to prevent infinite loop
                let mut spawn_pos = None;
                for _ in 0..300 {
                    let x = gen_range(MARGIN, (sw - MARGIN - ew).max(MARGIN + 1.0));
                    let y = gen_range(MARGIN, (sh - MARGIN - eh).max(MARGIN + 1.0));
                    let cx = sw / 2.0; let cy = sh / 2.0;
                    let far_from_center = ((x-cx).powi(2) + (y-cy).powi(2)).sqrt() > 150.0;
                    let e_rect = Rect::new(x, y, ew, eh);
                    let no_obs_overlap = obstacles.iter().all(|o| !e_rect.overlaps(o));
                    if far_from_center && no_obs_overlap { spawn_pos = Some((x, y)); break; }
                }
                // Fall back to a corner if no valid position found
                let (x, y) = spawn_pos.unwrap_or((MARGIN, MARGIN));

                let e_type = if room_type == RoomType::Boss { EnemyType::Vampire }
                    else { match gen_range(0, 3) { 0 => EnemyType::Skeleton1, 1 => EnemyType::Skeleton2, _ => EnemyType::Vampire } };

                let mut e = Enemy::new(x, y, e_type);
                if room_type == RoomType::Boss { e.width = 120.0; e.height = 120.0; e.hp = 20; e.speed = 120.0; }
                enemies.push(e);
            }
        }

        if room_type == RoomType::Item {
            let t = if gen_range(0, 2) == 0 { ItemType::Health } else { ItemType::Speed };
            items.push(Item { x: sw/2.0, y: sh/2.0, item_type: t, collected: false });
        }

        Self { room_type, enemies, items, pickups: Vec::new(), obstacles }
    }

    pub fn draw(&self, res: &Resources) {
        let sw = screen_width();
        let sh = screen_height();
        let ts = 64.0_f32;

        // Floor
        for gx in (0..(sw as i32)).step_by(ts as usize) {
            for gy in (0..(sh as i32)).step_by(ts as usize) {
                if let Some(ref tex) = res.obstacle {
                    draw_texture_ex(tex, gx as f32, gy as f32, WHITE, DrawTextureParams {
                        dest_size: Some(vec2(ts, ts)),
                        source: Some(Rect::new(16.0, 64.0, 16.0, 16.0)),
                        ..Default::default()
                    });
                } else {
                    draw_rectangle(gx as f32, gy as f32, ts, ts, Color::from_rgba(50, 50, 60, 255));
                }
            }
        }

        // Walls
        let wc = Color::from_rgba(35, 35, 42, 255);
        let w = WALL;
        draw_rectangle(0.0, 0.0, sw, w, wc);
        draw_rectangle(0.0, sh-w, sw, w, wc);
        draw_rectangle(0.0, 0.0, w, sh, wc);
        draw_rectangle(sw-w, 0.0, w, sh, wc);

        // Obstacles
        for obs in &self.obstacles {
            if let Some(ref tex) = res.obstacle {
                draw_texture_ex(tex, obs.x, obs.y, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(obs.w, obs.h)),
                    source: Some(Rect::new(32.0, 0.0, 32.0, 32.0)),
                    ..Default::default()
                });
            } else {
                draw_rectangle(obs.x, obs.y, obs.w, obs.h, BROWN);
            }
        }

        // Items
        for item in &self.items {
            if !item.collected {
                let tex_opt = match item.item_type { ItemType::Health => &res.health_item, ItemType::Speed => &res.speed_item };
                if let Some(tex) = tex_opt {
                    draw_texture_ex(tex, item.x-32.0, item.y-32.0, WHITE, DrawTextureParams { dest_size: Some(vec2(64.0, 64.0)), ..Default::default() });
                } else {
                    draw_rectangle(item.x-20.0, item.y-20.0, 40.0, 40.0, match item.item_type { ItemType::Health => RED, _ => GREEN });
                }
            }
        }

        // Pickups
        let hf = (get_time() * 8.0) as i32 % 3;
        for p in &self.pickups {
            if let Some(ref tex) = res.heart {
                let fh = tex.height() / 3.0;
                draw_texture_ex(tex, p.x-20.0, p.y-20.0, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(40.0, 40.0)),
                    source: Some(Rect::new(0.0, hf as f32 * fh, tex.width(), fh)),
                    ..Default::default()
                });
            } else { draw_circle(p.x, p.y, 12.0, RED); }
        }

        for e in &self.enemies { e.draw(res); }
    }
}
