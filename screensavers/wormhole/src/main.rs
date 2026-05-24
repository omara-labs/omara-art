// omara-wormhole - Fully self-contained space travel screensaver.
// Spaceship with OMARA logo cruising at sublight speed and transitioning into warp speed.

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

struct TunnelStar {
    x: f32,
    y: f32,
    z: f32,
}

struct FlameParticle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
}

// Third-person rear view of the OMARA spaceship
const SHIP_ART: &[&str] = &[
    r"         /\         ",
    r"        /  \        ",
    r"       /    \       ",
    r"     .-'      '-.   ",
    r"    /   OMARA    \  ",
    r"   |  .-.____.-.  | ",
    r"   |  |  ||||  |  | ",
    r"   '-/ \_/\_/\_/ \-'",
    r"     \  | | | |  /  ",
    r"      '-'-'-'-'-'   ",
];

const SHIP_WIDTH: u16 = 20;
const SHIP_HEIGHT: u16 = 10;

#[derive(PartialEq, Clone, Copy)]
enum WarpPhase {
    SublightCruise,
    WarpEngage,
    WarpCruising,
    WarpExit,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_spaceship_warp(&mut terminal);

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

fn run_spaceship_warp<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::rng();

    let mut stars: Vec<TunnelStar> = Vec::new();
    let mut flames: Vec<FlameParticle> = Vec::new();

    // Journey state machine
    let mut phase = WarpPhase::SublightCruise;
    let mut phase_timer = Instant::now();
    let mut current_speed;

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
            flames.clear();

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

        // JOURNEY LOOP STATE UPDATE
        let elapsed = phase_timer.elapsed().as_secs_f32();
        match phase {
            WarpPhase::SublightCruise => {
                current_speed = 0.0; // stars stay stationary/twinkle

                if elapsed > 6.0 {
                    phase = WarpPhase::WarpEngage;
                    phase_timer = Instant::now();
                }
            }
            WarpPhase::WarpEngage => {
                let ratio = (elapsed / 1.5).min(1.0);
                current_speed = ratio * 85.0; // accelerate

                if elapsed > 1.5 {
                    phase = WarpPhase::WarpCruising;
                    phase_timer = Instant::now();
                }
            }
            WarpPhase::WarpCruising => {
                current_speed = 85.0; // full warp speed

                if elapsed > 7.0 {
                    phase = WarpPhase::WarpExit;
                    phase_timer = Instant::now();
                }
            }
            WarpPhase::WarpExit => {
                let ratio = (elapsed / 1.5).min(1.0);
                current_speed = (1.0 - ratio) * 85.0; // decelerate

                if elapsed > 1.5 {
                    phase = WarpPhase::SublightCruise;
                    phase_timer = Instant::now();
                }
            }
        }

        // Update star depths
        for star in &mut stars {
            star.z -= current_speed * delta;

            // Reset stars that zoomed past
            if star.z <= 1.0 {
                star.z = 100.0;
                let theta = rng.random::<f32>() * std::f32::consts::TAU;
                let r = rng.random::<f32>() * 22.0 + 1.2;
                star.x = theta.cos() * r;
                star.y = theta.sin() * r;
            }
        }

        // Calculate spaceship positions
        let mut ship_x = center_x - SHIP_WIDTH as f32 / 2.0;
        // Floating motion during sublight, vibrating jitter during warp
        let mut ship_y = center_y - SHIP_HEIGHT as f32 / 2.0;

        if phase == WarpPhase::SublightCruise {
            ship_y += (time * 1.5).sin() * 1.2;
        } else if current_speed > 10.0 {
            // Jitter vibration
            ship_x += if rng.random_bool(0.45) { rng.random_range(-1..=1) as f32 } else { 0.0 };
            ship_y += if rng.random_bool(0.45) { rng.random_range(-1..=1) as f32 } else { 0.0 };
        }

        // Spawn engine flame particles
        let spawn_rate = if current_speed > 10.0 { 4 } else { 1 };
        for _ in 0..spawn_rate {
            let nozzle_offsets = [-3.0f32, -1.0, 1.0, 3.0];
            let offset = nozzle_offsets[rng.random_range(0..4)];
            let px = ship_x + 10.0 + offset + rng.random::<f32>() * 0.6 - 0.3;
            let py = ship_y + 9.2;

            let vx = rng.random::<f32>() * 0.8 - 0.4;
            let vy = if current_speed > 10.0 {
                rng.random::<f32>() * 12.0 + 9.0 // high energy warp flare
            } else {
                rng.random::<f32>() * 3.5 + 1.5 // low energy sublight flicker
            };

            flames.push(FlameParticle {
                x: px,
                y: py,
                vx,
                vy,
                life: 1.0,
            });
        }

        // Update flame particles
        for f in &mut flames {
            f.x += f.vx * delta;
            f.y += f.vy * delta;
            f.life -= if current_speed > 10.0 { delta * 2.8 } else { delta * 6.5 };
        }
        flames.retain(|f| f.life > 0.0);

