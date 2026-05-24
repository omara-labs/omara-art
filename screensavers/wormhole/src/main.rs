// omara-wormhole - Fully self-contained Star Trek / Star Wars warp speed screensaver.
// Clean star warp streaks against solid black space, logo cockpit vibration, and planet-to-planet hyperspace journey.

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

// Compact, clean ASCII planets representing space destinations
const PLANETS: &[&[&str]] = &[
    &[
        r"          .------.          ",
        r"        .-' .--. '-.        ",
        r"      ./  ./    \.  \.      ",
        r"   ===|  |        |  |===   ",
        r"      '\  '\    /'  /'      ",
        r"        '-. '--' .-'        ",
        r"           '------'         ",
    ],
    &[
        r"         .------.         ",
        r"       .-'  ()  '-.       ",
        r"      /  ()    ()  \      ",
        r"     |              |     ",
        r"     |   ()    ()   |     ",
        r"      \            /      ",
        r"       '-.______.-'       ",
    ],
    &[
        r"         .------.         ",
        r"       .-'  ~~  '-.       ",
        r"      /  ~~    ~   \      ",
        r"     |  ~  ~~~~     |     ",
        r"     |   ~~~~~      |     ",
        r"      \    ~       /      ",
        r"       '-.______.-'       ",
    ],
];

const PLANET_COLORS: &[Color] = &[
    Color::Rgb(240, 155, 30),  // Orange ringed gas giant
    Color::Rgb(150, 150, 150), // Grey cratered rocky moon
    Color::Rgb(0, 180, 240),   // Cyan/blue ocean planet
];

#[derive(PartialEq, Clone, Copy)]
enum WarpPhase {
    Arrived,
    HyperspaceEntry,
    HyperspaceCruising,
    HyperspaceExit,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_starways_warp(&mut terminal);

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

fn run_starways_warp<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut rng = rand::rng();

    let mut stars: Vec<TunnelStar> = Vec::new();

    // Hyperspace journey state machine
    let mut phase = WarpPhase::Arrived;
    let mut phase_timer = Instant::now();
    let mut current_speed;
    let mut planet_idx = 0;

    let mut planet_x = 0.0f32;
    let mut planet_y = 0.0f32;
    let mut planet_target_y = 0.0f32;

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

            // Align planet on left side
            planet_x = (width as f32 * 0.10).clamp(2.0, 15.0);
            planet_target_y = height as f32 / 2.0 - 3.5;
            planet_y = planet_target_y;

