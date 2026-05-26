use std::collections::HashMap;
use crate::room::*;
use crate::player::*;
use crate::projectiles::*;
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::Resources;
use crate::enemy::EnemyState;

pub const WALL: f32 = 48.0;
const DOOR_HALF: f32 = 40.0;

pub struct Level {
    pub rooms: HashMap<(i32,i32), Room>,
    pub current_room: (i32,i32),
    pub player: Player,
    pub projectiles: Vec<Projectile>,
    pub back_to_menu: bool,
}

impl Level {
    pub fn new() -> Self {
        let mut rooms = HashMap::new();
        let target = 12;
        let mut coords = vec![(0i32, 0i32)];
        let mut type_map: HashMap<(i32,i32), RoomType> = HashMap::new();
        type_map.insert((0,0), RoomType::Start);

        while type_map.len() < target {
            let &(x,y) = &coords[gen_range(0, coords.len())];
            let next = match gen_range(0,4) { 0=>(x,y-1), 1=>(x,y+1), 2=>(x-1,y), _=>(x+1,y) };
            if !type_map.contains_key(&next) { type_map.insert(next, RoomType::Normal); coords.push(next); }
        }
        if coords.len() > 3 {
            type_map.insert(coords[gen_range(1, coords.len()-1)], RoomType::Item);
            type_map.insert(*coords.last().unwrap(), RoomType::Boss);
        }
        for (pos, rt) in type_map { rooms.insert(pos, Room::new(rt)); }

        let sw = screen_width(); let sh = screen_height();
        Self { rooms, current_room:(0,0), player: Player::new(sw/2.0, sh/2.0),
               projectiles: Vec::new(), back_to_menu: false }
    }

