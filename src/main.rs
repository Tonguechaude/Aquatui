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
use std::{io, time::Duration, time::Instant};

mod lib;

use crate::lib::bubble::*;
use crate::lib::fish::*;
use crate::lib::seaweed::*;

const KAME_HOUSE: &str = r#"
  ____KAME____
 /   HOUSE    \
/______________\
| []   __   [] |
|____|_____|___|
"#;

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let size = terminal.size()?;
    let (cols, rows) = (size.width, size.height);
    let mut rng = rand::rng();
    let frame_duration = Duration::from_millis(100);
    let mut last_frame = Instant::now();

    let num_fish = 10;
    let mut fishes: Vec<Fish> = (0..num_fish)
        .map(|_| Fish::new(cols - 2, rows - 2, &mut rng))
        .collect();
    let mut bubbles: Vec<Bubble> = (0..40)
        .map(|_| {
            Bubble::new(
                rng.random_range(1..cols - 1),
                rng.random_range(1..rows - 1),
                rng.random_range(0..3),
            )
        })
        .collect();
    let mut seaweeds: Vec<Seaweed> = (0..30)
        .map(|_| {
            Seaweed::new(
                rng.random_range(1..cols - 1),
                rows - 5,
                rng.random_range(0..3),
                &mut rng,
            )
        })
        .collect();
    loop {
        terminal.draw(|f| {
            let area = f.area();
            let block = Block::default()
                .title("Aquatui")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White));
            f.render_widget(&block, area);

            let inner = block.inner(area);
            let mut lines =
                vec![Line::from(vec![Span::raw(" "); inner.width as usize]); inner.height as usize];

            // Affichage des poissons (multi-lignes)
            for fish in &fishes {
                for (dy, line) in fish.body.iter().enumerate() {
                    let y = fish.y as usize + dy;
                    if y >= lines.len() {
                        continue;
                    }

                    let chars: Vec<_> = line.chars().collect();
                    let start_x = fish.x as i16;
                    for (i, ch) in chars.into_iter().enumerate() {
                        let screen_x = start_x + i as i16;
                        if screen_x >= 0 && (screen_x as u16) < inner.width {
                            lines[y].spans[screen_x as usize] =
                                Span::styled(ch.to_string(), Style::default().fg(fish.color));
                        }
                    }
                }
            }

            // Bulles
            bubbles.sort_by_key(|b| b.z_index);
            for bubble in &bubbles {
                if (bubble.y as usize) < lines.len() {
                    let x = bubble.x.min(inner.width - 1) as usize;
                    let symbol = bubble.current_symbol();
                    lines[bubble.y as usize].spans[x] =
                        Span::styled(symbol, Style::default().fg(Color::Cyan));
                }
            }

            // Algues
            draw_seaweed(&mut lines, cols, rows);
            seaweeds.sort_by_key(|s| s.z_index);
            for seaweed in &seaweeds {
                let x = seaweed.x.min(inner.width - 1) as usize;
                let y = seaweed.y.min(inner.height - 1) as usize;
                let symbol = seaweed.current_symbol();
                let symbol_line = symbol.iter().rev().take(seaweed.height).collect::<Vec<_>>();
                for (dy, line) in symbol_line.iter().enumerate() {
                    let draw_y = y.saturating_sub(dy);
                    if draw_y >= lines.len() {
                        continue;
                    }
                    for (dx, char) in line.chars().enumerate() {
                        let draw_x = x + dx;
                        if draw_x >= lines[draw_y].spans.len() {
                            continue;
                        }
                        lines[draw_y].spans[draw_x] =
                            Span::styled(char.to_string(), Style::default().fg(Color::Green));
                    }
                }
            }

            // Kame House
            let house_lines: Vec<&str> = KAME_HOUSE.lines().collect();
            let house_width = house_lines.iter().map(|line| line.len()).max().unwrap_or(0);
            let house_height = house_lines.len();
            let start_x = cols.saturating_sub(house_width as u16 + 2) as usize;
            let seaweed_height = 4;
            let start_y = rows.saturating_sub(house_height as u16 + seaweed_height) as usize;

            for (dy, line) in house_lines.iter().enumerate() {
                let y = start_y + dy;
                if y >= lines.len() {
                    continue;
                }

                for (dx, ch) in line.chars().enumerate() {
                    let x = start_x + dx;
                    if x >= lines[y].spans.len() {
                        break;
                    }

                    lines[y].spans[x] =
                        Span::styled(ch.to_string(), Style::default().fg(Color::Magenta));
                }
            }

            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, inner);
        })?;

        if last_frame.elapsed() >= frame_duration {
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

            while fishes.len() < num_fish {
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

        // Gestion de la touche 'q'
        if poll(Duration::from_millis(1))? {
            if let Event::Key(key) = read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
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
