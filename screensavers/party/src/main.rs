// omara-party - Rave Party screensaver.
// Self-contained, resize-friendly.

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::Paragraph,
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use rand::Rng;

pub const DEFAULT_ART: &str = include_str!("../../../assets/brand/omara.txt");

pub fn load_branding() -> String {
    if let Some(config_dir) = dirs::config_dir() {
        let user_path = config_dir.join("omara/branding/screensaver.txt");
        if user_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&user_path) {
                if !content.trim().is_empty() {
                    return content;
                }
            }
        }
    }
    DEFAULT_ART.to_string()
}

struct Confetti {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    color: Color,
    ch: char,
    lifetime: f32,
    max_lifetime: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_party(&mut terminal);

    execute!(
        terminal.backend_mut(),
        crossterm::cursor::Show,
        LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal::disable_raw_mode()?;

    if let Err(err) = result {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_party<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut rng = rand::rng();
    let start_time = Instant::now();
    let mut last_frame = Instant::now();

    // Confetti particles
    let confetti_chars = ['*', '+', 'o', 'x', '•'];
    let neon_colors = [
        Color::Rgb(255, 0, 128),  // Neon Pink
        Color::Rgb(0, 255, 255),  // Neon Cyan
        Color::Rgb(255, 255, 0),  // Neon Yellow
        Color::Rgb(50, 255, 50),  // Neon Green
        Color::Rgb(180, 0, 255),  // Neon Purple
        Color::Rgb(255, 127, 0),  // Neon Orange
    ];

    let mut confetti_list: Vec<Confetti> = (0..60)
        .map(|_| {
            let max_lt = rng.random::<f32>() * 1.5 + 0.5;
            Confetti {
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                color: neon_colors[rng.random_range(0..neon_colors.len())],
                ch: confetti_chars[rng.random_range(0..confetti_chars.len())],
                lifetime: 0.0, // Force instant respawn
                max_lifetime: max_lt,
            }
        })
        .collect();

    loop {
        if event::poll(Duration::from_millis(8))? {
            if matches!(event::read()?, Event::Key(_) | Event::Mouse(_)) {
                break Ok(());
            }
        }

        let now = Instant::now();
        let delta = now.duration_since(last_frame).as_secs_f32().min(0.1);
        last_frame = now;
        let elapsed = start_time.elapsed().as_secs_f32();

        let size = terminal.size()?;
        let width = size.width;
        let height = size.height;

        let disco_x = width / 2;
        let disco_y = 4u16;

        // Update confetti particles
        for c in &mut confetti_list {
            c.lifetime -= delta;
            if c.lifetime <= 0.0 {
                // Respawn particle either from disco ball or logo center
                let spawn_from_disco = rng.random_bool(0.4);
                if spawn_from_disco {
                    c.x = disco_x as f32;
                    c.y = disco_y as f32;
                    // Shoot downwards/outwards
                    let angle = rng.random::<f32>() * std::f32::consts::PI; // 0 to PI
                    let speed = rng.random::<f32>() * 18.0 + 8.0;
                    c.vx = angle.cos() * speed / 0.55;
                    c.vy = angle.sin() * speed;
                } else {
                    c.x = (width / 2) as f32 + (rng.random::<f32>() * 20.0 - 10.0);
                    c.y = (height / 2) as f32 + (rng.random::<f32>() * 6.0 - 3.0);
                    // Shoot in any direction
                    let angle = rng.random::<f32>() * std::f32::consts::TAU;
                    let speed = rng.random::<f32>() * 12.0 + 6.0;
                    c.vx = angle.cos() * speed / 0.55;
                    c.vy = angle.sin() * speed;
                }
                c.color = neon_colors[rng.random_range(0..neon_colors.len())];
                c.ch = confetti_chars[rng.random_range(0..confetti_chars.len())];
                c.max_lifetime = rng.random::<f32>() * 1.2 + 0.4;
                c.lifetime = c.max_lifetime;
            } else {
                c.x += c.vx * delta;
                c.y += c.vy * delta;
                c.vy += 4.5 * delta; // Gravity pull
            }
        }

        // Draw Everything
        terminal.draw(|f| {
            let area = f.area();
            if area.width == 0 || area.height == 0 { return; }

            // 1. Strobe Beat Background Color
            let beat_period = 0.48; // ~125 BPM
            let beat_phase = elapsed % beat_period;
            let mut bg_color = Color::Black;
            if beat_phase < 0.08 {
                let beat_index = ((elapsed / beat_period) as usize) % 4;
                bg_color = match beat_index {
                    0 => Color::Rgb(25, 0, 50),   // Dim Purple
                    1 => Color::Rgb(0, 15, 45),   // Dim Blue
                    2 => Color::Rgb(40, 0, 20),   // Dim Pink/Red
                    _ => Color::Rgb(0, 25, 15),   // Dim Green
                };
            }
            f.render_widget(
                ratatui::widgets::Block::default().style(Style::default().bg(bg_color)),
                area,
            );

            // 2. Render Disco Ball light rays sweeping
            let num_rays = 8;
            let base_angle = elapsed * 0.7;
            let max_ray_len = 35.0;

            for k in 0..num_rays {
                let angle = base_angle + (k as f32) * (std::f32::consts::TAU / num_rays as f32);
                let ray_color_base = neon_colors[k % neon_colors.len()];

                // Draw ray segment by segment outward
                for d in 4..30 {
                    let rx = disco_x as f32 + d as f32 * angle.cos() / 0.55;
                    let ry = disco_y as f32 + d as f32 * angle.sin();

                    if rx >= 0.0 && rx < width as f32 && ry >= 0.0 && ry < height as f32 {
                        let x = rx as u16;
                        let y = ry as u16;

                        // Ray intensity fades out with distance
                        let intensity = 1.0 - (d as f32 / max_ray_len).min(1.0);
                        if intensity > 0.05 {
                            let ch = if d < 10 { '*' } else if d < 20 { '+' } else { '.' };
                            let color = if let Color::Rgb(r, g, b) = ray_color_base {
                                Color::Rgb(
                                    (r as f32 * intensity) as u8,
                                    (g as f32 * intensity) as u8,
                                    (b as f32 * intensity) as u8,
                                )
                            } else {
                                ray_color_base
                            };
                            f.render_widget(
                                Paragraph::new(ch.to_string()).style(Style::default().fg(color)),
                                Rect::new(x, y, 1, 1),
                            );
                        }
                    }
                }
            }

            // 3. Render Shimmering Disco Ball itself
            let radius_y = 3;
            let radius_x = 5;
            let disc_chars = ['░', '▒', '▓', '█'];
            let rot_offset = (elapsed * 5.0) as i32;

            for dy in -radius_y..=radius_y {
                let width_at_y = (radius_x as f32 * (1.0 - (dy as f32 / radius_y as f32).powi(2)).max(0.0).sqrt()).round() as i16;
                for dx in -width_at_y..=width_at_y {
                    let sx = (disco_x as i16 + dx) as u16;
                    let sy = (disco_y as i16 + dy) as u16;
                    if sx < width && sy < height {
                        // Rotation pattern lookup
                        let pattern_idx = ((dx + dy + rot_offset as i16).abs() % 4) as usize;
                        let ch = disc_chars[pattern_idx];
                        // Highlight near center
                        let dist_center = (dx as f32).powi(2) + (dy as f32 * 1.8).powi(2);
                        let bright = if dist_center < 3.0 {
                            Color::Rgb(255, 255, 255) // White hotspot
                        } else if dist_center < 9.0 {
                            Color::Rgb(200, 220, 255) // Bright Silver
                        } else {
                            Color::Rgb(100, 110, 130) // Silver/Gray
                        };
                        f.render_widget(
                            Paragraph::new(ch.to_string()).style(Style::default().fg(bright)),
                            Rect::new(sx, sy, 1, 1),
                        );
                    }
                }
            }

            // Draw disco string / hanger
            for sy in 0..disco_y.saturating_sub(radius_y as u16) {
                f.render_widget(
                    Paragraph::new("│").style(Style::default().fg(Color::Rgb(100, 100, 100))),
                    Rect::new(disco_x, sy, 1, 1),
                );
            }

            // 4. Render Bouncing Equalizer Bars (bottom)
            let max_bar_h = (height / 5).clamp(4, 10);
            let bar_spacing = 3;
            for x in (0..width).step_by(bar_spacing as usize) {
                // Determine heights with multiple overlapping sine waves + noise
                let freq1 = 2.4;
                let freq2 = 5.6;
                let phase = x as f32 * 0.15;
                let sin_v1 = (elapsed * freq1 + phase).sin().abs();
                let sin_v2 = (elapsed * freq2 - phase * 0.5).cos().abs() * 0.4;
                let rand_noise = rng.random::<f32>() * 0.25;
                let val = (sin_v1 + sin_v2 + rand_noise).clamp(0.0, 1.0);
                let bar_h = (val * max_bar_h as f32).round() as u16;

                for dy in 0..bar_h {
                    let y = height.saturating_sub(1).saturating_sub(dy);
                    if y >= height { continue; }

                    let pct = dy as f32 / max_bar_h as f32;
                    let color = if pct < 0.4 {
                        Color::Rgb(0, 255, 128) // Bottom green-cyan
                    } else if pct < 0.75 {
                        Color::Rgb(255, 200, 0) // Middle yellow-orange
                    } else {
                        Color::Rgb(255, 0, 100) // Top red-pink
                    };

                    // Draw a 2-character wide bar
                    for dx in 0..2 {
                        let bx = x + dx;
                        if bx < width {
                            f.render_widget(
                                Paragraph::new("█").style(Style::default().fg(color)),
                                Rect::new(bx, y, 1, 1),
                            );
                        }
                    }
                }
            }

            // 5. Exploding Neon Confetti Particles
            for c in &confetti_list {
                let px = c.x as u16;
                let py = c.y as u16;
                if px < width && py < height {
                    let life_pct = c.lifetime / c.max_lifetime;
                    let col = if let Color::Rgb(r, g, b) = c.color {
                        Color::Rgb(
                            (r as f32 * life_pct) as u8,
                            (g as f32 * life_pct) as u8,
                            (b as f32 * life_pct) as u8,
                        )
                    } else {
                        c.color
                    };
                    f.render_widget(
                        Paragraph::new(c.ch.to_string()).style(Style::default().fg(col)),
                        Rect::new(px, py, 1, 1),
                    );
                }
            }

            // 6. Centered Rave Rainbow Logo
            let logo_x = (width.saturating_sub(logo_width) / 2) as i16;
            let logo_y = (height.saturating_sub(logo_height) / 2) as i16;

            if logo_x >= 0 && logo_y >= 0 {
                // Color cycle based on sine wave
                let r = ((elapsed * 4.5).sin() * 127.0 + 128.0) as u8;
                let g = ((elapsed * 4.5 + 2.0).sin() * 127.0 + 128.0) as u8;
                let b = ((elapsed * 4.5 + 4.0).sin() * 127.0 + 128.0) as u8;
                let logo_color = Color::Rgb(r, g, b);

                // Draw a black backdrop for the logo so it stands out
                let bg_x = logo_x.saturating_sub(2) as u16;
                let bg_y = logo_y.saturating_sub(1) as u16;
                let bg_w = (logo_width + 4).min(width.saturating_sub(bg_x));
                let bg_h = (logo_height + 2).min(height.saturating_sub(bg_y));
                f.render_widget(
                    ratatui::widgets::Block::default().style(Style::default().bg(Color::Black)),
                    Rect::new(bg_x, bg_y, bg_w, bg_h),
                );

                for (i, line) in logo_lines.iter().enumerate() {
                    let ly = logo_y as u16 + i as u16;
                    if ly < height {
                        let para = Paragraph::new(*line).style(Style::default().fg(logo_color).bold());
                        f.render_widget(para, Rect::new(logo_x as u16, ly, logo_width, 1));
                    }
                }
            }
        })?;

        std::thread::sleep(Duration::from_millis(15));
    }
}
