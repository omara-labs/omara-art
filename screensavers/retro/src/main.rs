// omara-retro - 80s Synthwave / Outrun style screensaver.
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

struct Star {
    x: f32,
    y: f32,
    base_color: Color,
    phase: f32,
}

fn sun_color(pct: f32) -> Color {
    if pct < 0.4 {
        // Yellow to Orange
        let t = pct / 0.4;
        let r = 255;
        let g = (255.0 * (1.0 - t) + 140.0 * t) as u8;
        let b = 0;
        Color::Rgb(r, g, b)
    } else if pct < 0.75 {
        // Orange to Magenta/Hot Pink
        let t = (pct - 0.4) / 0.35;
        let r = 255;
        let g = (140.0 * (1.0 - t)) as u8;
        let b = (150.0 * t) as u8;
        Color::Rgb(r, g, b)
    } else {
        // Deep Purple/Magenta
        let t = (pct - 0.75) / 0.25;
        let r = (255.0 * (1.0 - t) + 120.0 * t) as u8;
        let g = 0;
        let b = (150.0 * (1.0 - t) + 200.0 * t) as u8;
        Color::Rgb(r, g, b)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_retro(&mut terminal);

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

fn run_retro<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut rng = rand::rng();
    let start_time = Instant::now();

    // Create background starfield for upper sky
    let stars: Vec<Star> = (0..50)
        .map(|_| Star {
            x: rng.random::<f32>(),
            y: rng.random::<f32>() * 0.5, // Only in top half
            base_color: if rng.random_bool(0.5) {
                Color::Rgb(0, 200, 255) // Cyan-ish stars
            } else {
                Color::Rgb(255, 0, 180) // Pink-ish stars
            },
            phase: rng.random::<f32>() * std::f32::consts::TAU,
        })
        .collect();

    let logo_colors = [
        Color::Rgb(255, 255, 255), // Top row highlight
        Color::Rgb(0, 255, 255),   // Cyan
        Color::Rgb(0, 180, 255),   // Blue
        Color::Rgb(160, 0, 255),   // Violet
        Color::Rgb(255, 0, 180),   // Hot Pink
        Color::Rgb(255, 80, 0),    // Neon Orange
    ];

    loop {
        if event::poll(Duration::from_millis(16))? {
            if matches!(event::read()?, Event::Key(_) | Event::Mouse(_)) {
                break Ok(());
            }
        }

        let elapsed = start_time.elapsed().as_secs_f32();

        terminal.draw(|f| {
            let area = f.area();
            let width = area.width;
            let height = area.height;

            // 1. Solid black background
            f.render_widget(
                ratatui::widgets::Block::default().style(Style::default().bg(Color::Black)),
                area,
            );

            let horizon = height / 2;

            // 2. Render background stars
            for star in &stars {
                let sx = (star.x * width as f32) as u16;
                let sy = (star.y * horizon as f32) as u16;
                if sx < width && sy < horizon {
                    let twinkle = ((elapsed * 3.0 + star.phase).sin() * 0.5 + 0.5).clamp(0.2, 1.0);
                    let col = if let Color::Rgb(r, g, b) = star.base_color {
                        Color::Rgb((r as f32 * twinkle) as u8, (g as f32 * twinkle) as u8, (b as f32 * twinkle) as u8)
                    } else {
                        star.base_color
                    };
                    let char_str = if star.phase > 3.0 { "." } else { "•" };
                    f.render_widget(Paragraph::new(char_str).style(Style::default().fg(col)), Rect::new(sx, sy, 1, 1));
                }
            }

            // 3. Render Outrun Sun (top half)
            let sun_y_center = horizon.saturating_sub(1) as f32;
            let sun_x_center = width as f32 / 2.0;
            let radius_y = (horizon as f32 * 0.75).clamp(5.0, 18.0);
            let radius_x = radius_y / 0.55;

            for y_val in 0..horizon {
                let dy = y_val as f32 - sun_y_center;
                if dy <= 0.0 {
                    let pct = (dy + radius_y) / radius_y;
                    if pct >= 0.0 && pct <= 1.0 {
                        // Scanlines: threshold increases at bottom
                        let stripe_freq = 1.6;
                        let val = (dy * stripe_freq).sin();
                        let threshold = -1.1 + pct * 1.95;

                        if val > threshold {
                            let width_at_y = (radius_x * (1.0 - (dy / radius_y).powi(2)).max(0.0).sqrt()).round() as i32;
                            let x_start = (sun_x_center - width_at_y as f32).round() as i32;
                            let x_end = (sun_x_center + width_at_y as f32).round() as i32;

                            let col = sun_color(pct);
                            let draw_width = (x_end - x_start + 1).max(1) as u16;
                            let draw_x = x_start.max(0) as u16;
                            if draw_x < width && draw_width > 0 {
                                let active_width = draw_width.min(width - draw_x);
                                let fill_str = "█".repeat(active_width as usize);
                                f.render_widget(
                                    Paragraph::new(fill_str).style(Style::default().fg(col)),
                                    Rect::new(draw_x, y_val, active_width, 1)
                                );
                            }
                        }
                    }
                }
            }

            // 4. Render Horizon Glow Line
            if horizon < height {
                let glow_str = "═".repeat(width as usize);
                f.render_widget(
                    Paragraph::new(glow_str).style(Style::default().fg(Color::Rgb(255, 0, 180)).bold()),
                    Rect::new(0, horizon, width, 1)
                );
            }

            // 5. Render 3D perspective scrolling grid (bottom half)
            if horizon + 1 < height {
                let grid_height = height - horizon - 1;
                let scroll_speed = 1.2;
                let scroll = (elapsed * scroll_speed) % 1.0;

                // Identify row indices for horizontal grid lines
                let mut is_horiz_line = vec![false; height as usize];
                for i in 1..25 {
                    let z = i as f32 - scroll;
                    if z > 0.4 {
                        let t_norm = (1.0 / z - 0.04) / (2.5 - 0.04);
                        if t_norm >= 0.0 && t_norm <= 1.0 {
                            let y_pos = horizon + 1 + (t_norm * grid_height as f32) as u16;
                            if y_pos < height {
                                is_horiz_line[y_pos as usize] = true;
                            }
                        }
                    }
                }

                // Render each grid row
                let num_diagonals = 16;
                for y in (horizon + 1)..height {
                    let t = (y - horizon - 1) as f32 / grid_height.max(1) as f32;
                    let is_horiz = is_horiz_line[y as usize];

                    // Draw horizontal line if active
                    if is_horiz {
                        let horiz_str = "─".repeat(width as usize);
                        // Make line brighter if closer to bottom
                        let color_intensity = (100.0 + t * 155.0) as u8;
                        f.render_widget(
                            Paragraph::new(horiz_str).style(Style::default().fg(Color::Rgb(color_intensity, 0, color_intensity))),
                            Rect::new(0, y, width, 1)
                        );
                    }

                    // Overlay diagonal perspective lines
                    for j in 0..=num_diagonals {
                        let x_bottom = width as f32 * j as f32 / num_diagonals as f32;
                        let x_pos = (width as f32 / 2.0) + t * (x_bottom - (width as f32 / 2.0));
                        let x_idx = x_pos.round() as i32;

                        if x_idx >= 0 && x_idx < width as i32 {
                            let ch = if x_idx < (width as i32 / 2) {
                                "\\"
                            } else if x_idx > (width as i32 / 2) {
                                "/"
                            } else {
                                "│"
                            };

                            let cell_style = if is_horiz {
                                // Intersection glow
                                Style::default().fg(Color::Rgb(0, 255, 255)).bold()
                            } else {
                                // Fading perspective line
                                let r = (50.0 + t * 70.0) as u8;
                                let b = (80.0 + t * 100.0) as u8;
                                Style::default().fg(Color::Rgb(r, 0, b))
                            };

                            f.render_widget(Paragraph::new(ch).style(cell_style), Rect::new(x_idx as u16, y, 1, 1));
                        }
                    }
                }
            }

            // 6. Centered Vibrating Metallic Logo
            let logo_x = (width as i16 - logo_width as i16) / 2;
            let logo_y = (height as i16 - logo_height as i16) / 2;

            // Calculate vibration glitch offsets
            let tick = (elapsed * 22.0) as i32;
            let mut vibe_dx = 0;
            let mut vibe_dy = 0;
            if tick % 8 == 0 {
                vibe_dx = if tick % 16 == 0 { 1 } else { -1 };
            }
            if tick % 13 == 0 {
                vibe_dy = if tick % 26 == 0 { 1 } else { -1 };
            }

            let final_x = (logo_x + vibe_dx).clamp(0, width as i16) as u16;
            let final_y = (logo_y + vibe_dy).clamp(0, height as i16) as u16;

            // Draw a solid black block backdrop for the logo so it pops
            let bg_x = final_x.saturating_sub(2);
            let bg_y = final_y.saturating_sub(1);
            let bg_w = (logo_width + 4).min(width.saturating_sub(bg_x));
            let bg_h = (logo_height + 2).min(height.saturating_sub(bg_y));
            f.render_widget(
                ratatui::widgets::Block::default().style(Style::default().bg(Color::Black)),
                Rect::new(bg_x, bg_y, bg_w, bg_h),
            );

            // Draw metallic logo lines
            for (i, line) in logo_lines.iter().enumerate() {
                let y = final_y + i as u16;
                if y < height {
                    let color = logo_colors[i.min(logo_colors.len() - 1)];
                    let para = Paragraph::new(*line).style(Style::default().fg(color).bold());
                    f.render_widget(para, Rect::new(final_x, y, logo_width, 1));
                }
            }
        })?;

        std::thread::sleep(Duration::from_millis(15));
    }
}
