// omara-singularity - Fully self-contained gravitational lensing black hole screensaver.
// Einstein rings, gravitational coordinate warping, and swirling accretion disk.

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

// Pseudo-random starfield generator using coordinates and time
fn get_star_at(x: f32, y: f32, time: f32) -> Option<(char, Color)> {
    let gx = x.round() as i32;
    let gy = y.round() as i32;

    // Simple hash function for grid coordinates
    let h = (gx.wrapping_mul(12973).wrapping_add(gy.wrapping_mul(78643))) as f32;
    let rand_val = (h.sin() * 43758.5453).fract().abs();

    // Density: roughly 0.7% of cosmic coordinate cells contain stars
    if rand_val < 0.007 {
        let phase = rand_val * std::f32::consts::TAU;
        let sparkle = ((time * 1.8 + phase).sin() + 1.0) * 0.5;

        if sparkle > 0.1 {
            let ch = if rand_val < 0.0008 {
                '✦' // bright star
            } else if rand_val < 0.0022 {
                '•' // medium star
            } else {
                '.' // faint star
            };

            // Star color: white/blue tint, fading with sparkle
            let b = (90.0 + sparkle * 165.0) as u8;
            let rg = (60.0 + sparkle * 195.0) as u8;
            return Some((ch, Color::Rgb(rg, rg, b)));
        }
    }
    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_singularity(&mut terminal);

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

fn run_singularity<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let start_time = Instant::now();

    loop {
        // Exit on key press or mouse event
        if event::poll(Duration::from_millis(8))? {
            if matches!(event::read()?, Event::Key(_) | Event::Mouse(_)) {
                break Ok(());
            }
        }

        let time = start_time.elapsed().as_secs_f32();

        let size = terminal.size()?;
        let width = size.width;
        let height = size.height;

        if width == 0 || height == 0 {
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }

        // Center of the black hole
        let bh_x = width as f32 / 2.0;
        let bh_y = height as f32 / 2.0;

        // Size of the Event Horizon (scales with screen size)
        let r_event = (height as f32 * 0.14).clamp(4.0, 11.0);

        // Slow Lissajous curve for floating logo movement
        let logo_max_x = width.saturating_sub(logo_width) as f32;
        let logo_max_y = height.saturating_sub(logo_height) as f32;
        
        let logo_center_x = logo_max_x / 2.0 + (time * 0.18).sin() * logo_max_x * 0.42;
        let logo_center_y = logo_max_y / 2.0 + (time * 0.11).cos() * logo_max_y * 0.32;

        terminal.draw(|f| {
            let area = f.area();
            let buf = f.buffer_mut();

            for y in 0..area.height {
                for x in 0..area.width {
                    // 1. Calculate distance from the Singularity
                    let dx = (x as f32 - bh_x) * 0.55; // Aspect-ratio scaling
                    let dy = y as f32 - bh_y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    let cell = &mut buf[(x, y)];

                    // 2. Render Event Horizon (Singularity Void)
                    if dist <= r_event {
                        cell.set_symbol(" ");
                        cell.set_style(Style::default().bg(Color::Black));
                        continue;
                    }

                    // 3. Relativistic Accretion Disk (Swirling Hot Gas)
                    // Disk lives just outside the Event Horizon
                    let disk_width = 8.5f32;
                    let disk_dist = dist - r_event;
                    let mut is_disk = false;
                    let mut disk_char = ' ';
                    let mut disk_style = Style::default();

                    if disk_dist < disk_width {
                        is_disk = true;
                        let disk_intensity = 1.0 - (disk_dist / disk_width); // fades outwards

                        // Spiral arm calculation based on cell angle and distance
                        let cell_angle = dy.atan2(dx);
                        let rotation_speed = 3.5 / dist.max(1.0);
                        let sample_angle = cell_angle - time * rotation_speed;

                        // 3 spiral arms around the hole
                        let swirl = ((sample_angle * 3.0 + dist * 0.6).sin() + 1.0) * 0.5;
                        let intensity = disk_intensity * (0.35 + 0.65 * swirl);

                        // Color shifts: White-Hot near horizon -> Orange -> Red -> Faint Purple on edges
                        let r_val = (110.0 + intensity * 145.0).min(255.0) as u8;
                        let g_val = (intensity * 125.0) as u8;
                        let b_val = (intensity * 40.0 + (1.0 - intensity) * 35.0) as u8;

                        disk_char = if intensity > 0.72 {
                            '▓'
                        } else if intensity > 0.52 {
                            '▒'
                        } else if intensity > 0.32 {
                            '░'
                        } else if intensity > 0.15 {
                            '*'
                        } else {
                            '.'
                        };

                        disk_style = Style::default()
                            .fg(Color::Rgb(r_val, g_val, b_val))
                            .bg(Color::Rgb((r_val / 9) as u8, (g_val / 9) as u8, (b_val / 9) as u8));
                    }

                    // 4. Gravitational Lensing (Deflection of Background Spacetime)
                    // Trace ray backwards: source coordinates are pushed outwards from center
                    let lens_ratio = 1.0 + (r_event * r_event) / (dist * dist - r_event * r_event).max(0.005);
                    let source_x = bh_x + (dx * lens_ratio) / 0.55;
                    let source_y = bh_y + (dy * lens_ratio);

                    // 5. Look up Logo in warped source coordinates
                    let mut drawn_logo = false;
                    let lx = (source_x - logo_center_x) as i32;
                    let ly = (source_y - logo_center_y) as i32;

                    if lx >= 0 && lx < logo_width as i32 && ly >= 0 && ly < logo_height as i32 {
                        let line = logo_lines[ly as usize];
                        if let Some(ch) = line.chars().nth(lx as usize) {
                            if ch != ' ' {
                                // Draw lensed logo character!
                                drawn_logo = true;

                                // Base Logo style is electric indigo/purple
                                // Gravitational stress adds heating/white glow closer to horizon
                                let stress = (1.0 / (dist - r_event).max(0.1)).min(1.0);
                                let l_r = (140.0 + stress * 115.0) as u8;
                                let l_g = (20.0 + stress * 220.0) as u8;
                                let l_b = (255.0) as u8;

                                cell.set_symbol(&ch.to_string());
                                let mut style = Style::default()
                                    .fg(Color::Rgb(l_r, l_g, l_b))
                                    .add_modifier(Modifier::BOLD);

                                // Blend background with accretion disk if applicable
                                if is_disk {
                                    style = style.bg(disk_style.bg.unwrap_or(Color::Black));
                                } else {
                                    style = style.bg(Color::Black);
                                }

                                cell.set_style(style);
                            }
                        }
                    }

                    if !drawn_logo {
                        // 6. Look up Starfield in warped source coordinates
                        if let Some((star_ch, star_color)) = get_star_at(source_x, source_y, time) {
                            cell.set_symbol(&star_ch.to_string());
                            let mut style = Style::default().fg(star_color);
                            if is_disk {
                                style = style.bg(disk_style.bg.unwrap_or(Color::Black));
                            } else {
                                style = style.bg(Color::Black);
                            }
                            cell.set_style(style);
                        } else if is_disk {
                            // 7. Draw accretion disk particles if nothing else is in front
                            cell.set_symbol(&disk_char.to_string());
                            cell.set_style(disk_style);
                        } else {
                            // Empty deep space
                            cell.set_symbol(" ");
                            cell.set_style(Style::default().bg(Color::Black));
                        }
                    }
                }
            }
        })?;

        let frame_time = Duration::from_millis(16);
        let elapsed = start_time.elapsed(); // using start_time to keep frame pacing standard
        let last_elapsed = elapsed.as_secs_f32() - time;
        let target_frame_time = frame_time.as_secs_f32();
        if last_elapsed < target_frame_time {
            let sleep_dur = Duration::from_secs_f32(target_frame_time - last_elapsed);
            std::thread::sleep(sleep_dur);
        }
    }
}