    pub fn update_level(&mut self) {
        let sw = screen_width(); let sh = screen_height();
        let dt = get_frame_time();

        if is_key_pressed(KeyCode::Escape) { self.back_to_menu = true; return; }

        if self.player.hp <= 0 {
            if is_key_pressed(KeyCode::R) { *self = Level::new(); }
            return;
        }

        self.player.invulnerable_timer = (self.player.invulnerable_timer - dt).max(0.0);
        let pw = self.player.width; let ph = self.player.height;

        // ── Input ──
        let mut vx = 0.0_f32; let mut vy = 0.0_f32;
        if is_key_down(KeyCode::W) { vy -= 1.0; }
        if is_key_down(KeyCode::S) { vy += 1.0; }
        if is_key_down(KeyCode::A) { vx -= 1.0; self.player.facing_left = true; }
        if is_key_down(KeyCode::D) { vx += 1.0; self.player.facing_left = false; }
        let moving = vx != 0.0 || vy != 0.0;
        self.player.moving = moving;

        // Normalise diagonal
        let len = (vx*vx + vy*vy).sqrt();
        if len > 0.0 { vx /= len; vy /= len; }

        if moving {
            self.player.frame_timer += dt;
            if self.player.frame_timer > 0.1 { self.player.frame = self.player.frame.wrapping_add(1); self.player.frame_timer = 0.0; }
        } else { self.player.frame = 0; }

        // Clone obstacles once to avoid borrow conflicts
        let obstacles: Vec<Rect> = self.rooms.get(&self.current_room)
            .map(|r| r.obstacles.clone()).unwrap_or_default();

        // ── Move X then resolve ──
        self.player.x += vx * self.player.speed * dt;
        for obs in &obstacles {
            let r = Rect::new(self.player.x, self.player.y, pw, ph);
            if r.overlaps(obs) {
                if vx > 0.0 { self.player.x = obs.x - pw; }
                else        { self.player.x = obs.x + obs.w; }
            }
        }

        // ── Move Y then resolve ──
        self.player.y += vy * self.player.speed * dt;
        for obs in &obstacles {
            let r = Rect::new(self.player.x, self.player.y, pw, ph);
            if r.overlaps(obs) {
                if vy > 0.0 { self.player.y = obs.y - ph; }
                else        { self.player.y = obs.y + obs.h; }
            }
        }

        // ── Items & pickups ──
        if let Some(room) = self.rooms.get_mut(&self.current_room) {
            let cx = self.player.x + pw/2.0; let cy = self.player.y + ph/2.0;
            for item in &mut room.items {
                if !item.collected && ((cx-item.x).powi(2)+(cy-item.y).powi(2)).sqrt() < 40.0 {
                    item.collected = true;
                    match item.item_type { ItemType::Health => { self.player.max_hp+=1; self.player.hp=self.player.max_hp; } ItemType::Speed => self.player.speed += 50.0 }
                }
            }
            room.pickups.retain(|p| {
                if ((cx-p.x).powi(2)+(cy-p.y).powi(2)).sqrt() < 30.0 && self.player.hp < self.player.max_hp { self.player.hp+=1; return false; }
                true
            });
        }

        // ── Room transitions ──
        let cx = self.player.x + pw/2.0; let cy = self.player.y + ph/2.0;
        let mh = sh/2.0; let mw = sw/2.0; let ds = DOOR_HALF;
        let clear = self.rooms.get(&self.current_room).map_or(true, |r| r.enemies.is_empty());
        let mut trans = false;
        let (rx,ry) = self.current_room;

        if clear {
            if      self.player.x >= sw-pw   && cy>mh-ds && cy<mh+ds && self.rooms.contains_key(&(rx+1,ry)) { self.current_room=(rx+1,ry); self.player.x=WALL+10.0; self.player.y=mh-ph/2.0; trans=true; }
            else if self.player.x <= 0.0      && cy>mh-ds && cy<mh+ds && self.rooms.contains_key(&(rx-1,ry)) { self.current_room=(rx-1,ry); self.player.x=sw-pw-WALL-10.0; self.player.y=mh-ph/2.0; trans=true; }
            else if self.player.y >= sh-ph    && cx>mw-ds && cx<mw+ds && self.rooms.contains_key(&(rx,ry+1)) { self.current_room=(rx,ry+1); self.player.x=mw-pw/2.0; self.player.y=WALL+10.0; trans=true; }
            else if self.player.y <= 0.0      && cx>mw-ds && cx<mw+ds && self.rooms.contains_key(&(rx,ry-1)) { self.current_room=(rx,ry-1); self.player.x=mw-pw/2.0; self.player.y=sh-ph-WALL-10.0; trans=true; }
        }
        if trans { self.projectiles.clear(); self.player.invulnerable_timer = 1.0; }

        self.player.x = self.player.x.clamp(0.0, sw-pw);
        self.player.y = self.player.y.clamp(0.0, sh-ph);

        // ── Enemies & projectiles ──
        if let Some(room) = self.rooms.get_mut(&self.current_room) {
            let mut drops: Vec<Pickup> = Vec::new();

            // Shoot
            let spd = 500.0;
            let px = self.player.x+pw/2.0; let py = self.player.y+ph/2.0;
            if is_key_pressed(KeyCode::Up)    { self.projectiles.push(Projectile::new(px,py, 0.0,-spd)); }
            if is_key_pressed(KeyCode::Down)  { self.projectiles.push(Projectile::new(px,py, 0.0, spd)); }
            if is_key_pressed(KeyCode::Left)  { self.projectiles.push(Projectile::new(px,py,-spd, 0.0)); }
            if is_key_pressed(KeyCode::Right) { self.projectiles.push(Projectile::new(px,py, spd, 0.0)); }

            // Move projectiles
            for p in self.projectiles.iter_mut() {
                p.update();
                if p.x<WALL||p.x>sw-WALL||p.y<WALL||p.y>sh-WALL { p.alive=false; continue; }
                let pr = Rect::new(p.x-4.0,p.y-4.0,8.0,8.0);
                for obs in &room.obstacles { if pr.overlaps(obs) { p.alive=false; break; } }
            }

            // Move enemies
            let n = room.enemies.len();
            for i in 0..n {
                let (oex, oey) = (room.enemies[i].x, room.enemies[i].y);
                let (ew, eh)   = (room.enemies[i].width, room.enemies[i].height);

                room.enemies[i].update(self.player.x, self.player.y);

                if room.enemies[i].state == EnemyState::Chasing {
                    let dx = self.player.x - oex; let dy = self.player.y - oey;
                    let dist = (dx*dx+dy*dy).sqrt();
                    if dist > 1.0 {
                        let s = room.enemies[i].speed;
                        // X
                        room.enemies[i].x = oex + (dx/dist)*s*dt;
                        for obs in &room.obstacles {
                            if Rect::new(room.enemies[i].x, oey, ew, eh).overlaps(obs) {
                                room.enemies[i].x = if dx>0.0 { obs.x-ew } else { obs.x+obs.w };
                            }
                        }
                        // Y
                        let ex = room.enemies[i].x;
                        room.enemies[i].y = oey + (dy/dist)*s*dt;
                        for obs in &room.obstacles {
                            if Rect::new(ex, room.enemies[i].y, ew, eh).overlaps(obs) {
                                room.enemies[i].y = if dy>0.0 { obs.y-eh } else { obs.y+obs.h };
                            }
                        }
                    }
                }

                // Wall clamp
                room.enemies[i].x = room.enemies[i].x.clamp(WALL, sw-WALL-ew);
                room.enemies[i].y = room.enemies[i].y.clamp(WALL, sh-WALL-eh);

                // Enemy-enemy separation
                for j in (i+1)..n {
                    let dx = room.enemies[j].x - room.enemies[i].x;
                    let dy = room.enemies[j].y - room.enemies[i].y;
                    let sep = (room.enemies[i].width + room.enemies[j].width) / 2.0;
                    let d   = (dx*dx+dy*dy).sqrt();
                    if d < sep && d > 0.1 {
                        let push = (sep-d)/2.0;
                        room.enemies[i].x -= dx/d*push; room.enemies[i].y -= dy/d*push;
                        room.enemies[j].x += dx/d*push; room.enemies[j].y += dy/d*push;
                    }
                }

                // Player damage
                if Rect::new(room.enemies[i].x,room.enemies[i].y,ew,eh)
                    .overlaps(&Rect::new(self.player.x,self.player.y,pw,ph))
                    && self.player.invulnerable_timer <= 0.0
                {
                    self.player.hp -= 1; self.player.invulnerable_timer = 1.5;
                }
            }

            // Projectile-enemy
            for p in self.projectiles.iter_mut() {
                if !p.alive { continue; }
                let pr = Rect::new(p.x-4.0,p.y-4.0,8.0,8.0);
                for e in room.enemies.iter_mut() {
                    if !e.alive { continue; }
                    if pr.overlaps(&Rect::new(e.x,e.y,e.width,e.height)) {
                        e.take_damage(1); p.alive=false;
                        if !e.alive && gen_range(0,5)==0 {
                            drops.push(Pickup { x:e.x+e.width/2.0, y:e.y+e.height/2.0, pickup_type:PickupType::Heart });
                        }
                        break;
                    }
                }
            }

            room.enemies.retain(|e| e.alive);
            room.pickups.append(&mut drops);
        }
        self.projectiles.retain(|p| p.alive && p.x>0.0 && p.x<sw && p.y>0.0 && p.y<sh);
    }

