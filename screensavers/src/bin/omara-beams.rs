// omara-beams - Sweeping beam / spotlight screensaver for Omara (purple theme)

use omara_screensavers::{branding, effects};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, style::{Color, Style}, widgets::Paragraph, Terminal};
use std::io;
use std::time::{Duration, Instant};

use crate::effects::beams::create_beams_effect;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run(&mut terminal);

    execute!(
        terminal.backend_mut(),
        crossterm::cursor::Show,
        LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal::disable_raw_mode()?;

    result
}

fn run<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {
    let art = branding::load_branding();
    let mut effect = create_beams_effect(&art);

    // Moving beams
    let mut beams = vec![8.0, 22.0, 35.0];
    let mut dirs = vec![0.9, -0.7, 1.1];

    let start = Instant::now();
    let mut last = Instant::now();

    loop {
        if event::poll(Duration::from_millis(8))? {
            if matches!(event::read()?, Event::Key(_) | Event::Mouse(_)) {
                break;
            }
        }

        // Auto-exit after ~30 seconds
        if start.elapsed() > Duration::from_secs(30) {
            break;
        }

        effect.tick();

        // Move beams
        for (b, d) in beams.iter_mut().zip(dirs.iter_mut()) {
            *b += *d;
            if *b < 0.0 { *b = 0.0; *d = -*d; }
            if *b > effect.width as f32 { *b = effect.width as f32; *d = -*d; }
        }

        terminal.draw(|f| {
            let size = f.area();
            f.render_widget(ratatui::widgets::Block::default().style(Style::default().bg(Color::Black)), size);

            let mut screen = vec![vec![' '; effect.width as usize]; effect.height as usize];
            for g in &effect.glyphs {
                let ix = g.x.round() as isize;
                let iy = g.y.round() as isize;
                if ix >= 0 && iy >= 0 && (ix as u16) < effect.width && (iy as u16) < effect.height {
                    screen[iy as usize][ix as usize] = g.ch;
                }
            }

            // Draw visible moving beams as bright vertical lines
            let beam_char = '│'; // or '█' for thicker look
            for &beam_x in &beams {
                let col = beam_x.round() as isize;
                if col >= 0 && col < effect.width as isize {
                    for row in 0..effect.height as usize {
                        screen[row][col as usize] = beam_char;
                    }
                }
            }

            let text: String = screen.into_iter().map(|r| r.into_iter().collect::<String>()).collect::<Vec<_>>().join("\n");
            let p = Paragraph::new(text).style(Style::default().fg(Color::Rgb(255, 180, 255))); // bright purple for beams
            f.render_widget(p, centered_rect(effect.width + 4, effect.height + 2, size));
        })?;

        let now = Instant::now();
        if let Some(d) = Duration::from_millis(16).checked_sub(now.duration_since(last)) {
            std::thread::sleep(d);
        }
        last = now;
    }

    Ok(())
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let v = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length((r.height.saturating_sub(height))/2),
            ratatui::layout::Constraint::Length(height),
            ratatui::layout::Constraint::Length((r.height.saturating_sub(height))/2),
        ]).split(r);

    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Length((r.width.saturating_sub(width))/2),
            ratatui::layout::Constraint::Length(width),
            ratatui::layout::Constraint::Length((r.width.saturating_sub(width))/2),
        ]).split(v[1])[1]
}
