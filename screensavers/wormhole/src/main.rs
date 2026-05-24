// omara-goku - Fully self-contained Goku Kamehameha ASCII screensaver.
// Walking in, going Super Saiyan, charging energy, blasting a massive beam, and shaking the OMARA logo.

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

// Stances (10x10 grids)
const GOKU_WALK_A: &[&str] = &[
    r"   _/\_   ",
    r"  /    \  ",
    r"  \(o.o)/ ",
    r"   \-=-/  ",
    r"  /[Gi]\  ",
    r"  | | | | ",
    r"  | | | | ",
    r"   /   \  ",
    r"  /     \ ",
    r" _\     _\",
];

const GOKU_WALK_B: &[&str] = &[
    r"   _/\_   ",
    r"  /    \  ",
    r"  \(o.o)/ ",
    r"   \-=-/  ",
    r"  /[Gi]\  ",
    r"  | | | | ",
    r"  | | | | ",
    r"   |   |  ",
    r"   /   \  ",
    r"  /_   /_ ",
];

const GOKU_POWER: &[&str] = &[
    r"   _/\_   ",
    r"  /    \  ",
    r"  \(O.O)/ ",
    r"   \-=-/  ",
    r" _/[Gi]\_ ",
    r"  | | | | ",
    r"  | | | | ",
    r"   /   \  ",
    r"  |     | ",
    r"  |     | ",
];

const GOKU_CHARGE: &[&str] = &[
    r"   _/\_   ",
    r"  /    \  ",
    r"  \(>.<)/ ",
    r"   \-=-/  ",
    r"  /[Gi]\  ",
    r" /  | | | ",
    r"|  /___/  ",
    r"   /   \  ",
    r"  /     \ ",
    r" _\     _\",
];

const GOKU_FIRE: &[&str] = &[
    r"   _/\_   ",
    r"  /    \  ",
    r"  \(>O<)/ ",
    r"   \-=-/  ",
    r"  /[Gi]=====",
    r" /  | |   ",
    r"|  /  |   ",
    r"   /   \  ",
    r"  /     \ ",
    r" _\     _\",
];

struct AuraParticle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
}

struct Spark {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    color: Color,
}

struct ChargeParticle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Phase {
    WalkingIn,
    PoweringUp,
    Charging,
    Blasting,
    CoolDown,
    WalkingOut,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_goku(&mut terminal);

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

fn run_goku<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut rng = rand::rng();

    let mut aura_particles: Vec<AuraParticle> = Vec::new();
    let mut sparks: Vec<Spark> = Vec::new();
    let mut charge_particles: Vec<ChargeParticle> = Vec::new();

    let mut phase = Phase::WalkingIn;
    let mut phase_timer = Instant::now();

    let mut goku_x = -15.0f32;
    let mut goku_y = 0.0f32;

    let mut last_frame = Instant::now();
    let start_time = Instant::now();
    let mut last_width = 0;
    let mut last_height = 0;

    let blast_symbols = ["█", "▓", "▒", "░", "=", "%", "*", "+"];

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
            goku_x = -15.0f32;
            goku_y = height as f32 / 2.0 - 5.0; // center vertically

            phase = Phase::WalkingIn;
            phase_timer = Instant::now();

            aura_particles.clear();
            sparks.clear();
            charge_particles.clear();