    pub fn draw_level(&self, res: &Resources) {
        let sw = screen_width(); let sh = screen_height();

        if let Some(room) = self.rooms.get(&self.current_room) {
            room.draw(res);
            if room.enemies.is_empty() {
                let ds = DOOR_HALF; let (rx,ry) = self.current_room;
                if self.rooms.contains_key(&(rx+1,ry)) { draw_rectangle(sw-20.0,sh/2.0-ds,20.0,ds*2.0,BLACK); }
                if self.rooms.contains_key(&(rx-1,ry)) { draw_rectangle(0.0,    sh/2.0-ds,20.0,ds*2.0,BLACK); }
                if self.rooms.contains_key(&(rx,ry+1)) { draw_rectangle(sw/2.0-ds,sh-20.0,ds*2.0,20.0,BLACK); }
                if self.rooms.contains_key(&(rx,ry-1)) { draw_rectangle(sw/2.0-ds,0.0,    ds*2.0,20.0,BLACK); }
            }
        }

        self.player.draw(res.player.as_ref());
        for p in &self.projectiles { p.draw(res.projectile.as_ref()); }

        // ── Hearts HUD ──
        let hf = (get_time()*8.0) as i32 % 3;
        for i in 0..self.player.max_hp {
            let color = if i < self.player.hp { WHITE } else { Color::from_rgba(100,100,100,150) };
            if let Some(ref tex) = res.heart {
                let fh = tex.height()/3.0;
                draw_texture_ex(tex, 20.0+i as f32*45.0, 20.0, color, DrawTextureParams {
                    dest_size: Some(vec2(40.0,40.0)), source: Some(Rect::new(0.0,hf as f32*fh,tex.width(),fh)), ..Default::default()
                });
            } else { draw_rectangle(20.0+i as f32*45.0, 20.0, 32.0, 32.0, color); }
        }

        // ── Minimap ──
        self.draw_minimap();

        // ── ESC hint ──
        draw_text("ESC: Menu", sw - 110.0, sh - 14.0, 18.0, Color::from_rgba(140,130,160,200));

        if self.player.hp <= 0 {
            draw_rectangle(0.0,0.0,sw,sh,Color::from_rgba(0,0,0,160));
            let t = "GAME OVER";
            let tw = measure_text(t,None,56,1.0).width;
            draw_text(t, sw/2.0-tw/2.0+3.0, sh/2.0+3.0, 56.0, BLACK);
            draw_text(t, sw/2.0-tw/2.0, sh/2.0, 56.0, RED);
            let s = "Press R to Restart  |  ESC for Menu";
            let sw2 = measure_text(s,None,22,1.0).width;
            draw_text(s, sw/2.0-sw2/2.0, sh/2.0+50.0, 22.0, WHITE);
        }
    }

