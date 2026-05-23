// omara-bounce - Classic DVD-style bouncing logo screensaver for Omara
// Uses the big bold multi-line "Omara" logo. Smooth movement, color changes only on side hits.

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};

const OMARA_LOGO: &str = r#"
  ██████╗  ███╗   ███╗  █████╗  ██████╗   █████╗ 
 ██╔═══██╗ ████╗ ████║ ██╔══██╗ ██╔══██╗ ██╔══██╗
 ██║   ██║ ██╔████╔██║ ███████║ ██████╔╝ ███████║
 ██║   ██║ ██║╚██╔╝██║ ██╔══██║ ██╔══██╗ ██╔══██║
 ╚██████╔╝ ██║ ╚═╝ ██║ ██║  ██║ ██║  ██║ ██║  ██║
  ╚═════╝  ╚═╝     ╚═╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_bounce(&mut terminal);

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

fn run_bounce<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Fixed visual size for the bouncing logo block (matches the big ASCII art)
    const LOGO_WIDTH: u16 = 56;
    const LOGO_HEIGHT: u16 = 6;

    // Prepare the logo (we still use the full art string)
    // Start position
    let mut x: f32 = 5.0;
    let mut y: f32 = 3.0;

    // Nice relaxed but visible speed (cells per second) - tuned for smooth feel
    let mut vx: f32 = 7.0;
    let mut vy: f32 = 3.8;

    // Color palette
    let colors: Vec<Color> = vec![
        Color::Rgb(180, 0, 255),   // Omara purple
        Color::Rgb(0, 200, 255),   // Cyan
        Color::Rgb(255, 200, 0),   // Yellow
        Color::Rgb(0, 255, 150),   // Green
        Color::Rgb(255, 100, 180), // Pink
    ];
    let mut color_index: usize = 0;

    // Starfield (positions are normalized 0.0..1.0 so they scale with terminal size)
    struct Star {
        x: f32,
        y: f32,
        ch: char,
        base_color: Color,
        phase: f32,   // individual sparkle phase
    }

    let stars: Vec<Star> = (0..140)
        .map(|i| {
            let ch = if i % 17 == 0 { '✦' } else if i % 7 == 0 { '•' } else { '.' };
            let brightness = if i % 9 == 0 { 200 } else if i % 4 == 0 { 140 } else { 80 };
            Star {
                x: rand::random::<f32>(),
                y: rand::random::<f32>(),
                ch,
                base_color: Color::Rgb(brightness, brightness, brightness + 25),
                phase: rand::random::<f32>() * std::f32::consts::TAU,
            }
        })
        .collect();

    // A few "bright" stars that will have simple lens flares
    let flare_indices: Vec<usize> = vec![7, 29, 61];

    let mut last_frame = Instant::now();

    loop {
        // Exit only on user input
        if event::poll(Duration::from_millis(8))? {
            if matches!(event::read()?, Event::Key(_) | Event::Mouse(_)) {
                break Ok(());
            }
        }

        let now = Instant::now();
        let delta = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        let size = terminal.size()?;
        let max_x = size.width.saturating_sub(LOGO_WIDTH) as f32;
        let max_y = size.height.saturating_sub(LOGO_HEIGHT) as f32;

        // Delta-time movement for smoothness
        x += vx * delta;
        y += vy * delta;

        let mut hit_horizontal = false;
        let mut hit_vertical = false;

        // Left wall
        if x <= 0.0 {
            x = 0.0;
            vx = -vx;
            hit_horizontal = true;
        }
        // Right wall
        if x >= max_x {
            x = max_x;
            vx = -vx;
            hit_horizontal = true;
        }

        // Top wall
        if y <= 0.0 {
            y = 0.0;
            vy = -vy;
            hit_vertical = true;
        }
        // Bottom wall
        if y >= max_y {
            y = max_y;
            vy = -vy;
            hit_vertical = true;
        }

        // Only change color on corner hits (both horizontal and vertical in same frame)
        // This is the classic DVD logo behavior — color changes are rare and nice.
        if hit_horizontal || hit_vertical {
            color_index = (color_index + 1) % colors.len();
        }

        // Draw
        terminal.draw(|f| {
            let area = f.area();

            // Black background
            f.render_widget(
                ratatui::widgets::Block::default()
                    .style(Style::default().bg(Color::Black)),
                area,
            );

            // === Starfield + simple lens flares ===
            let time = last_frame.elapsed().as_secs_f32();

            for star in &stars {
                let sx = (star.x * area.width as f32) as u16;
                let sy = (star.y * area.height as f32) as u16;

                if sx < area.width && sy < area.height {
                    let mut c = star.base_color;

                    // Individual per-star sparkle
                    let sparkle = ((time * 2.7 + star.phase).sin() * 70.0) as i16;
                    if let Color::Rgb(r, g, b) = c {
                        let adj = sparkle.clamp(-55, 55) as u8;
                        c = Color::Rgb(
                            r.saturating_add(adj),
                            g.saturating_add(adj / 2),
                            b.saturating_add(adj / 3),
                        );
                    }

                    let star_para = Paragraph::new(star.ch.to_string()).style(Style::default().fg(c));
                    f.render_widget(star_para, Rect::new(sx, sy, 1, 1));
                }
            }

            // Simple lens flares on a few bright stars
            for &idx in &flare_indices {
                if let Some(star) = stars.get(idx) {
                    let sx = (star.x * area.width as f32) as u16;
                    let sy = (star.y * area.height as f32) as u16;

                    // Horizontal flare
                    for dx in 1..6 {
                        let alpha = 80 - (dx * 12) as u8;
                        if sx + dx < area.width {
                            let flare = Paragraph::new("─").style(Style::default().fg(Color::Rgb(alpha, alpha, alpha + 30)));
                            f.render_widget(flare, Rect::new(sx + dx, sy, 1, 1));
                        }
                        if sx >= dx {
                            let flare = Paragraph::new("─").style(Style::default().fg(Color::Rgb(alpha, alpha, alpha + 30)));
                            f.render_widget(flare, Rect::new(sx - dx, sy, 1, 1));
                        }
                    }

                    // Vertical flare
                    for dy in 1..4 {
                        let alpha = 70 - (dy * 15) as u8;
                        if sy + dy < area.height {
                            let flare = Paragraph::new("│").style(Style::default().fg(Color::Rgb(alpha, alpha, alpha + 20)));
                            f.render_widget(flare, Rect::new(sx, sy + dy, 1, 1));
                        }
                        if sy >= dy {
                            let flare = Paragraph::new("│").style(Style::default().fg(Color::Rgb(alpha, alpha, alpha + 20)));
                            f.render_widget(flare, Rect::new(sx, sy - dy, 1, 1));
                        }
                    }
                }
            }

            // === Bouncing Logo ===
            let logo_rect = Rect {
                x: x as u16,
                y: y as u16,
                width: LOGO_WIDTH,
                height: LOGO_HEIGHT,
            };

            let logo = Paragraph::new(OMARA_LOGO)
                .style(
                    Style::default()
                        .fg(colors[color_index])
                        .add_modifier(ratatui::style::Modifier::BOLD),
                )
                .alignment(ratatui::layout::Alignment::Left);

            f.render_widget(logo, logo_rect);
        })?;

        // Target ~60 FPS
        let frame_time = Duration::from_millis(16);
        let elapsed = now.elapsed();
        if let Some(remaining) = frame_time.checked_sub(elapsed) {
            std::thread::sleep(remaining);
        }
    }
}
