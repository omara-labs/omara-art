// omara-garden - Zen Sakura and Vine growth screensaver.
// Self-contained, resize-friendly.

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

#[derive(Clone, Copy, Debug, PartialEq)]
enum CellState {
    Empty,
    Vine,
    Leaf,
    LogoNormal(char),
    LogoVine(char),
    LogoFlower { ch: char, color: Color },
}

struct Grower {
    x: u16,
    y: u16,
    active: bool,
}

struct Petal {
    x: f32,
    y: f32,
    speed_y: f32,
    speed_x: f32,
    phase: f32,
    ch: char,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_garden(&mut terminal);

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

fn run_garden<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut rng = rand::rng();
    let start_time = Instant::now();
    let mut last_frame = Instant::now();

    // Petals particle system
    let petal_chars = ['❀', '✿', '❁', '❃', '•', '*', '.'];
    let mut petals: Vec<Petal> = (0..100)
        .map(|_| Petal {
            x: rng.random::<f32>() * 120.0,
            y: rng.random::<f32>() * 40.0,
            speed_y: rng.random::<f32>() * 3.0 + 1.5,
            speed_x: rng.random::<f32>() * 1.5 - 0.75,
            phase: rng.random::<f32>() * std::f32::consts::TAU,
            ch: petal_chars[rng.random_range(0..petal_chars.len())],
        })
        .collect();

    // Vine growth state
    let mut width: u16 = 0;
    let mut height: u16 = 0;
    let mut grid: Vec<CellState> = Vec::new();
    let mut growers: Vec<Grower> = Vec::new();
    let mut growth_timer = Instant::now();
    let mut cycle_reset_timer: Option<Instant> = None;

    let grid_index = |x: u16, y: u16, w: u16| -> usize {
        (y as usize) * (w as usize) + (x as usize)
    };

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
        let current_width = size.width;
        let current_height = size.height;

        // Reset grid and growers on resize or first frame
        if current_width != width || current_height != height {
            width = current_width;
            height = current_height;

            // Re-allocate grid
            grid = vec![CellState::Empty; (width as usize) * (height as usize)];

            // Insert logo in center
            let logo_x = (width.saturating_sub(logo_width) / 2) as usize;
            let logo_y = (height.saturating_sub(logo_height) / 2) as usize;

            for (y_offset, line) in logo_lines.iter().enumerate() {
                let ly = logo_y + y_offset;
                if ly < height as usize {
                    for (x_offset, ch) in line.chars().enumerate() {
                        let lx = logo_x + x_offset;
                        if lx < width as usize && ch != ' ' {
                            let idx = ly * (width as usize) + lx;
                            grid[idx] = CellState::LogoNormal(ch);
                        }
                    }
                }
            }

            // Spawn initial growers at the bottom
            growers.clear();
            let num_growers = (width / 12).clamp(4, 12);
            for i in 0..num_growers {
                growers.push(Grower {
                    x: (width as f32 * (i as f32 + 0.5) / num_growers as f32) as u16,
                    y: height.saturating_sub(1),
                    active: true,
                });
            }
            cycle_reset_timer = None;

            // Resize petals boundary
            for p in &mut petals {
                p.x = rng.random::<f32>() * width as f32;
                p.y = rng.random::<f32>() * height as f32;
            }
        }

        // Update falling petals
        let wind = (elapsed * 0.8).sin() * 2.0; // Dynamic drifting wind
        for p in &mut petals {
            p.y += p.speed_y * delta;
            p.x += (p.speed_x + wind + (elapsed * 1.5 + p.phase).sin() * 0.5) * delta;

            // Recycle petals
            if p.y >= height as f32 || p.x < 0.0 || p.x >= width as f32 {
                p.y = -1.0;
                p.x = rng.random::<f32>() * width as f32;
                p.speed_y = rng.random::<f32>() * 3.0 + 1.5;
                p.speed_x = rng.random::<f32>() * 1.5 - 0.75;
                p.ch = petal_chars[rng.random_range(0..petal_chars.len())];
            }
        }

