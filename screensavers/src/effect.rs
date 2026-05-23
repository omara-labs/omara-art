use rand::Rng;
use ratatui::style::Color;

#[derive(Clone, Debug)]
pub struct Glyph {
    pub ch: char,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub home_x: f32,
    pub home_y: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: Color,
    pub target_color: Color,
}

impl Glyph {
    pub fn new(ch: char, x: f32, y: f32) -> Self {
        let mut rng = rand::rng();
        Self {
            ch,
            x,
            y,
            vx: rng.random_range(-0.8..0.8),
            vy: rng.random_range(-0.3..0.3),
            home_x: x,
            home_y: y,
            lifetime: rng.random_range(20.0..80.0),
            max_lifetime: 80.0,
            color: Color::Rgb(180, 0, 255),
            target_color: Color::Rgb(120, 0, 200),
        }
    }

    pub fn update(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        self.vy += 0.01;
        self.vx *= 0.992;
        self.vy *= 0.995;

        let dx = self.home_x - self.x;
        let dy = self.home_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 4.0 {
            self.vx += dx * 0.08;
            self.vy += dy * 0.08;
            self.vx *= 0.6;
            self.vy *= 0.6;

            if dist < 0.6 {
                self.x = self.home_x;
                self.y = self.home_y;
                self.vx = 0.0;
                self.vy = 0.0;
            }
        }

        self.lifetime -= 1.0;

        if let (Color::Rgb(r, g, b), Color::Rgb(tr, tg, tb)) = (self.color, self.target_color) {
            let mix = 0.035;
            self.color = Color::Rgb(
                (r as f32 * (1.0 - mix) + tr as f32 * mix) as u8,
                (g as f32 * (1.0 - mix) + tg as f32 * mix) as u8,
                (b as f32 * (1.0 - mix) + tb as f32 * mix) as u8,
            );
        }
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }
}

#[derive(Clone, Debug)]
pub struct Effect {
    pub glyphs: Vec<Glyph>,
    pub width: u16,
    pub height: u16,
}

impl Effect {
    pub fn from_art(art: &str) -> Self {
        let lines: Vec<&str> = art.lines().collect();
        let height = lines.len() as u16;
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;

        let mut glyphs = Vec::new();
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if !ch.is_whitespace() {
                    glyphs.push(Glyph::new(ch, x as f32, y as f32));
                }
            }
        }

        Self { glyphs, width, height }
    }

    pub fn tick(&mut self) {
        for g in &mut self.glyphs {
            g.update();
        }
        self.glyphs.retain(|g| g.is_alive());
    }
}
