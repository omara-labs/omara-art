use crate::effect::Effect;
use rand::Rng;
use ratatui::style::Color;

/// Pour / Cascade effect
/// Glyphs fall from the top and "stack" or settle into the final shape from the bottom up.
pub fn create_pour_effect(art: &str) -> Effect {
    let mut effect = Effect::from_art(art);
    let mut rng = rand::rng();

    for g in &mut effect.glyphs {
        // Start them high up, with some horizontal spread
        g.y = rng.random_range(-25.0..-3.0);
        g.x += rng.random_range(-2.0..2.0);

        g.vx = rng.random_range(-0.3..0.3);
        g.vy = rng.random_range(0.8..1.5);

        g.lifetime = rng.random_range(200.0..450.0);
        g.max_lifetime = g.lifetime;

        // Cool blue/cyan pour colors
        g.color = Color::Rgb(40, 120, 200);
        g.target_color = Color::Rgb(120, 200, 255);
    }

    effect
}