        // Update vine growth logic
        if growth_timer.elapsed() >= Duration::from_millis(70) {
            growth_timer = Instant::now();

            let mut new_growers = Vec::new();
            let mut any_active = false;
            let current_growers_len = growers.len();

            // 1. Move growers
            for g in &mut growers {
                if !g.active { continue; }
                any_active = true;

                // Look for adjacent LogoNormal cells (climbing targets)
                let mut targets = Vec::new();
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        let nx = g.x as i16 + dx;
                        let ny = g.y as i16 + dy;
                        if nx >= 0 && nx < width as i16 && ny >= 0 && ny < height as i16 {
                            let idx = grid_index(nx as u16, ny as u16, width);
                            if let CellState::LogoNormal(_) = grid[idx] {
                                targets.push((nx as u16, ny as u16, idx));
                            }
                        }
                    }
                }

                if !targets.is_empty() {
                    // Growth climb: enter one of the adjacent LogoNormal cells
                    let next_idx = rng.random_range(0..targets.len());
                    let (nx, ny, idx) = targets[next_idx];
                    if let CellState::LogoNormal(ch) = grid[idx] {
                        grid[idx] = CellState::LogoVine(ch);
                    }
                    g.x = nx;
                    g.y = ny;

                    // Branching probability inside the logo
                    if rng.random_bool(0.15) && current_growers_len + new_growers.len() < 35 {
                        if targets.len() > 1 {
                            let b_idx = (next_idx + 1) % targets.len();
                            let (bx, by, bidx) = targets[b_idx];
                            if let CellState::LogoNormal(ch) = grid[bidx] {
                                grid[bidx] = CellState::LogoVine(ch);
                            }
                            new_growers.push(Grower { x: bx, y: by, active: true });
                        }
                    }
                } else {
                    // Not adjacent to logo. Seek nearest LogoNormal globally and move towards it.
                    let mut nearest: Option<(u16, u16)> = None;
                    let mut min_dist = 99999.0;

                    // Search for closest unvisited logo block
                    for ly in 0..height {
                        for lx in 0..width {
                            let idx = grid_index(lx, ly, width);
                            if let CellState::LogoNormal(_) = grid[idx] {
                                // Correct aspect ratio offset
                                let dist = (((lx as f32 - g.x as f32) * 0.55).powi(2) + (ly as f32 - g.y as f32).powi(2)).sqrt();
                                if dist < min_dist {
                                    min_dist = dist;
                                    nearest = Some((lx, ly));
                                }
                            }
                        }
                    }

                    if let Some((lx, ly)) = nearest {
                        let dx = (lx as i16 - g.x as i16).signum();
                        let dy = (ly as i16 - g.y as i16).signum();

                        let nx = (g.x as i16 + dx).clamp(0, width as i16 - 1) as u16;
                        let ny = (g.y as i16 + dy).clamp(0, height as i16 - 1) as u16;

                        let idx = grid_index(nx, ny, width);
                        if grid[idx] == CellState::Empty {
                            grid[idx] = if rng.random_bool(0.2) {
                                CellState::Leaf
                            } else {
                                CellState::Vine
                            };
                            g.x = nx;
                            g.y = ny;
                        } else {
                            // Blocked organically, try to crawl directly up
                            let ny_up = g.y.saturating_sub(1);
                            let idx_up = grid_index(g.x, ny_up, width);
                            if grid[idx_up] == CellState::Empty {
                                grid[idx_up] = CellState::Vine;
                                g.y = ny_up;
                            } else {
                                // Deactivate grower
                                g.active = false;
                            }
                        }
                    } else {
                        // No unvisited logo pieces remaining!
                        g.active = false;
                    }
                }
            }

            // Append new branches
            growers.extend(new_growers);

            // 2. Grow flowers randomly on visited vine-logo parts
            for idx in 0..grid.len() {
                if let CellState::LogoVine(_ch) = grid[idx] {
                    if rng.random_bool(0.04) {
                        let fl_chars = ['❀', '✿', '❁', '❃'];
                        let fl_ch = fl_chars[rng.random_range(0..fl_chars.len())];
                        let fl_colors = [
                            Color::Rgb(255, 130, 180), // Cherry Pink
                            Color::Rgb(255, 105, 180), // Hot Pink
                            Color::Rgb(255, 215, 0),   // Jasmine Yellow
                            Color::Rgb(186, 85, 211),  // Orchid Purple
                            Color::Rgb(0, 255, 220),   // Morning Glory Cyan
                            Color::Rgb(255, 140, 0),   // Marigold Orange
                        ];
                        let fl_col = fl_colors[rng.random_range(0..fl_colors.len())];
                        grid[idx] = CellState::LogoFlower { ch: fl_ch, color: fl_col };
                    }
                }
            }

            // 3. Handle resetting cycles
            if !any_active {
                if let Some(reset_time) = cycle_reset_timer {
                    if reset_time.elapsed() >= Duration::from_secs(6) {
                        // Re-trigger cycle
                        width = 0; // Force rebuild next loop
                    }
                } else {
                    cycle_reset_timer = Some(Instant::now());
                }
            }
        }

        // Draw Frame
        terminal.draw(|f| {
            let area = f.area();
            if area.width == 0 || area.height == 0 { return; }

            // Render static grid (vines + logo states)
            for y in 0..height {
                for x in 0..width {
                    let idx = grid_index(x, y, width);
                    if idx >= grid.len() { continue; }

                    match grid[idx] {
                        CellState::Empty => {}
                        CellState::Vine => {
                            let leaf_str = if (x + y) % 3 == 0 { "·" } else { "°" };
                            f.render_widget(
                                Paragraph::new(leaf_str).style(Style::default().fg(Color::Rgb(46, 120, 50))),
                                Rect::new(x, y, 1, 1),
                            );
                        }
                        CellState::Leaf => {
                            let leaf_char = if x % 2 == 0 { "v" } else { "y" };
                            f.render_widget(
                                Paragraph::new(leaf_char).style(Style::default().fg(Color::Rgb(34, 180, 50))),
                                Rect::new(x, y, 1, 1),
                            );
                        }
                        CellState::LogoNormal(ch) => {
                            f.render_widget(
                                Paragraph::new(ch.to_string()).style(Style::default().fg(Color::Rgb(140, 130, 120))),
                                Rect::new(x, y, 1, 1),
                            );
                        }
                        CellState::LogoVine(ch) => {
                            f.render_widget(
                                Paragraph::new(ch.to_string()).style(Style::default().fg(Color::Rgb(50, 220, 60))),
                                Rect::new(x, y, 1, 1),
                            );
                        }
                        CellState::LogoFlower { ch, color } => {
                            f.render_widget(
                                Paragraph::new(ch.to_string()).style(Style::default().fg(color)),
                                Rect::new(x, y, 1, 1),
                            );
                        }
                    }
                }
            }

            // Render wind-drifted sakura petals on top
            for p in &petals {
                let px = p.x as u16;
                let py = p.y as u16;
                if px < width && py < height {
                    let color = if p.ch == '.' {
                        Color::Rgb(255, 200, 210) // Light pink
                    } else if p.ch == '*' {
                        Color::Rgb(255, 160, 180) // Medium pink
                    } else {
                        Color::Rgb(255, 105, 180) // Vivid hot pink
                    };
                    f.render_widget(
                        Paragraph::new(p.ch.to_string()).style(Style::default().fg(color)),
                        Rect::new(px, py, 1, 1),
                    );
                }
            }
        })?;

        std::thread::sleep(Duration::from_millis(15));
    }
}
