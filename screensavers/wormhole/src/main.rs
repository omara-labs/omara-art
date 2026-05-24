// omara-wormhole - Fully self-contained Star Wars hyperspace fly-through.
// Streaking stars, swirling blue warp tunnel, vibrating logo, and occasional majestic space whales (Purrgil).

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style, Modifier},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use rand::Rng;

// === BRANDING ===
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

struct TunnelStar {
    x: f32,
    y: f32,
    z: f32,
}

// ASCII Art of the majestic bioluminescent space whale (Purrgil)
const WHALE_ART: &[&str] = &[
    r"       .---.                                                 ",
    r"    .-'     '--..___                                         ",
    r"   /   o   x        `''---...___                             ",
    r"  |  X                          `''--.._                     ",
    r"   \          .____..---.               `'-._                ",
    r"    '.      .'           \  \  \             '.              ",
    r"      '----'              \  \  \    ~~ ~      |             ",
    r"                           \  \  \  ~~~~~     /              ",
    r"                            \__\__\ ~~~~    .'               ",
    r"                             (~~~)  ~~~   .-'                ",
    r"                              \  \       /                   ",
    r"                               \__\     |                    ",
];

const WHALE_WIDTH: i16 = 60;
const WHALE_HEIGHT: i16 = 12;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_hyperspace(&mut terminal);

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