    fn draw_minimap(&self) {
        let sw = screen_width(); let sh = screen_height();
        let cell  = 14.0_f32;
        let gap   = 3.0_f32;
        let step  = cell + gap;
        let pad   = 10.0_f32;

        // Find bounds
        let (mut mnx,mut mxx,mut mny,mut mxy) = (i32::MAX,i32::MIN,i32::MAX,i32::MIN);
        for &(x,y) in self.rooms.keys() {
            mnx=mnx.min(x); mxx=mxx.max(x); mny=mny.min(y); mxy=mxy.max(y);
        }
        let cols = (mxx-mnx+1) as f32;
        let rows = (mxy-mny+1) as f32;
        let map_w = cols*step - gap + pad*2.0;
        let map_h = rows*step - gap + pad*2.0;

        // Panel in top-right
        let ox = sw - map_w - 12.0;
        let oy = 12.0;
        draw_rectangle(ox-2.0, oy-2.0, map_w+4.0, map_h+4.0, Color::from_rgba(0,0,0,180));
        draw_rectangle_lines(ox-2.0, oy-2.0, map_w+4.0, map_h+4.0, 1.5, Color::from_rgba(120,100,140,200));

        for (&(rx,ry), room) in &self.rooms {
            let cx = ox + pad + (rx-mnx) as f32 * step;
            let cy = oy + pad + (ry-mny) as f32 * step;
            let is_current = (rx,ry) == self.current_room;

            // Draw corridor connectors
            let dirs = [(rx+1,ry,1.0_f32,0.0_f32),(rx,ry+1,0.0_f32,1.0_f32)];
            for &(nx,ny,ddx,ddy) in &dirs {
                if self.rooms.contains_key(&(nx,ny)) {
                    let lx = cx + cell/2.0 + ddx*(cell/2.0);
                    let ly = cy + cell/2.0 + ddy*(cell/2.0);
                    draw_line(cx+cell/2.0, cy+cell/2.0, lx+ddx*(gap+cell/2.0), ly+ddy*(gap+cell/2.0),
                              2.0, Color::from_rgba(90,80,110,200));
                }
            }

            // Room square
            let color = match room.room_type {
                crate::room::RoomType::Start  => Color::from_rgba(60,160,80,230),
                crate::room::RoomType::Boss   => Color::from_rgba(180,40,40,230),
                crate::room::RoomType::Item   => Color::from_rgba(60,120,200,230),
                crate::room::RoomType::Normal => Color::from_rgba(80,70,100,230),
            };
            draw_rectangle(cx, cy, cell, cell, color);
            if is_current {
                draw_rectangle_lines(cx, cy, cell, cell, 2.0, WHITE);
                // Player dot
                draw_circle(cx+cell/2.0, cy+cell/2.0, 3.0, WHITE);
            }
            if room.enemies.is_empty() { /* cleared - slightly brighter already */ }
        }

        // Legend
        let ly = oy + map_h + 4.0;
        let legends = [("S", Color::from_rgba(60,160,80,230)),
                       ("B", Color::from_rgba(180,40,40,230)),
                       ("I", Color::from_rgba(60,120,200,230))];
        for (i,(label,col)) in legends.iter().enumerate() {
            let lx = ox + i as f32 * 26.0;
            draw_rectangle(lx, ly, 10.0, 10.0, *col);
            draw_text(label, lx+13.0, ly+10.0, 11.0, Color::from_rgba(180,170,200,220));
        }
    }
}
