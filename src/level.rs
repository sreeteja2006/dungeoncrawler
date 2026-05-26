use std::collections::HashMap;
use crate::room::*;
use crate::player::*;
use crate::projectiles::*;
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::Resources;
use crate::enemy::EnemyState;

pub struct Level {
    pub rooms: HashMap<(i32,i32), Room>,
    pub current_room: (i32,i32),
    pub player: Player,
    pub projectiles: Vec<Projectile>,
}

const WALL: f32 = 48.0;
const DOOR_HALF: f32 = 40.0;

impl Level {
    pub fn new() -> Self {
        let mut rooms = HashMap::new();
        let target_count = 12;
        let mut coords = vec![(0, 0)];
        let mut type_map = HashMap::new();
        type_map.insert((0, 0), RoomType::Start);

        while type_map.len() < target_count {
            let (x, y) = coords[gen_range(0, coords.len())];
            let next_pos = match gen_range(0, 4) {
                0 => (x, y - 1), 1 => (x, y + 1), 2 => (x - 1, y), _ => (x + 1, y),
            };
            if !type_map.contains_key(&next_pos) {
                type_map.insert(next_pos, RoomType::Normal);
                coords.push(next_pos);
            }
        }

        if coords.len() > 3 {
            type_map.insert(coords[gen_range(1, coords.len() - 1)], RoomType::Item);
            type_map.insert(*coords.last().unwrap(), RoomType::Boss);
        }

        for (pos, r_type) in type_map {
            rooms.insert(pos, Room::new(r_type));
        }

        let sw = screen_width();
        let sh = screen_height();
        Self {
            rooms,
            current_room: (0, 0),
            player: Player::new(sw / 2.0, sh / 2.0),
            projectiles: Vec::new(),
        }
    }