            last_width = width;
            last_height = height;
        }

        let target_x = width as f32 * 0.18;
        let center_y = height as f32 / 2.0;

        let logo_target_x = width.saturating_sub(logo_width) / 2 + 10;
        let logo_x = (width as f32 * 0.58).max(logo_target_x as f32);
        let logo_y = center_y - (logo_height as f32 / 2.0);

        let elapsed = phase_timer.elapsed().as_secs_f32();

        // STATE MACHINE
        match phase {
            Phase::WalkingIn => {
                goku_x += 11.5 * delta;
                if goku_x >= target_x {
                    goku_x = target_x;
                    phase = Phase::PoweringUp;
                    phase_timer = Instant::now();
                }
            }
            Phase::PoweringUp => {
                // Gold Super Saiyan aura particles
                if aura_particles.len() < 50 {
                    aura_particles.push(AuraParticle {
                        x: goku_x + 5.0 + rng.random::<f32>() * 6.0 - 3.0,
                        y: goku_y + 9.5,
                        vx: rng.random::<f32>() * 2.0 - 1.0,
                        vy: -(rng.random::<f32>() * 8.0 + 4.0),
                        life: 1.0,
                    });
                }

                if elapsed > 2.5 {
                    phase = Phase::Charging;
                    phase_timer = Instant::now();
                }
            }
            Phase::Charging => {
                // Aura particles continue
                if aura_particles.len() < 60 {
                    aura_particles.push(AuraParticle {
                        x: goku_x + 5.0 + rng.random::<f32>() * 6.0 - 3.0,
                        y: goku_y + 9.5,
                        vx: rng.random::<f32>() * 2.0 - 1.0,
                        vy: -(rng.random::<f32>() * 9.0 + 5.0),
                        life: 1.0,
                    });
                }

                // Swirling blue charge particles flying inwards to the hands (hands cupped at goku_x + 9, goku_y + 6)
                let hand_x = goku_x + 9.0;
                let hand_y = goku_y + 6.0;
                if charge_particles.len() < 40 {
                    let angle = rng.random::<f32>() * std::f32::consts::TAU;
                    let r = 16.0f32;
                    charge_particles.push(ChargeParticle {
                        x: hand_x + angle.cos() * r * 2.0,
                        y: hand_y + angle.sin() * r,
                        vx: -angle.cos() * 12.0,
                        vy: -angle.sin() * 6.0,
                        life: 1.0,
                    });
                }

                // Update charge particles
                for cp in &mut charge_particles {
                    cp.x += cp.vx * delta;
                    cp.y += cp.vy * delta;
                    // Shrink life as they get closer
                    let dx = cp.x - hand_x;
                    let dy = cp.y - hand_y;
                    let dist = (dx*dx + dy*dy).sqrt();
                    cp.life = (dist / 32.0).min(1.0);
                }
                charge_particles.retain(|cp| cp.life > 0.05);

                if elapsed > 2.5 {
                    phase = Phase::Blasting;
                    phase_timer = Instant::now();
                    charge_particles.clear();
                }
            }
            Phase::Blasting => {
                // High-intensity aura particles
                if aura_particles.len() < 75 {
                    aura_particles.push(AuraParticle {
                        x: goku_x + 5.0 + rng.random::<f32>() * 6.0 - 3.0,
                        y: goku_y + 9.5,
                        vx: rng.random::<f32>() * 3.5 - 1.75,
                        vy: -(rng.random::<f32>() * 12.0 + 7.0),
                        life: 1.0,
                    });
                }

                // Spawn splash impact sparks at the logo impact zone (logo_x, logo_y + 4)
                let impact_x = logo_x;
                let impact_y = logo_y + 4.0;
                for _ in 0..3 {
                    sparks.push(Spark {
                        x: impact_x + rng.random::<f32>() * 1.5 - 0.75,
                        y: impact_y + rng.random::<f32>() * 2.0 - 1.0,
                        vx: rng.random::<f32>() * 28.0 + 8.0,
                        vy: rng.random::<f32>() * 16.0 - 8.0,
                        life: 1.0,
                        color: if rng.random_bool(0.7) {
                            Color::Rgb(0, 240, 255) // Cyan sparks
                        } else {
                            Color::Rgb(255, 255, 255) // White sparks
                        },
                    });
                }

                if elapsed > 3.8 {
                    phase = Phase::CoolDown;
                    phase_timer = Instant::now();
                }
            }
            Phase::CoolDown => {
                if elapsed > 2.0 {
                    phase = Phase::WalkingOut;
                    phase_timer = Instant::now();
                }
            }
            Phase::WalkingOut => {
                goku_x += 11.5 * delta;
                if goku_x > width as f32 + 5.0 {
                    goku_x = -15.0f32;
                    phase = Phase::WalkingIn;
                    phase_timer = Instant::now();
                }
            }
        }

        // UPDATE PARTICLES
        // Update aura particles
        for p in &mut aura_particles {
            p.x += p.vx * delta;
            p.y += p.vy * delta;
            p.life -= delta * 1.4;
        }
        aura_particles.retain(|p| p.life > 0.0);

        // Update sparks
        for s in &mut sparks {
            s.x += s.vx * delta;
            s.y += s.vy * delta;
            // Gravity drift downwards
            s.vy += 8.5 * delta;
            s.life -= delta * 2.2;
        }
        sparks.retain(|s| s.life > 0.0);

        // RENDER
        terminal.draw(|f| {
            let area = f.area();
            let buf = f.buffer_mut();

            // Clear screen (black background)
            for y in 0..area.height {
                for x in 0..area.width {
                    let cell = &mut buf[(x, y)];
                    cell.set_symbol(" ");
                    cell.set_style(Style::default().bg(Color::Black));
                }
            }

            // 1. Draw Aura Particles (SSJ Flame)
            if phase == Phase::PoweringUp || phase == Phase::Charging || phase == Phase::Blasting {
                for p in &aura_particles {
                    let px = p.x as u16;
                    let py = p.y as u16;
                    if px < area.width && py < area.height {
                        let cell = &mut buf[(px, py)];
                        // Flame sparkles: gold/yellow/orange
                        let alpha = (p.life * 255.0) as u8;
                        let r = 255;
                        let g = (140.0 + p.life * 115.0) as u8;
                        let b = (alpha / 4) as u8;

                        cell.set_symbol(if p.life > 0.5 { "^" } else { "." });
                        cell.set_style(Style::default().fg(Color::Rgb(r, g, b)));
                    }
                }
            }

            // 2. Draw Charging Particles
            if phase == Phase::Charging {
                // Swirling blue dots
                for cp in &charge_particles {
                    let cpx = cp.x as u16;
                    let cpy = cp.y as u16;
                    if cpx < area.width && cpy < area.height {
                        let cell = &mut buf[(cpx, cpy)];
                        let alpha = (cp.life * 255.0) as u8;
                        cell.set_symbol(".");
                        cell.set_style(Style::default().fg(Color::Rgb(0, alpha, 255)));
                    }
                }

                // Main energy sphere in hands (goku_x + 9, goku_y + 6)
                let hx = (goku_x + 9.0) as i16;
                let hy = (goku_y + 6.0) as i16;
                let size_factor = (elapsed / 2.5).min(1.0);
                let r_sphere = (size_factor * 2.2) as i16;

                for dy in -r_sphere..=r_sphere {
                    for dx in -r_sphere * 2..=r_sphere * 2 {
                        let sx = hx + dx;
                        let sy = hy + dy;

                        // Circular constraint
                        let rx = dx as f32 * 0.5;
                        let ry = dy as f32;
                        if (rx*rx + ry*ry).sqrt() <= size_factor * 2.0 {
                            if sx >= 0 && sx < area.width as i16 && sy >= 0 && sy < area.height as i16 {
                                let cell = &mut buf[(sx as u16, sy as u16)];
                                cell.set_symbol("O");
                                cell.set_style(Style::default()
                                    .fg(Color::Rgb(200, 245, 255))
                                    .add_modifier(Modifier::BOLD));
                            }
                        }
                    }
                }
            }

            // 3. Draw Kamehameha Energy Blast Beam (only in Blasting)
            if phase == Phase::Blasting {
                let bx_start = (goku_x + 10.0) as u16;
                let bx_end = logo_x as u16;
                let by = (goku_y + 4.0) as u16;

                if bx_start < bx_end {
                    // Draw a 3-line thick beam
                    for x in bx_start..bx_end {
                        if x >= area.width { continue; }

                        for dy in -1..=1 {
                            let py = (by as i16 + dy) as u16;
                            if py >= area.height { continue; }

                            // Swirling, moving texture along beam
                            let sx_idx = ((x as f32 - time * 35.0) as i32).abs() as usize % blast_symbols.len();
                            let ch = blast_symbols[sx_idx];

                            // Core is white-cyan, outer beam is blue
                            let color = if dy == 0 {
                                Color::Rgb(220, 250, 255) // White core
                            } else {
                                Color::Rgb(0, (140.0 + (x % 5) as f32 * 20.0) as u8, 255) // Blue-cyan outer
                            };

                            let cell = &mut buf[(x, py)];
                            cell.set_symbol(ch);
                            cell.set_style(Style::default().fg(color).add_modifier(Modifier::BOLD));
                        }
                    }
                }
            }

            // 4. Draw Splash sparks (Impact stars)
            if phase == Phase::Blasting {
                for s in &sparks {
                    let sx = s.x as u16;
                    let sy = s.y as u16;
                    if sx < area.width && sy < area.height {
                        let cell = &mut buf[(sx, sy)];
                        let ch = if s.life > 0.6 { "*" } else { "." };
                        cell.set_symbol(ch);
                        cell.set_style(Style::default().fg(s.color));
                    }
                }
            }

            // 5. Draw Goku Stance
            let is_ssj = phase == Phase::PoweringUp || phase == Phase::Charging || phase == Phase::Blasting;
            let stance = match phase {
                Phase::WalkingIn | Phase::WalkingOut => {
                    let step = (time * 6.5) as i32 % 2;
                    if step == 0 { GOKU_WALK_A } else { GOKU_WALK_B }
                }
                Phase::PoweringUp => GOKU_POWER,
                Phase::Charging => GOKU_CHARGE,
                Phase::Blasting => GOKU_FIRE,
                Phase::CoolDown => GOKU_WALK_A,
            };

            let gx_start = goku_x as i16;
            let gy_start = goku_y as i16;

            for (row, line) in stance.iter().enumerate() {
                let py = gy_start + row as i16;
                if py >= 0 && py < area.height as i16 {
                    for (col, ch) in line.chars().enumerate() {
                        if ch == ' ' { continue; }
                        let px = gx_start + col as i16;
                        if px >= 0 && px < area.width as i16 {
                            let cell = &mut buf[(px as u16, py as u16)];

                            // Color sprite elements dynamically
                            let color = if row <= 1 || (row == 2 && (col == 2 || col == 6)) {
                                // Spiky Hair
                                if is_ssj {
                                    Color::Rgb(255, 215, 0) // Golden Yellow (Super Saiyan!)
                                } else {
                                    Color::Rgb(95, 95, 95) // Grey base hair
                                }
                            } else if row == 2 || (row == 3 && col != 3) {
                                Color::Rgb(255, 200, 150) // Peach Skin
                            } else if ch == '=' {
                                Color::Rgb(255, 200, 150) // Extended skin arms
                            } else if ch == '[' || ch == 'G' || ch == 'i' || ch == ']' {
                                Color::Rgb(255, 90, 0) // Orange Gi chest
                            } else {
                                Color::Rgb(255, 90, 0) // Gi pants / legs
                            };

                            cell.set_symbol(&ch.to_string());
                            let mut style = Style::default().fg(color);
                            if is_ssj || row == 0 {
                                style = style.add_modifier(Modifier::BOLD);
                            }
                            cell.set_style(style);
                        }
                    }
                }
            }

            // 6. Draw Anime Dialogue bubble above Goku's head (e.g. goku_y - 2)
            let speak_text = match phase {
                Phase::PoweringUp => {
                    if elapsed < 1.2 { Some("KA...") } else { Some("ME...") }
                }
                Phase::Charging => {
                    if elapsed < 1.2 { Some("..HA...") } else { Some("..ME...") }
                }
                Phase::Blasting => {
                    Some("HA!!!!!")
                }
                _ => None,
            };

            if let Some(text) = speak_text {
                let tx = (goku_x + 3.0) as i16;
                let ty = (goku_y - 2.0) as i16;

                if ty >= 0 && ty < area.height as i16 {
                    for (i, ch) in text.chars().enumerate() {
                        let px = tx + i as i16;
                        if px >= 0 && px < area.width as i16 {
                            let cell = &mut buf[(px as u16, ty as u16)];
                            cell.set_symbol(&ch.to_string());
                            cell.set_style(Style::default()
                                .fg(Color::Rgb(255, 255, 255))
                                .add_modifier(Modifier::BOLD));
                        }
                    }
                }
            }

            // 7. Draw OMARA logo with vibration offsets during impact
            let mut jitter_x = 0;
            let mut jitter_y = 0;
            if phase == Phase::Blasting {
                // Violent screen shaking during energy blast!
                jitter_x = rng.random_range(-2..=2);
                jitter_y = rng.random_range(-1..=1);
            }

            let lx_start = (logo_x as i16 + jitter_x).max(0) as u16;
            let ly_start = (logo_y as i16 + jitter_y).max(0) as u16;

            for (y_offset, line) in logo_lines.iter().enumerate() {
                let py = ly_start + y_offset as u16;
                if py >= area.height { continue; }

                for (x_offset, ch) in line.chars().enumerate() {
                    if ch == ' ' { continue; }
                    let px = lx_start + x_offset as u16;
                    if px >= area.width { continue; }

                    let cell = &mut buf[(px, py)];
                    cell.set_symbol(&ch.to_string());

                    // Logo colors react to energy blast impact:
                    // Flashes white-hot during blast, else standard indigo-purple
                    let logo_color = if phase == Phase::Blasting {
                        let flash = rng.random_range(160..=255) as u8;
                        Color::Rgb(flash, 255, 255) // Cyan-white blast reaction
                    } else if phase == Phase::CoolDown {
                        // Slowly returning to purple
                        let ratio = (elapsed / 2.0).min(1.0);
                        let r = (255.0 * (1.0 - ratio) + 140.0 * ratio) as u8;
                        let g = (255.0 * (1.0 - ratio) + 20.0 * ratio) as u8;
                        let b = (255.0 * (1.0 - ratio) + 240.0 * ratio) as u8;
                        Color::Rgb(r, g, b)
                    } else {
                        Color::Rgb(140, 20, 240) // Standard Purple
                    };

                    cell.set_style(Style::default()
                        .fg(logo_color)
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

