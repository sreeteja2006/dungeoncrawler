use macroquad::prelude::*;
use crate::level::*;

mod room; mod player; mod update; mod projectiles; mod enemy; mod level;

pub struct EnemyTextures { pub idle: Option<Texture2D>, pub run: Option<Texture2D> }

pub struct Resources {
    pub player: Option<Texture2D>,
    pub skeleton1: EnemyTextures, pub skeleton2: EnemyTextures, pub vampire: EnemyTextures,
    pub projectile: Option<Texture2D>, pub heart: Option<Texture2D>,
    pub speed_item: Option<Texture2D>, pub health_item: Option<Texture2D>,
    pub obstacle: Option<Texture2D>,
}

impl Resources {
    pub async fn load() -> Self {
        async fn tex(path: &str) -> Option<Texture2D> {
            let t = load_texture(path).await.ok();
            if let Some(ref t) = t { t.set_filter(FilterMode::Nearest); }
            t
        }
        Self {
            player:      tex("assets/player.png").await,
            skeleton1:   EnemyTextures { idle: tex("assets/skeleton1_idle.png").await, run: tex("assets/skeleton1_run.png").await },
            skeleton2:   EnemyTextures { idle: tex("assets/skeleton2_idle.png").await, run: tex("assets/skeleton2_run.png").await },
            vampire:     EnemyTextures { idle: tex("assets/vampire_idle.png").await,   run: tex("assets/vampire_run.png").await   },
            projectile:  tex("assets/projectile.png").await,
            heart:       tex("assets/heart.png").await,
            speed_item:  tex("assets/flask_speed.png").await,
            health_item: tex("assets/flask_health.png").await,
            obstacle:    tex("assets/tileset.png").await,
        }
    }
}

#[macroquad::main("Dungeon Crawler")]
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
