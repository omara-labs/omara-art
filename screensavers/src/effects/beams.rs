use crate::effect::Effect;
use rand::Rng;
use ratatui::style::Color;

pub fn create_beams_effect(art: &str) -> Effect {
    let mut effect = Effect::from_art(art);
    let mut rng = rand::rng();

    for g in &mut effect.glyphs {
        g.color = Color::Rgb(60, 0, 90);
        g.target_color = Color::Rgb(180, 0, 255);
        g.vx = rng.random_range(-0.15..0.15);
        g.vy = rng.random_range(-0.08..0.08);
        g.lifetime = 9999.0;
        g.max_lifetime = 9999.0;
    }

    effect
}

pub fn beam_highlight(base: Color, beam_x: f32, glyph_x: f32) -> Color {
    let dist = (beam_x - glyph_x).abs();
    if dist < 1.5 {
        Color::Rgb(255, 220, 255)
    } else if dist < 3.5 {
        Color::Rgb(220, 80, 255)
    } else {
        base
    }
}
