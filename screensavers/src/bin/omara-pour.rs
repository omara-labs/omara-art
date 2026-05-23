// omara-pour - Cascading / liquid pour screensaver

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, style::{Color, Style}, widgets::Paragraph, Terminal};
use std::io;
use std::time::{Duration, Instant};

use omara_screensavers::{branding, effects::pour::create_pour_effect};

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
    let mut effect = create_pour_effect(&art);

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

            let text: String = screen.into_iter().map(|r| r.into_iter().collect::<String>()).collect::<Vec<_>>().join("\n");
            let p = Paragraph::new(text).style(Style::default().fg(Color::Rgb(120, 200, 255)));
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