            last_width = width;
            last_height = height;
        }

        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;

        // JOURNEY LOOP STATE UPDATE
        let elapsed = phase_timer.elapsed().as_secs_f32();
        match phase {
            WarpPhase::Arrived => {
                current_speed = 0.0;
                // Gentle orbital floating
                planet_y = planet_target_y + (time * 1.1).sin() * 0.5;

                if elapsed > 5.5 {
                    phase = WarpPhase::HyperspaceEntry;
                    phase_timer = Instant::now();
                }
            }
            WarpPhase::HyperspaceEntry => {
                let ratio = (elapsed / 1.5).min(1.0);
                // Accelerate rapidly
                current_speed = ratio * 85.0;

                // Planet flies downwards off-screen
                planet_y += 35.0 * delta;

                if elapsed > 1.5 {
                    phase = WarpPhase::HyperspaceCruising;
                    phase_timer = Instant::now();
                }
            }
            WarpPhase::HyperspaceCruising => {
                current_speed = 85.0;

                if elapsed > 6.5 {
                    phase = WarpPhase::HyperspaceExit;
                    phase_timer = Instant::now();

                    // Swap planet index for arrival
                    planet_idx = (planet_idx + 1) % PLANETS.len();
                    // Spawn new planet high above the screen
                    planet_y = -12.0;
                }
            }
            WarpPhase::HyperspaceExit => {
                let ratio = (elapsed / 1.5).min(1.0);
                // Decelerate rapidly
                current_speed = (1.0 - ratio) * 85.0;

                // New planet glides down into resting orbital position
                planet_y += (planet_target_y - planet_y) * 4.2 * delta;

                if elapsed > 1.5 {
                    phase = WarpPhase::Arrived;
                    phase_timer = Instant::now();
                    planet_y = planet_target_y;
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

            // 2. Draw stars & star streaks (depending on speed)
            for star in &stars {
                let x_curr = center_x + (star.x / star.z) * fov;
                let y_curr = center_y + (star.y / star.z) * fov * 0.55;

                let px = x_curr as u16;
                let py = y_curr as u16;

                if current_speed < 1.0 {
                    // Warp is inactive (Arrived): Draw twinkling stationary stars
                    if px < area.width && py < area.height {
                        let h = (star.x.abs().wrapping_to_u32() ^ star.y.abs().wrapping_to_u32()) as f32;
                        let sparkle = ((time * 1.5 + h).sin() + 1.0) * 0.5;

                        if sparkle > 0.15 {
                            let intensity = (80.0 + sparkle * 175.0) as u8;
                            let ch = if star.z < 25.0 { '✦' } else if star.z < 65.0 { '•' } else { '.' };
                            let cell = &mut buf[(px, py)];
                            cell.set_symbol(&ch.to_string());
                            cell.set_style(Style::default().fg(Color::Rgb(intensity, intensity, intensity + 20)));
                        }
                    }
                } else {
                    // Warp is active: Draw star streaks
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

                                // Streaks shift from deep blue to white-hot at front
                                let r = (color_ratio * 220.0) as u8;
                                let g = (90.0 + color_ratio * 165.0) as u8;
                                let b = 255;

                                let cell = &mut buf[(sx, sy)];
                                let ch = if step == steps - 1 { '*' } else { '.' };

                                // Avoid overwriting where the logo center is
                                let is_logo_center = sx >= (area.width.saturating_sub(logo_width) / 2)
                                    && sx < (area.width.saturating_sub(logo_width) / 2) + logo_width
                                    && sy >= (area.height.saturating_sub(logo_height) / 2)
                                    && sy < (area.height.saturating_sub(logo_height) / 2) + logo_height;

                                if !is_logo_center {
                                    cell.set_symbol(&ch.to_string());
                                    cell.set_style(Style::default().fg(Color::Rgb(r, g, b)));
                                }
                            }
                        }
                    }
                }
            }

            // 3. Draw planet destination (if on screen)
            let p_art = PLANETS[planet_idx];
            let p_color = PLANET_COLORS[planet_idx];
            let px_start = planet_x as i16;
            let py_start = planet_y as i16;

            for (row, line) in p_art.iter().enumerate() {
                let py = py_start + row as i16;
                if py >= 0 && py < area.height as i16 {
                    for (col, ch) in line.chars().enumerate() {
                        if ch == ' ' { continue; }
                        let px = px_start + col as i16;
                        if px >= 0 && px < area.width as i16 {
                            let cell = &mut buf[(px as u16, py as u16)];
                            cell.set_symbol(&ch.to_string());
                            cell.set_style(Style::default().fg(p_color));
                        }
                    }
                }
            }

            // 4. Draw Logo in center (vibrates slightly at high speed)
            let jitter_x = if current_speed > 10.0 && rng.random_bool(0.4) { rng.random_range(-1..=1) } else { 0 };
            let jitter_y = if current_speed > 10.0 && rng.random_bool(0.4) { rng.random_range(-1..=1) } else { 0 };

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
                    
                    // Electric white-blue glow
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

// Simple helper trait to get numeric values without compiling warnings
trait F32Ext {
    fn wrapping_to_u32(self) -> u32;
}

impl F32Ext for f32 {
    fn wrapping_to_u32(self) -> u32 {
        self as u32
    }
}
