use crossterm::{
    event::{Event, KeyCode, poll, read},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::{Rng, prelude::IndexedRandom, rngs::ThreadRng};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{io, thread, time::Duration, time::Instant};

enum Direction {
    Left,
    Right,
}

struct Fish {
    x: u16,
    y: u16,
    body: String,
    color: Color,
    speed: u16,
    direction: Direction,
}

struct Bubble {
    x: u16,
    y: u16,
}

impl Fish {
    fn new(max_x: u16, max_y: u16, rng: &mut ThreadRng) -> Self {
        let direction = if rng.random_bool(0.5) {
            Direction::Right
        } else {
            Direction::Left
        };

        let bodies = match direction {
            Direction::Right => vec!["><((°>", "><>", ">º)))>", "⩿⩾⩽⩾"],
            Direction::Left => vec!["<°))><", "<><", "<(((º<", "⩾⩽⩾⩿"],
        };

        let body = bodies.choose(rng).unwrap().to_string();

        let x = match direction {
            Direction::Right => 0,
            Direction::Left => max_x,
        };

        let color = *[
            Color::Cyan,
            Color::LightMagenta,
            Color::Yellow,
            Color::LightBlue,
            Color::Green,
        ]
        .choose(rng)
        .unwrap();

        Self {
            x,
            y: rng.random_range(1..max_y),
            body,
            color,
            speed: rng.random_range(1..3),
            direction,
        }
    }

    fn update(&mut self, max_x: u16) {
        match self.direction {
            Direction::Right => {
                self.x += self.speed;
                if self.x > max_x {
                    self.x = 0;
                }
            }
            Direction::Left => {
                if self.x < self.speed {
                    self.x = max_x;
                } else {
                    self.x -= self.speed;
                }
            }
        }
    }
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

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let size = terminal.size()?;
    let (cols, rows) = (size.width, size.height);

    let mut rng = rand::rng();
    let num_fish = 15;
    let mut fishes: Vec<Fish> = (0..num_fish)
        .map(|_| Fish::new(cols - 2, rows - 2, &mut rng))
        .collect();

    let frame_duration = Duration::from_millis(60);
    let mut last_frame = Instant::now();

    let mut bubbles: Vec<Bubble> = (0..20)
        .map(|_| Bubble {
            x: rng.random_range(1..cols - 1),
            y: rng.random_range(1..rows - 1),
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
            let mut lines = vec![Line::from(""); inner.height as usize];

            // Fond océan dégradé
            for i in 0..lines.len() {
                let shade = match i {
                    0..=3 => Color::Blue,
                    4..=6 => Color::Rgb(0, 0, 139),
                    _ => Color::Black,
                };
                lines[i] = Line::from(Span::styled(
                    " ".repeat(inner.width as usize),
                    Style::default().bg(shade),
                ));
            }

            let mut lines: Vec<Line> = vec![Line::from(""); inner.height as usize];

            for fish in &fishes {
                if (fish.y as usize) < lines.len() {
                    let mut line = String::new();
                    let display_x = fish
                        .x
                        .min(inner.width.saturating_sub(fish.body.len() as u16));
                    line.push_str(&" ".repeat(display_x as usize));
                    line.push_str(&fish.body);
                    lines[fish.y as usize] =
                        Line::from(Span::styled(line, Style::default().fg(fish.color)));
                }
            }

            // Bulles
            for bubble in &bubbles {
                if (bubble.y as usize) < lines.len() {
                    let x = bubble.x.min(inner.width - 1) as usize;
                    if x < inner.width as usize {
                        let mut content = lines[bubble.y as usize].clone();
                        let bubble_span = Span::styled("o", Style::default().fg(Color::White));
                        if x < content.spans.len() {
                            content.spans[x] = bubble_span;
                        } else {
                            content.spans.push(bubble_span);
                        }
                        lines[bubble.y as usize] = content;
                    }
                }
            }

            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, inner);
        })?;

        if last_frame.elapsed() >= frame_duration {
            // Mise à jour des poissons
            for fish in &mut fishes {
                fish.update(cols - 2);
            }

            // Mise à jour des bulles
            for bubble in &mut bubbles {
                if bubble.y > 0 {
                    bubble.y -= 1;
                } else {
                    bubble.y = rows - 2;
                    bubble.x = rng.random_range(1..cols - 1);
                }
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

        // Pause très courte pour éviter 100% CPU
        thread::sleep(Duration::from_millis(20));
    }

    Ok(())
}
