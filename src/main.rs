use macroquad::prelude::*;
use crate::level::*;

mod room;
mod player;
mod update;
mod projectiles;
mod enemy;
mod level;

pub struct EnemyTextures {
    pub idle: Option<Texture2D>,
    pub run: Option<Texture2D>,
}

pub struct Resources {
    pub player: Option<Texture2D>,
    pub skeleton1: EnemyTextures,
    pub skeleton2: EnemyTextures,
    pub vampire: EnemyTextures,
    pub projectile: Option<Texture2D>,
    pub heart: Option<Texture2D>,
    pub speed_item: Option<Texture2D>,
    pub health_item: Option<Texture2D>,
    pub obstacle: Option<Texture2D>,
}

impl Resources {
    pub async fn load() -> Self {
        let base = "assets/2D Pixel Dungeon Asset Pack v2.0/2D Pixel Dungeon Asset Pack";
        let enem = "assets/Enemy_Animations_Set";
        Self {
            player:      load_texture(&format!("{}/character and tileset/Dungeon_Character.png", base)).await.ok(),
            skeleton1:   EnemyTextures { idle: load_texture(&format!("{}/enemies-skeleton1_idle.png", enem)).await.ok(),     run: load_texture(&format!("{}/enemies-skeleton1_movement.png", enem)).await.ok() },
            skeleton2:   EnemyTextures { idle: load_texture(&format!("{}/enemies-skeleton2_idle.png", enem)).await.ok(),     run: load_texture(&format!("{}/enemies-skeleton2_movemen.png", enem)).await.ok() },
            vampire:     EnemyTextures { idle: load_texture(&format!("{}/enemies-vampire_idle.png", enem)).await.ok(),       run: load_texture(&format!("{}/enemies-vampire_movement.png", enem)).await.ok() },
            projectile:  load_texture("assets/projectile.png").await.ok(),
            heart:       load_texture("assets/heart.png").await.ok(),
            speed_item:  load_texture(&format!("{}/items and trap_animation/flasks/flasks_2_1.png", base)).await.ok(),
            health_item: load_texture(&format!("{}/items and trap_animation/flasks/flasks_1_1.png", base)).await.ok(),
            obstacle:    load_texture(&format!("{}/character and tileset/Dungeon_Tileset.png", base)).await.ok(),
        }
    }
}

#[macroquad::main("Isaac Clone")]
async fn main() {
    let resources = Resources::load().await;
    let mut level = Level::new();
    loop {
        clear_background(BLACK);
        level.update_level();
        level.draw_level(&resources);
        next_frame().await;
    }
}