fn run_hyperspace<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut rng = rand::rng();

    let mut stars: Vec<TunnelStar> = Vec::new();

    // Space whale state
    let mut whale_x = 0.0f32;
    let mut whale_y = 0.0f32;
    let mut show_whale = false;
    let mut whale_dir = true; // true = left-to-right, false = right-to-left
    let mut next_whale_time = Instant::now() + Duration::from_secs(4); // spawn first whale quickly

    let mut last_frame = Instant::now();
    let start_time = Instant::now();
    let mut last_width = 0;
    let mut last_height = 0;

    loop {
        // Exit on key press or mouse event
        if event::poll(Duration::from_millis(8))? {
            if matches!(event::read()?, Event::Key(_) | Event::Mouse(_)) {
                break Ok(());
            }
        }

        let now = Instant::now();
        let delta = now.duration_since(last_frame).as_secs_f32().min(0.1);
        last_frame = now;

        let time = start_time.elapsed().as_secs_f32();

        let size = terminal.size()?;
        let width = size.width;
        let height = size.height;

        if width == 0 || height == 0 {
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }

        // Initialize / Resize handling
        if width != last_width || height != last_height {
            stars.clear();

            // Star counts scale with screen area
            let target_stars = (width as usize * 3 / 2).clamp(60, 200);
            for _ in 0..target_stars {
                let theta = rng.random::<f32>() * std::f32::consts::TAU;
                let r = rng.random::<f32>() * 22.0 + 1.2;
                stars.push(TunnelStar {
                    x: theta.cos() * r,
                    y: theta.sin() * r,
                    z: rng.random::<f32>() * 99.0 + 1.0,
                });
            }

            last_width = width;
            last_height = height;
        }

        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;

        // 1. Update space whale trajectory
        if !show_whale && Instant::now() > next_whale_time {
            show_whale = true;
            whale_dir = rng.random_bool(0.5);
            if whale_dir {
                whale_x = -(WHALE_WIDTH as f32) - 5.0;
            } else {
                whale_x = width as f32 + 5.0;
            }
            whale_y = center_y - (WHALE_HEIGHT as f32 / 2.0);
        }

        if show_whale {
            let swim_speed = 13.5f32;
            if whale_dir {
                whale_x += swim_speed * delta;
            } else {
                whale_x -= swim_speed * delta;
            }
            // Gentle wave swimming motion
            whale_y = center_y - (WHALE_HEIGHT as f32 / 2.0) - 2.0 + (time * 1.4).sin() * 2.5;

            // Check if whale is fully off-screen
            if (whale_dir && whale_x > width as f32 + 10.0)
                || (!whale_dir && whale_x < -(WHALE_WIDTH as f32) - 10.0)
            {
                show_whale = false;
                next_whale_time = Instant::now() + Duration::from_secs(rng.random_range(16..30));
            }
        }

        // 2. Update tunnel stars (streaking forward)
        let tunnel_speed = 82.0f32; // Hyperspace speed!
        for star in &mut stars {
            star.z -= tunnel_speed * delta;

            if star.z <= 1.0 {
                star.z = 100.0;
                let theta = rng.random::<f32>() * std::f32::consts::TAU;
                let r = rng.random::<f32>() * 22.0 + 1.2;
                star.x = theta.cos() * r;
                star.y = theta.sin() * r;
            }
        }

        // Render
        terminal.draw(|f| {
            let area = f.area();
            let buf = f.buffer_mut();

            // 1. Draw swirling hyperspace background tunnel
            for y in 0..area.height {
                let dy = y as f32 - center_y;
                for x in 0..area.width {
                    let dx = (x as f32 - center_x) * 0.55;
                    let dist = (dx * dx + dy * dy).sqrt();

                    let cell = &mut buf[(x, y)];
                    cell.set_symbol(" ");

                    // Concentric swirling warp tunnel
                    let angle = dy.atan2(dx);
                    let wave = ((dist * 0.28 - time * 8.5) + angle * 2.2).sin();

                    if wave > 0.45 {
                        let scale = (wave - 0.45) * 1.82; // 0.0 to 1.0
                        let bg_g = (scale * 28.0) as u8;
                        let bg_b = (12.0 + scale * 52.0) as u8;
                        cell.set_style(Style::default().bg(Color::Rgb(0, bg_g, bg_b)));
                    } else {
                        cell.set_style(Style::default().bg(Color::Black));
                    }
                }
            }

            // fov perspective scaling
            let fov = area.width as f32 * 0.45;

            // 2. Draw tunnel star streaks
            for star in &stars {
                let z_prev = star.z + tunnel_speed * delta;

                let x_curr = center_x + (star.x / star.z) * fov;
                let y_curr = center_y + (star.y / star.z) * fov * 0.55;

                let x_prev = center_x + (star.x / z_prev) * fov;
                let y_prev = center_y + (star.y / z_prev) * fov * 0.55;

                let dx = x_curr - x_prev;
                let dy = y_curr - y_prev;
                let dist = (dx*dx + dy*dy).sqrt();

                // Draw line from prev to curr
                if dist > 0.5 && x_curr >= 0.0 && x_curr < area.width as f32 && y_curr >= 0.0 && y_curr < area.height as f32 {
                    let steps = (dist as usize).clamp(2, 11);
                    for step in 0..steps {
                        let t = step as f32 / (steps - 1) as f32;
                        let px = (x_prev + dx * t) as u16;
                        let py = (y_prev + dy * t) as u16;

                        if px < area.width && py < area.height {
                            let intensity = 1.0 - (star.z / 100.0);
                            let color_ratio = t * intensity;

                            // Star streak colors (deep blue to bright white at head)
                            let r = (color_ratio * 215.0) as u8;
                            let g = (85.0 + color_ratio * 170.0) as u8;
                            let b = 255;

                            let cell = &mut buf[(px, py)];
                            let ch = if step == steps - 1 { '*' } else { '.' };

                            // Avoid overwriting where the logo center is
                            let is_logo_center = px >= (area.width.saturating_sub(logo_width) / 2)
                                && px < (area.width.saturating_sub(logo_width) / 2) + logo_width
                                && py >= (area.height.saturating_sub(logo_height) / 2)
                                && py < (area.height.saturating_sub(logo_height) / 2) + logo_height;

                            if !is_logo_center {
                                cell.set_symbol(&ch.to_string());
                                cell.set_style(cell.style().fg(Color::Rgb(r, g, b)));
                            }
                        }
                    }
                }
            }

            // 3. Draw space whale (Purrgil) with transparent background
            if show_whale {
                let wx_start = whale_x as i16;
                let wy_start = whale_y as i16;

                for (row, line) in WHALE_ART.iter().enumerate() {
                    let py = wy_start + row as i16;
                    if py < 0 || py >= area.height as i16 { continue; }

                    for (col, ch) in line.chars().enumerate() {
                        if ch == ' ' { continue; }
                        let px = wx_start + col as i16;
                        if px < 0 || px >= area.width as i16 { continue; }

                        let cell = &mut buf[(px as u16, py as u16)];

                        // Swirly bioluminescent cycling colors: body cyan, tentacles purple
                        let is_tentacle = ch == '~' || ch == '(' || ch == ')' || row > 8;
                        let color = if is_tentacle {
                            Color::Rgb(165, 30, 255) // bright cosmic purple
                        } else {
                            // pulsing cyan
                            let pulse = ((time * 2.2).sin() + 1.0) * 0.5;
                            let r = (pulse * 50.0) as u8;
                            let g = (175.0 + pulse * 80.0) as u8;
                            let b = (220.0 + pulse * 35.0) as u8;
                            Color::Rgb(r, g, b)
                        };

                        cell.set_symbol(&ch.to_string());
                        // Maintain the existing background color (warp tunnel lanes) to look semi-transparent!
                        cell.set_style(cell.style().fg(color).add_modifier(Modifier::BOLD));
                    }
                }
            }

            // 4. Draw Logo in the center with flight vibration/jitter
            let jitter_x = if rng.random_bool(0.45) { rng.random_range(-1..=1) } else { 0 };
            let jitter_y = if rng.random_bool(0.45) { rng.random_range(-1..=1) } else { 0 };

            let logo_x = (area.width.saturating_sub(logo_width) / 2) as i16 + jitter_x;
            let logo_y = (area.height.saturating_sub(logo_height) / 2) as i16 + jitter_y;

            for (y_offset, line) in logo_lines.iter().enumerate() {
                let py = logo_y + y_offset as i16;
                if py < 0 || py >= area.height as i16 { continue; }

                for (x_offset, ch) in line.chars().enumerate() {
                    if ch == ' ' { continue; }
                    let px = logo_x + x_offset as i16;
                    if px < 0 || px >= area.width as i16 { continue; }

                    let cell = &mut buf[(px as u16, py as u16)];
                    cell.set_symbol(&ch.to_string());
                    // Bright white-blue glow
                    cell.set_style(cell.style()
                        .fg(Color::Rgb(225, 248, 255))
                        .add_modifier(Modifier::BOLD));
                }
            }
        })?;

        let frame_time = Duration::from_millis(16);
        let elapsed_time = now.elapsed();
        if let Some(remaining) = frame_time.checked_sub(elapsed_time) {
            std::thread::sleep(remaining);
        }
    }
}