    pub fn update_level(&mut self) {
        let sw = screen_width();
        let sh = screen_height();
        let dt = get_frame_time();

        if self.player.hp <= 0 {
            if is_key_pressed(KeyCode::R) { *self = Level::new(); }
            return;
        }

        // ── Decrement timers ──
        self.player.invulnerable_timer = (self.player.invulnerable_timer - dt).max(0.0);

        let old_px = self.player.x;
        let old_py = self.player.y;
        let pw = self.player.width;
        let ph = self.player.height;

        // ── Player movement ──
        let mut moving = false;
        if is_key_down(KeyCode::W) { self.player.y -= self.player.speed * dt; moving = true; }
        if is_key_down(KeyCode::A) { self.player.x -= self.player.speed * dt; moving = true; }
        if is_key_down(KeyCode::S) { self.player.y += self.player.speed * dt; moving = true; }
        if is_key_down(KeyCode::D) { self.player.x += self.player.speed * dt; moving = true; }

        if moving {
            self.player.frame_timer += dt;
            if self.player.frame_timer > 0.1 { self.player.frame = self.player.frame.wrapping_add(1); self.player.frame_timer = 0.0; }
            self.player.facing_left = is_key_down(KeyCode::A);
        } else {
            self.player.frame = 0;
        }
        self.player.moving = moving;

        // ── Player obstacle collision ──
        if let Some(room) = self.rooms.get(&self.current_room) {
            let p_rect = Rect::new(self.player.x, self.player.y, pw, ph);
            for obs in &room.obstacles {
                if p_rect.overlaps(obs) {
                    if Rect::new(self.player.x, old_py, pw, ph).overlaps(obs) { self.player.x = old_px; }
                    if Rect::new(old_px, self.player.y, pw, ph).overlaps(obs) { self.player.y = old_py; }
                }
            }
        }

        // ── Items & pickups ──
        if let Some(room) = self.rooms.get_mut(&self.current_room) {
            let cx = self.player.x + pw / 2.0;
            let cy = self.player.y + ph / 2.0;
            for item in &mut room.items {
                if !item.collected {
                    let d = ((cx - item.x).powi(2) + (cy - item.y).powi(2)).sqrt();
                    if d < 40.0 {
                        item.collected = true;
                        match item.item_type {
                            ItemType::Health => { self.player.max_hp += 1; self.player.hp = self.player.max_hp; }
                            ItemType::Speed  => { self.player.speed += 50.0; }
                        }
                    }
                }
            }
            room.pickups.retain(|p| {
                let d = ((cx - p.x).powi(2) + (cy - p.y).powi(2)).sqrt();
                if d < 30.0 && self.player.hp < self.player.max_hp { self.player.hp += 1; return false; }
                true
            });
        }

        // ── Room transitions ──
        let cx = self.player.x + pw / 2.0;
        let cy = self.player.y + ph / 2.0;
        let ds = DOOR_HALF;
        let mh = sh / 2.0;
        let mw = sw / 2.0;
        let enemies_cleared = self.rooms.get(&self.current_room).map_or(true, |r| r.enemies.is_empty());
        let mut transitioned = false;
        let (rx, ry) = self.current_room;

        if enemies_cleared {
            if self.player.x >= sw - pw && cy > mh - ds && cy < mh + ds && self.rooms.contains_key(&(rx+1,ry)) {
                self.current_room = (rx+1, ry); self.player.x = WALL + 10.0; self.player.y = mh - ph/2.0; transitioned = true;
            } else if self.player.x <= 0.0 && cy > mh - ds && cy < mh + ds && self.rooms.contains_key(&(rx-1,ry)) {
                self.current_room = (rx-1, ry); self.player.x = sw - pw - WALL - 10.0; self.player.y = mh - ph/2.0; transitioned = true;
            } else if self.player.y >= sh - ph && cx > mw - ds && cx < mw + ds && self.rooms.contains_key(&(rx,ry+1)) {
                self.current_room = (rx, ry+1); self.player.x = mw - pw/2.0; self.player.y = WALL + 10.0; transitioned = true;
            } else if self.player.y <= 0.0 && cx > mw - ds && cx < mw + ds && self.rooms.contains_key(&(rx,ry-1)) {
                self.current_room = (rx, ry-1); self.player.x = mw - pw/2.0; self.player.y = sh - ph - WALL - 10.0; transitioned = true;
            }
        }

        if transitioned { self.projectiles.clear(); self.player.invulnerable_timer = 1.0; }

        // Clamp player inside walls (allow door openings)
        self.player.x = self.player.x.clamp(0.0, sw - pw);
        self.player.y = self.player.y.clamp(0.0, sh - ph);

        // ── Enemy update ──
        if let Some(room) = self.rooms.get_mut(&self.current_room) {
            let mut drops: Vec<Pickup> = Vec::new();

            // Projectile shooting
            let proj_speed = 500.0;
            let px = self.player.x + pw / 2.0;
            let py = self.player.y + ph / 2.0;
            if is_key_pressed(KeyCode::Up)    { self.projectiles.push(Projectile::new(px, py,  0.0, -proj_speed)); }
            if is_key_pressed(KeyCode::Down)  { self.projectiles.push(Projectile::new(px, py,  0.0,  proj_speed)); }
            if is_key_pressed(KeyCode::Left)  { self.projectiles.push(Projectile::new(px, py, -proj_speed, 0.0)); }
            if is_key_pressed(KeyCode::Right) { self.projectiles.push(Projectile::new(px, py,  proj_speed, 0.0)); }

            // Projectile movement & wall/obstacle collision
            for p in self.projectiles.iter_mut() {
                p.update();
                if p.x < WALL || p.x > sw-WALL || p.y < WALL || p.y > sh-WALL { p.alive = false; continue; }
                let pr = Rect::new(p.x - 4.0, p.y - 4.0, 8.0, 8.0);
                for obs in &room.obstacles { if pr.overlaps(obs) { p.alive = false; break; } }
            }

            // Enemy movement & collision
            let n = room.enemies.len();
            for i in 0..n {
                let old_ex = room.enemies[i].x;
                let old_ey = room.enemies[i].y;
                let ew = room.enemies[i].width;
                let eh = room.enemies[i].height;

                room.enemies[i].update(self.player.x, self.player.y);

                if room.enemies[i].state == EnemyState::Chasing {
                    let dx = self.player.x - old_ex;
                    let dy = self.player.y - old_ey;
                    let dist = (dx*dx + dy*dy).sqrt();
                    if dist > 1.0 {
                        let spd = room.enemies[i].speed;
                        room.enemies[i].x = old_ex + (dx / dist) * spd * dt;
                        let er = Rect::new(room.enemies[i].x, old_ey, ew, eh);
                        for obs in &room.obstacles { if er.overlaps(obs) { room.enemies[i].x = old_ex; break; } }

                        room.enemies[i].y = old_ey + (dy / dist) * spd * dt;
                        let er = Rect::new(room.enemies[i].x, room.enemies[i].y, ew, eh);
                        for obs in &room.obstacles { if er.overlaps(obs) { room.enemies[i].y = old_ey; break; } }
                    }
                }

                // Clamp enemies inside walls
                room.enemies[i].x = room.enemies[i].x.clamp(WALL, sw - WALL - ew);
                room.enemies[i].y = room.enemies[i].y.clamp(WALL, sh - WALL - eh);

                // Enemy-enemy separation (push apart)
                for j in (i+1)..n {
                    let dx = room.enemies[j].x - room.enemies[i].x;
                    let dy = room.enemies[j].y - room.enemies[i].y;
                    let min_sep = (ew + room.enemies[j].width) / 2.0;
                    let dist = (dx*dx + dy*dy).sqrt();
                    if dist < min_sep && dist > 0.1 {
                        let push = (min_sep - dist) / 2.0;
                        room.enemies[i].x -= dx / dist * push;
                        room.enemies[i].y -= dy / dist * push;
                        room.enemies[j].x += dx / dist * push;
                        room.enemies[j].y += dy / dist * push;
                    }
                }

                // Player-enemy collision
                let er = Rect::new(room.enemies[i].x, room.enemies[i].y, ew, eh);
                let pr = Rect::new(self.player.x, self.player.y, pw, ph);
                if er.overlaps(&pr) && self.player.invulnerable_timer <= 0.0 {
                    self.player.hp -= 1;
                    self.player.invulnerable_timer = 1.5;
                }
            }

            // Projectile-enemy hit
            for p in self.projectiles.iter_mut() {
                if !p.alive { continue; }
                let pr = Rect::new(p.x - 4.0, p.y - 4.0, 8.0, 8.0);
                for e in room.enemies.iter_mut() {
                    if !e.alive { continue; }
                    if pr.overlaps(&Rect::new(e.x, e.y, e.width, e.height)) {
                        e.take_damage(1);
                        p.alive = false;
                        if !e.alive && gen_range(0, 5) == 0 {
                            drops.push(Pickup { x: e.x + e.width/2.0, y: e.y + e.height/2.0, pickup_type: PickupType::Heart });
                        }
                        break;
                    }
                }
            }

            room.enemies.retain(|e| e.alive);
            room.pickups.append(&mut drops);
        }

        self.projectiles.retain(|p| p.alive && p.x > 0.0 && p.x < sw && p.y > 0.0 && p.y < sh);
    }

