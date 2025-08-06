use crossterm::{
    event::{Event, KeyCode, poll, read},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::Rng;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{io, thread, time::Duration, time::Instant};

use aquatui::config::*;
use aquatui::entities::{Fish, Bubble, Seaweed};
use aquatui::renderer::{draw_seaweed, render_fish, render_bubble, render_seaweed, render_kame_house};

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let size = terminal.size()?;
    let (cols, rows) = (size.width, size.height);
    let mut rng = rand::rng();
    let mut last_frame = Instant::now();

    let mut fishes: Vec<Fish> = (0..NUM_FISH)
        .map(|_| Fish::new(cols - 2, rows - 2, &mut rng))
        .collect();
    let mut bubbles: Vec<Bubble> = (0..NUM_BUBBLES)
        .map(|_| {
            Bubble::new(
                rng.random_range(1..cols - 1),
                rng.random_range(1..rows - 1),
                rng.random_range(0..3),
            )
        })
        .collect();
    let mut seaweeds: Vec<Seaweed> = (0..NUM_SEAWEEDS)
        .map(|_| {
            Seaweed::new(
                rng.random_range(1..cols - 1),
                rows - 5,
                rng.random_range(0..3),
                &mut rng,
            )
        })
        .collect();
    
    // Trier une seule fois au début
    bubbles.sort_by_key(|b| b.z_index);
    seaweeds.sort_by_key(|s| s.z_index);
    
    let mut paused = false;
    let mut speed_multiplier = 1.0f64;
    
    loop {
        terminal.draw(|f| {
            let area = f.area();
            let status = if paused { " [PAUSE]" } else { "" };
            let title = format!("Aquatui - Speed: {:.1}x{} - [q]uit [SPACE]pause [+/-]speed", speed_multiplier, status);
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White));
            f.render_widget(&block, area);

            let inner = block.inner(area);
            let mut lines =
                vec![Line::from(vec![Span::raw(" "); inner.width as usize]); inner.height as usize];

            // Affichage des poissons (multi-lignes)
            for fish in &fishes {
                render_fish(&mut lines, fish, inner.width);
            }

            // Bulles
            for bubble in &bubbles {
                render_bubble(&mut lines, bubble, inner.width);
            }

            // Algues
            draw_seaweed(&mut lines, cols, rows);
            for seaweed in &seaweeds {
                render_seaweed(&mut lines, seaweed, inner.width, inner.height);
            }

            // Kame House
            render_kame_house(&mut lines, cols, rows);

            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, inner);
        })?;

        let adjusted_duration = Duration::from_millis((FRAME_DURATION_MS as f64 / speed_multiplier) as u64);
        if last_frame.elapsed() >= adjusted_duration && !paused {
            fishes = fishes
                .into_iter()
                .filter_map(|mut fish| {
                    if fish.update(cols - 2) {
                        Some(fish)
                    } else {
                        None
                    }
                })
                .collect();

            while fishes.len() < NUM_FISH {
                fishes.push(Fish::new(cols - 2, rows - 2, &mut rng));
            }
            // Mise à jour des bulles
            for bubble in &mut bubbles {
                bubble.update(rows - 2, cols - 1, &mut rng);
            }
            // Mise à jours des algues
            for seaweed in &mut seaweeds {
                seaweed.update();
            }
            last_frame = Instant::now();
        }

        // Gestion des touches
        if poll(Duration::from_millis(1))? {
            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        speed_multiplier = (speed_multiplier + 0.1).min(5.0);
                    }
                    KeyCode::Char('-') => {
                        speed_multiplier = (speed_multiplier - 0.1).max(0.1);
                    }
                    KeyCode::Char('r') => {
                        // Reset speed
                        speed_multiplier = 1.0;
                        paused = false;
                    }
                    _ => {}
                }
            }
        }

        // Pause très courte pour éviter 100% CPU
        thread::sleep(Duration::from_millis(SLEEP_DURATION_MS));
    }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}