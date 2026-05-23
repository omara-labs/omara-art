use crate::effect::Effect;
use rand::Rng;
use ratatui::style::Color;

/// Unstable / Explode + Reassemble effect
/// Glyphs violently fly outward then get yanked back into the logo shape.
pub fn create_unstable_effect(art: &str) -> Effect {
    let mut effect = Effect::from_art(art);
    let mut rng = rand::rng();

    for g in &mut effect.glyphs {
        // Give them a strong random outward kick
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(1.5..4.5);

        g.vx = angle.cos() * speed;
        g.vy = angle.sin() * speed * 0.7 + rng.random_range(-0.5..0.5);

        // Short chaotic phase, then long settle
        g.lifetime = rng.random_range(80.0..160.0);
        g.max_lifetime = g.lifetime;

        // Flashy unstable colors
        g.color = Color::Rgb(255, 120, 80);
        g.target_color = Color::Rgb(180, 60, 220);
    }

    effect
}