    pub fn draw_level(&self, res: &Resources) {
        let sw = screen_width();
        let sh = screen_height();

        if let Some(room) = self.rooms.get(&self.current_room) {
            room.draw(res);

            if room.enemies.is_empty() {
                let ds = DOOR_HALF;
                let (rx, ry) = self.current_room;
                if self.rooms.contains_key(&(rx+1,ry)) { draw_rectangle(sw-20.0, sh/2.0-ds, 20.0, ds*2.0, BLACK); }
                if self.rooms.contains_key(&(rx-1,ry)) { draw_rectangle(0.0,     sh/2.0-ds, 20.0, ds*2.0, BLACK); }
                if self.rooms.contains_key(&(rx,ry+1)) { draw_rectangle(sw/2.0-ds, sh-20.0, ds*2.0, 20.0, BLACK); }
                if self.rooms.contains_key(&(rx,ry-1)) { draw_rectangle(sw/2.0-ds, 0.0,     ds*2.0, 20.0, BLACK); }
            }
        }

        self.player.draw(res.player.as_ref());
        for p in &self.projectiles { p.draw(res.projectile.as_ref()); }

        // HUD - hearts
        let heart_frame = (get_time() * 8.0) as i32 % 3;
        for i in 0..self.player.max_hp {
            if let Some(ref tex) = res.heart {
                let fh = tex.height() / 3.0;
                let color = if i < self.player.hp { WHITE } else { Color::from_rgba(100,100,100,150) };
                draw_texture_ex(tex, 20.0 + i as f32 * 45.0, 20.0, color, DrawTextureParams {
                    dest_size: Some(vec2(40.0, 40.0)),
                    source: Some(Rect::new(0.0, heart_frame as f32 * fh, tex.width(), fh)),
                    ..Default::default()
                });
            } else {
                let color = if i < self.player.hp { RED } else { DARKGRAY };
                draw_rectangle(20.0 + i as f32 * 45.0, 20.0, 32.0, 32.0, color);
            }
        }

        if self.player.hp <= 0 {
            draw_text("GAME OVER", sw/2.0 - 100.0, sh/2.0, 40.0, RED);
            draw_text("Press 'R' to Restart", sw/2.0 - 120.0, sh/2.0 + 50.0, 30.0, WHITE);
        }
    }
}
