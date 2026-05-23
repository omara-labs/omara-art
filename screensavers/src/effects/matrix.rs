use crate::effect::Effect;
use rand::Rng;
use ratatui::style::Color;

pub fn create_matrix_effect(art: &str) -> Effect {
    let mut effect = Effect::from_art(art);
    let mut rng = rand::rng();

    for g in &mut effect.glyphs {
        let home_x = g.home_x;
        g.y = rng.random_range(-40.0..-10.0);
        g.x = home_x + rng.random_range(-5.0..5.0);
        g.vx = rng.random_range(-0.25..0.25);
        g.vy = rng.random_range(0.9..1.8);
        g.lifetime = rng.random_range(220.0..620.0);
        g.max_lifetime = g.lifetime;
        g.color = Color::Rgb(0, 35, 0);
        g.target_color = Color::Rgb(0, 210, 90);
    }

    effect
}