        // Render
        terminal.draw(|f| {
            let area = f.area();
            let buf = f.buffer_mut();

            // 1. Clear background to solid black space
            for y in 0..area.height {
                for x in 0..area.width {
                    let cell = &mut buf[(x, y)];
                    cell.set_symbol(" ");
                    cell.set_style(Style::default().bg(Color::Black));
                }
            }

            // fov perspective scaling
            let fov = area.width as f32 * 0.45;

            // 2. Draw stars & star streaks
            for star in &stars {
                let x_curr = center_x + (star.x / star.z) * fov;
                let y_curr = center_y + (star.y / star.z) * fov * 0.55;

                let px = x_curr as u16;
                let py = y_curr as u16;

                if current_speed < 1.0 {
                    // Sublight cruise (stationary/twinkling stars)
                    if px < area.width && py < area.height {
                        let h = (star.x.abs().wrapping_to_u32() ^ star.y.abs().wrapping_to_u32()) as f32;
                        let sparkle = ((time * 1.5 + h).sin() + 1.0) * 0.5;

                        if sparkle > 0.15 {
                            let intensity = (70.0 + sparkle * 185.0) as u8;
                            let ch = if star.z < 25.0 { '✦' } else if star.z < 65.0 { '•' } else { '.' };
                            let cell = &mut buf[(px, py)];
                            cell.set_symbol(&ch.to_string());
                            cell.set_style(Style::default().fg(Color::Rgb(intensity, intensity, intensity + 20)));
                        }
                    }
                } else {
                    // Warp streaks
                    let z_prev = star.z + current_speed * delta;
                    let x_prev = center_x + (star.x / z_prev) * fov;
                    let y_prev = center_y + (star.y / z_prev) * fov * 0.55;

                    let dx = x_curr - x_prev;
                    let dy = y_curr - y_prev;
                    let dist = (dx*dx + dy*dy).sqrt();

                    if dist > 0.5 && x_curr >= 0.0 && x_curr < area.width as f32 && y_curr >= 0.0 && y_curr < area.height as f32 {
                        let steps = (dist as usize).clamp(2, 11);
                        for step in 0..steps {
                            let t = step as f32 / (steps - 1) as f32;
                            let sx = (x_prev + dx * t) as u16;
                            let sy = (y_prev + dy * t) as u16;

                            if sx < area.width && sy < area.height {
                                let intensity = 1.0 - (star.z / 100.0);
                                let color_ratio = t * intensity;

                                let r = (color_ratio * 220.0) as u8;
                                let g = (90.0 + color_ratio * 165.0) as u8;
                                let b = 255;

                                let cell = &mut buf[(sx, sy)];
                                let ch = if step == steps - 1 { '*' } else { '.' };

                                // Avoid overwriting where the ship is
                                let is_ship_zone = sx >= ship_x as u16
                                    && sx < ship_x as u16 + SHIP_WIDTH
                                    && sy >= ship_y as u16
                                    && sy < ship_y as u16 + SHIP_HEIGHT;

                                if !is_ship_zone {
                                    cell.set_symbol(&ch.to_string());
                                    cell.set_style(Style::default().fg(Color::Rgb(r, g, b)));
                                }
                            }
                        }
                    }
                }
            }

            // 3. Draw engine flame particles
            for f in &flames {
                let fx = f.x as u16;
                let fy = f.y as u16;

                if fx < area.width && fy < area.height {
                    let cell = &mut buf[(fx, fy)];
                    let (r, g, b) = if current_speed > 10.0 {
                        // Warp engine: hot electric blue-white flame
                        let l = f.life;
                        ((l * 180.0) as u8, (120.0 + l * 135.0) as u8, 255)
                    } else {
                        // Sublight engine: flickering orange-red flame
                        let l = f.life;
                        (255, (l * 120.0) as u8, 0)
                    };

                    let ch = if f.life > 0.65 { '*' } else if f.life > 0.35 { '+' } else { '.' };
                    cell.set_symbol(&ch.to_string());
                    cell.set_style(Style::default().fg(Color::Rgb(r, g, b)));
                }
            }

            // 4. Draw OMARA Spaceship
            let sx_start = ship_x as i16;
            let sy_start = ship_y as i16;

            for (row, line) in SHIP_ART.iter().enumerate() {
                let py = sy_start + row as i16;
                if py >= 0 && py < area.height as i16 {
                    for (col, ch) in line.chars().enumerate() {
                        if ch == ' ' { continue; }
                        let px = sx_start + col as i16;
                        if px >= 0 && px < area.width as i16 {
                            let cell = &mut buf[(px as u16, py as u16)];

                            // Color design:
                            // Hull: Steel Blue/Grey
                            // OMARA Text: Electric white-blue
                            // Wings/Details: Cyan
                            let color = if row == 4 && col >= 8 && col <= 12 {
                                Color::Rgb(225, 248, 255) // Logo
                            } else if row == 5 || row == 6 {
                                Color::Rgb(0, 220, 255) // Inner panels
                            } else {
                                Color::Rgb(105, 125, 150) // Outer hull
                            };

                            cell.set_symbol(&ch.to_string());
                            let mut style = Style::default().fg(color);
                            if (row == 4 && col >= 8 && col <= 12) || row == 0 {
                                style = style.add_modifier(Modifier::BOLD);
                            }
                            cell.set_style(style);
                        }
                    }
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

// Helper trait to convert coordinate types safely
trait F32Ext {
    fn wrapping_to_u32(self) -> u32;
}

impl F32Ext for f32 {
    fn wrapping_to_u32(self) -> u32 {
        self as u32
    }
}
